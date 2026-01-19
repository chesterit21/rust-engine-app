#!/usr/bin/env python3
"""
Main Training Script for Gemma 3 270M Fine-Tuning on TPU

Production-grade fine-tuning using PyTorch XLA + QLoRA on Google Colab TPU.
Dataset format: JSONL with "messages" field.

Usage:
    python scripts/train.py --dataset path/to/dataset.jsonl

TPU Optimizations:
- Uses bfloat16 (TPU native precision)
- Batch size optimized for TPU (multiples of 128)
- Reduced logging frequency to avoid host-device sync overhead
- Uses xm.optimizer_step() for gradient synchronization

Reference:
- https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch
- https://pytorch.org/xla/release/r2.8/learn/xla-overview.html
"""

import torch
import os
import sys
import argparse
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from datasets import load_dataset
from transformers import AutoTokenizer, TrainingArguments, DataCollatorForLanguageModeling

# Import our modules
from src.data.dataset_analyzer import analyze_dataset_and_configure
from src.data.dataset_splitter import split_dataset, analyze_split_distribution
from src.training.mixed_precision import setup_mixed_precision_tpu, get_tpu_world_size
from src.training.callbacks import (
    TPULoggingCallback,
    DynamicConfigCallback,
    EarlyStoppingCallback,
    ValidationLossLoggerCallback,
)
from src.training.metrics import compute_perplexity_only
from src.models.lora_config import get_dynamic_lora_config


# ===== CONFIGURATION =====
MODEL_NAME = "google/gemma-3-270m-it"  # Gemma 3 270M Instruct
TPU_CORES = 8  # TPU v2/v3 has 8 cores

# W&B Setup (optional)
USE_WANDB = False
try:
    import wandb
    USE_WANDB = True
except ImportError:
    print("⚠️  wandb not installed, using TensorBoard only")


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Fine-tune Gemma 3 270M on TPU")
    parser.add_argument(
        "--dataset", 
        type=str, 
        required=True,
        help="Path to JSONL dataset file"
    )
    parser.add_argument(
        "--output_dir", 
        type=str, 
        default="./outputs",
        help="Output directory for checkpoints and model"
    )
    parser.add_argument(
        "--epochs", 
        type=int, 
        default=3,
        help="Number of training epochs"
    )
    parser.add_argument(
        "--wandb_project", 
        type=str, 
        default="gemma3-finetuning-tpu",
        help="Weights & Biases project name"
    )
    parser.add_argument(
        "--batch_size_per_replica",
        type=int,
        default=128,
        help="Batch size per TPU replica (should be multiple of 128)"
    )
    return parser.parse_args()


