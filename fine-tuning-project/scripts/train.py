#!/usr/bin/env python3
"""
Main Training Script for Qwen3-0.6B Fine-Tuning

Production-grade fine-tuning using Unsloth + QLoRA on Google Colab T4 16GB.
Dataset format: JSONL with "text" field.

Usage:
    python scripts/train.py --dataset path/to/dataset.jsonl

Reference:
- https://unsloth.ai/blog/long-context
- https://huggingface.co/docs/transformers/main_classes/trainer
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
from src.training.mixed_precision import setup_mixed_precision
from src.training.callbacks import (
    VRAMMonitorCallback, 
    DynamicConfigCallback,
    EarlyStoppingCallback,
    ValidationLossLoggerCallback,
)
from src.training.metrics import compute_perplexity_only
from src.models.lora_config import get_dynamic_lora_config


# ===== CONFIGURATION =====
MODEL_NAME = "Qwen/Qwen3-0.6B"  # Qwen3 0.6B (not Base version)
VRAM_GB = 16.0  # Google Colab T4

# W&B Setup (optional)
USE_WANDB = False
try:
    import wandb
    USE_WANDB = True
except ImportError:
    print("⚠️  wandb not installed, using TensorBoard only")


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Fine-tune Qwen3-0.6B")
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
        "--use_unsloth", 
        action="store_true",
        help="Use Unsloth for faster training (requires unsloth installed)"
    )
    parser.add_argument(
        "--wandb_project", 
        type=str, 
        default="qwen3-finetuning",
        help="Weights & Biases project name"
    )
    return parser.parse_args()


def main():
    args = parse_args()
    
    print("=" * 80)
    print("🚀 Qwen3-0.6B Fine-Tuning Script")
    print("=" * 80)
    
    # ===== 1. MIXED PRECISION SETUP =====
    bf16_support, fp16_support, precision_mode = setup_mixed_precision()
    
    # ===== 2. LOAD DATASET =====
    print(f"\n📥 Loading dataset from: {args.dataset}")
    full_dataset = load_dataset("json", data_files={"train": args.dataset}, split="train")
    print(f"   Total samples: {len(full_dataset):,}")
    
    # ===== 3. SPLIT DATASET (80/10/10) =====
    dataset_dict = split_dataset(
        full_dataset,
        train_ratio=0.80,
        val_ratio=0.10,
        test_ratio=0.10,
        seed=42
    )
    
    # Save test set for final evaluation (DON'T TOUCH UNTIL TRAINING COMPLETE!)
    os.makedirs(args.output_dir, exist_ok=True)
    test_dataset_path = f"{args.output_dir}/test_dataset.json"
    dataset_dict["test"].to_json(test_dataset_path)
    print(f"✅ Test dataset saved to: {test_dataset_path}")
    print("⚠️  DO NOT use test set until training is fully complete!")
    
    # ===== 4. LOAD TOKENIZER =====
    print(f"\n📝 Loading tokenizer: {MODEL_NAME}")
    tokenizer = AutoTokenizer.from_pretrained(MODEL_NAME, use_fast=True)
    if tokenizer.pad_token is None:
        tokenizer.add_special_tokens({'pad_token': '[PAD]'})
    
    # ===== 5. ANALYZE DATASET (TRAIN SET ONLY!) =====
    train_dataset, dynamic_config = analyze_dataset_and_configure(
        dataset_dict["train"], 
        tokenizer, 
        max_length=32768, 
        vram_gb=VRAM_GB
    )
    
    # Analyze distribution for each split
    analyze_split_distribution(dataset_dict, tokenizer)
    
    # ===== 6. LOAD MODEL =====
    print(f"\n🔥 Loading model: {MODEL_NAME}")
    
    if args.use_unsloth:
        try:
            from unsloth import FastLanguageModel
            model, tokenizer = FastLanguageModel.from_pretrained(
                model_name=MODEL_NAME,
                max_seq_length=dynamic_config["max_seq_length"],
                dtype=torch.bfloat16 if bf16_support else None,
                load_in_4bit=True,
                device_map="auto"
            )
            print("✅ Loaded with Unsloth (faster training)")
        except ImportError:
            print("⚠️  Unsloth not installed, using standard transformers")
            args.use_unsloth = False
    
    if not args.use_unsloth:
        from transformers import AutoModelForCausalLM, BitsAndBytesConfig
        from peft import get_peft_model, LoraConfig
        
        # 4-bit quantization config
        bnb_config = BitsAndBytesConfig(
            load_in_4bit=True,
            bnb_4bit_quant_type="nf4",
            bnb_4bit_compute_dtype=torch.float16,
            bnb_4bit_use_double_quant=True,
        )
        
        model = AutoModelForCausalLM.from_pretrained(
            MODEL_NAME,
            quantization_config=bnb_config,
            device_map="auto",
            trust_remote_code=True,
        )
        print("✅ Loaded with transformers + bitsandbytes (4-bit)")
    
    # Resize embeddings if added special tokens
    model.resize_token_embeddings(len(tokenizer))
    
    # ===== 7. SETUP LORA/QLORA =====
    lora_config_dict = get_dynamic_lora_config(MODEL_NAME, dynamic_config["max_seq_length"])
    
    if args.use_unsloth:
        from unsloth import FastLanguageModel
        model = FastLanguageModel.get_peft_model(
            model,
            r=lora_config_dict["r"],
            lora_alpha=lora_config_dict["lora_alpha"],
            target_modules=lora_config_dict["target_modules"],
            lora_dropout=lora_config_dict["lora_dropout"],
            bias=lora_config_dict["bias"],
            use_gradient_checkpointing=dynamic_config["use_gradient_checkpointing"],
            use_rslora=lora_config_dict["use_rslora"],
            random_state=3407
        )
    else:
        from peft import get_peft_model, LoraConfig
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
    
    # ===== 8. DATA COLLATOR =====
    data_collator = DataCollatorForLanguageModeling(
        tokenizer=tokenizer,
        mlm=False,
        pad_to_multiple_of=8
    )
    
    # ===== 9. W&B SETUP =====
    if USE_WANDB:
        try:
            wandb.login()
            os.environ["WANDB_PROJECT"] = args.wandb_project
            print(f"✅ Weights & Biases enabled: {args.wandb_project}")
        except:
            global USE_WANDB
            USE_WANDB = False
            print("⚠️  W&B login failed, using TensorBoard only")
    
    # ===== 10. TRAINING ARGUMENTS =====
    training_args = TrainingArguments(
        output_dir=args.output_dir,
        per_device_train_batch_size=dynamic_config["per_device_train_batch_size"],
        gradient_accumulation_steps=dynamic_config["gradient_accumulation_steps"],
        
        # Learning rate & schedule
        learning_rate=2e-5,
        num_train_epochs=args.epochs,
        lr_scheduler_type="cosine",
        warmup_ratio=0.1,
        
        # Mixed precision
        bf16=bf16_support,
        fp16=fp16_support,
        
        # Optimizer
        optim="paged_adamw_8bit",
        weight_decay=0.01,
        max_grad_norm=1.0,
        
        # Gradient checkpointing
        gradient_checkpointing=dynamic_config["use_gradient_checkpointing"],
        gradient_checkpointing_kwargs={"use_reentrant": False},  # PyTorch 2.0+
        
        # Evaluation
        eval_strategy="steps",
        eval_steps=100,
        per_device_eval_batch_size=2,
        load_best_model_at_end=True,
        metric_for_best_model="eval_loss",
        greater_is_better=False,
        
        # Logging & Saving
        logging_steps=10,
        save_strategy="steps",
        save_steps=100,
        save_total_limit=3,
        
        # Reporting
        report_to=["wandb", "tensorboard"] if USE_WANDB else ["tensorboard"],
        
        # Performance
        dataloader_num_workers=2,
        dataloader_pin_memory=True,
        torch_compile=False,  # Disable for compatibility
        
        # Memory optimization
        ddp_find_unused_parameters=False,
    )
    
    # ===== 11. CALLBACKS =====
    callbacks = [
        VRAMMonitorCallback(threshold_percent=95.0),
        DynamicConfigCallback(dynamic_config),
        EarlyStoppingCallback(patience=5, min_delta=0.001),
        ValidationLossLoggerCallback(),
    ]
    
    # ===== 12. TRAINER =====
    try:
        from trl import SFTTrainer
        trainer = SFTTrainer(
            model=model,
            args=training_args,
            train_dataset=dataset_dict["train"],
            eval_dataset=dataset_dict["validation"],
            data_collator=data_collator,
            callbacks=callbacks,
            compute_metrics=compute_perplexity_only,
            dataset_text_field="text",
            max_seq_length=dynamic_config["max_seq_length"],
            packing=False,
        )
        print("✅ Using SFTTrainer from TRL")
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
    
    # ===== 13. TRAINING =====
    print("\n" + "=" * 80)
    print("🚀 Starting Training with Validation...")
    print("=" * 80 + "\n")
    
    train_result = trainer.train()
    
    # ===== 14. SAVE MODEL =====
    print("\n💾 Saving final model...")
    final_model_path = f"{args.output_dir}/final_model"
    trainer.save_model(final_model_path)
    tokenizer.save_pretrained(final_model_path)
    
    # ===== 15. FINAL VALIDATION =====
    print("\n📊 Running final validation...")
    val_results = trainer.evaluate()
    print(f"\n✅ Final Validation Results:")
    print(f"   Validation Loss: {val_results.get('eval_loss', 'N/A'):.4f}")
    print(f"   Validation Perplexity: {val_results.get('eval_perplexity', 'N/A')}")
    
    print(f"\n✅ Training completed! Model saved to: {final_model_path}")
    print(f"\n📊 Training Stats:")
    print(f"   Total steps: {train_result.global_step}")
    print(f"   Training loss: {train_result.training_loss:.4f}")
    
    print(f"\n⚠️  TEST SET EVALUATION:")
    print(f"   Test dataset saved at: {test_dataset_path}")
    print(f"   Run: python scripts/test_model.py --model {final_model_path} --dataset {test_dataset_path}")
    
    # ===== 16. CLEANUP =====
    if USE_WANDB:
        wandb.finish()


if __name__ == "__main__":
    main()