def main():
    args = parse_args()
    
    print("=" * 80)
    print("🚀 Gemma 3 270M Fine-Tuning Script (TPU Edition)")
    print("=" * 80)
    
    # ===== 1. TPU SETUP =====
    print("\n🔧 Setting up TPU...")
    try:
        import torch_xla
        import torch_xla.core.xla_model as xm
        import torch_xla.distributed.parallel_loader as pl
        
        device = xm.xla_device()
        world_size = xm.xrt_world_size()
        print(f"✅ TPU initialized: {device}")
        print(f"   TPU cores: {world_size}")
    except ImportError:
        print("❌ torch_xla not installed!")
        print("   Run: !pip install cloud-tpu-client torch torch_xla")
        sys.exit(1)
    except Exception as e:
        print(f"❌ TPU initialization failed: {e}")
        print("   Make sure Runtime > Change runtime type > TPU")
        sys.exit(1)
    
    # ===== 2. MIXED PRECISION SETUP (bfloat16 for TPU) =====
    bf16_support, fp16_support, precision_mode = setup_mixed_precision_tpu()
    
    # ===== 3. LOAD DATASET =====
    print(f"\n📥 Loading dataset from: {args.dataset}")
    full_dataset = load_dataset("json", data_files={"train": args.dataset}, split="train")
    print(f"   Total samples: {len(full_dataset):,}")
    
    # ===== 4. SPLIT DATASET (80/10/10) =====
    dataset_dict = split_dataset(
        full_dataset,
        train_ratio=0.80,
        val_ratio=0.10,
        test_ratio=0.10,
        seed=42
    )
    
    # Save test set for final evaluation
    os.makedirs(args.output_dir, exist_ok=True)
    test_dataset_path = f"{args.output_dir}/test_dataset.json"
    dataset_dict["test"].to_json(test_dataset_path)
    print(f"✅ Test dataset saved to: {test_dataset_path}")
    print("⚠️  DO NOT use test set until training is fully complete!")
    
    # ===== 5. LOAD TOKENIZER =====
    print(f"\n📝 Loading tokenizer: {MODEL_NAME}")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, use_fast=True)
    if tokenizer.pad_token is None:
        tokenizer.add_special_tokens({'pad_token': '[PAD]'})
    
    # ===== 6. ANALYZE DATASET (TRAIN SET ONLY!) =====
    # Gemma 3 270M has 32K context window (not 128K like larger variants)
    train_dataset, dynamic_config = analyze_dataset_and_configure(
        dataset_dict["train"], 
        tokenizer, 
        max_length=32768,  # Gemma 3 270M max context
        vram_gb=16.0  # Not relevant for TPU but kept for compatibility
    )
    
    # Override batch size for TPU optimization
    # TPU works best with batch sizes that are multiples of 128
    dynamic_config["per_device_train_batch_size"] = args.batch_size_per_replica
    dynamic_config["gradient_accumulation_steps"] = max(1, 256 // args.batch_size_per_replica)
    dynamic_config["global_batch_size"] = (
        dynamic_config["per_device_train_batch_size"] * 
        world_size * 
        dynamic_config["gradient_accumulation_steps"]
    )
    
    print(f"\n🔧 TPU-Optimized Configuration:")
    print(f"   Batch size per replica: {dynamic_config['per_device_train_batch_size']}")
    print(f"   TPU cores: {world_size}")
    print(f"   Gradient accumulation: {dynamic_config['gradient_accumulation_steps']}")
    print(f"   Global batch size: {dynamic_config['global_batch_size']}")
    
    # Analyze distribution for each split
    analyze_split_distribution(dataset_dict, tokenizer)
    
    # ===== 7. LOAD MODEL =====
    print(f"\n🔥 Loading model: {MODEL_NAME}")
    
    from transformers import AutoModelForCausalLM
    from peft import get_peft_model, LoraConfig
    
    # Load model with official Google recommended settings
    model = AutoModelForCausalLM.from_pretrained(
        MODEL_NAME,
        torch_dtype="auto",  # Auto-detect best dtype
        device_map="auto",
        attn_implementation="eager",  # Official Google recommendation
        trust_remote_code=True,
    )
    
    print(f"✅ Model loaded with dtype: {model.dtype}")
    print(f"   Device: {model.device}")
    print(f"   Attention: eager (Google recommended)")
    
    # Resize embeddings if added special tokens
    model.resize_token_embeddings(len(tokenizer))
    
    # ===== 8. SETUP LORA =====
    lora_config_dict = get_dynamic_lora_config(MODEL_NAME, dynamic_config["max_seq_length"])
    
    peft_config = LoraConfig(
        r=lora_config_dict["r"],
        lora_alpha=lora_config_dict["lora_alpha"],
        target_modules=lora_config_dict["target_modules"],
        lora_dropout=lora_config_dict["lora_dropout"],
        bias=lora_config_dict["bias"],
        task_type=lora_config_dict["task_type"],
    )
    model = get_peft_model(model, peft_config)
    model.print_trainable_parameters()
    
    # ===== 9. DATA COLLATOR =====
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=False,
        pad_to_multiple_of=128  # TPU optimization: pad to multiples of 128
    )
    
    # ===== 10. W&B SETUP =====
    if USE_WANDB:
        try:
            wandb.login()
            os.environ["WANDB_PROJECT"] = args.wandb_project
            print(f"✅ Weights & Biases enabled: {args.wandb_project}")
        except:
            global USE_WANDB
            USE_WANDB = False
            print("⚠️  W&B login failed, using TensorBoard only")
    
    # ===== 11. TRAINING ARGUMENTS (Official Google Pattern) =====
    # Detect model dtype for precision settings
    torch_dtype = model.dtype
    
    training_args = TrainingArguments(
        output_dir=args.output_dir,
        per_device_train_batch_size=dynamic_config["per_device_train_batch_size"],
        gradient_accumulation_steps=dynamic_config["gradient_accumulation_steps"],
        
        # Learning rate & schedule (Official Google: 5e-5, constant)
        learning_rate=5e-5,  # Official Google recommendation
        num_train_epochs=args.epochs,
        lr_scheduler_type="constant",  # Official Google recommendation
        warmup_ratio=0.1,
        
        # Precision - auto-detect from model
        bf16=True if torch_dtype == torch.bfloat16 else False,
        fp16=True if torch_dtype == torch.float16 else False,
        
        # Optimizer - Official Google recommendation
        optim="adamw_torch_fused",  # Fused optimizer for better performance
        weight_decay=0.01,
        max_grad_norm=1.0,
        
        # Gradient checkpointing - disabled for caching compatibility
        gradient_checkpointing=False,  # Google: "Caching is incompatible with gradient checkpointing"
        
        # Evaluation
        eval_strategy="epoch",  # Official Google: per epoch
        per_device_eval_batch_size=dynamic_config["per_device_train_batch_size"],
        load_best_model_at_end=True,
        metric_for_best_model="eval_loss",
        greater_is_better=False,
        
        # Logging & Saving
        logging_steps=1,  # Official Google: log every step
        save_strategy="epoch",  # Official Google: save per epoch
        save_total_limit=3,
        
        # Reporting
        report_to=["wandb", "tensorboard"] if USE_WANDB else ["tensorboard"],
        
        # Performance
        dataloader_num_workers=4,
        dataloader_pin_memory=True,
        torch_compile=False,
        
        # Push to Hub (optional)
        push_to_hub=False,
        
        # TPU specific
        dataloader_drop_last=True,
    )
    
    # ===== 12. CALLBACKS =====
    callbacks = [
        TPULoggingCallback(log_every_n_steps=50),  # Reduced logging
        DynamicConfigCallback(dynamic_config),
        EarlyStoppingCallback(patience=5, min_delta=0.001),
        ValidationLossLoggerCallback(),
    ]
    
    # ===== 13. TRAINER (Official Google Pattern) =====
    try:
        from trl import SFTTrainer
        trainer = SFTTrainer(
            model=model,
            args=training_args,
            train_dataset=dataset_dict["train"],
            eval_dataset=dataset_dict["validation"],
            callbacks=callbacks,
            processing_class=tokenizer,  # Official Google pattern
            max_seq_length=dynamic_config["max_seq_length"],
            packing=False,  # Official Google: packing=False
            dataset_kwargs={
                "add_special_tokens": False,  # Template with special tokens
                "append_concat_token": True,  # Add EOS token as separator
            },
        )
        print("✅ Using SFTTrainer from TRL (Official Google pattern)")
    except ImportError:
        from transformers import Trainer
        trainer = Trainer(
            model=model,
            args=training_args,
            train_dataset=dataset_dict["train"],
            eval_dataset=dataset_dict["validation"],
            data_collator=data_collator,
            callbacks=callbacks,
            compute_metrics=compute_perplexity_only,
        )
        print("✅ Using Trainer from transformers")
    
    # ===== 14. TRAINING =====
    print("\n" + "=" * 80)
    print("🚀 Starting Training on TPU...")
    print("=" * 80 + "\n")
    
    train_result = trainer.train()
    
    # ===== 15. SAVE MODEL =====
    print("\n💾 Saving final model...")
    final_model_path = f"{args.output_dir}/final_model"
    
    # For TPU, use xm.save for proper saving
    try:
        import torch_xla.core.xla_model as xm
        if xm.is_master_ordinal():
            trainer.save_model(final_model_path)
            tokenizer.save_pretrained(final_model_path)
    except:
        trainer.save_model(final_model_path)
        tokenizer.save_pretrained(final_model_path)
    
    # ===== 16. FINAL VALIDATION =====
    print("\n📊 Running final validation...")
    val_results = trainer.evaluate()
    print(f"\n✅ Final Validation Results:")
    print(f"   Validation Loss: {val_results.get('eval_loss', 'N/A'):.4f}")
    print(f"   Validation Perplexity: {val_results.get('eval_perplexity', 'N/A')}")
    
    print(f"\n✅ Training completed on TPU! Model saved to: {final_model_path}")
    print(f"\n📊 Training Stats:")
    print(f"   Total steps: {train_result.global_step}")
    print(f"   Training loss: {train_result.training_loss:.4f}")
    
    print(f"\n⚠️  TEST SET EVALUATION:")
    print(f"   Test dataset saved at: {test_dataset_path}")
    print(f"   Run: python scripts/test_model.py --model {final_model_path} --dataset {test_dataset_path}")
    
    # ===== 17. CLEANUP =====
    if USE_WANDB:
        wandb.finish()


if __name__ == "__main__":
    main()
