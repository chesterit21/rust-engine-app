#!/usr/bin/env python3
"""
Test Model Script

Evaluate trained model on test set (ONLY after training is complete!).

Usage:
    python scripts/test_model.py --model ./outputs/final_model --dataset ./outputs/test_dataset.json

Reference:
- https://huggingface.co/docs/transformers/perplexity
"""

import torch
import argparse
import numpy as np
from pathlib import Path
from datasets import load_dataset
from transformers import AutoTokenizer, AutoModelForCausalLM
from tqdm import tqdm


def parse_args():
    """Parse command line arguments."""
    parser = argparse.ArgumentParser(description="Evaluate fine-tuned model on test set")
    parser.add_argument(
        "--model", 
        type=str, 
        required=True,
        help="Path to trained model"
    )
    parser.add_argument(
        "--dataset", 
        type=str, 
        required=True,
        help="Path to test dataset (JSON/JSONL)"
    )
    parser.add_argument(
        "--max_samples", 
        type=int, 
        default=None,
        help="Maximum number of samples to evaluate"
    )
    parser.add_argument(
        "--max_length", 
        type=int, 
        default=2048,
        help="Maximum sequence length for evaluation"
    )
    return parser.parse_args()


def evaluate_on_test_set(model_path: str, test_dataset_path: str, max_samples: int = None, max_length: int = 2048):
    """
    Evaluate trained model on test set.
    
    Metrics:
    - Test Loss
    - Test Perplexity
    - Token-level Accuracy
    
    Args:
        model_path: Path to trained model
        test_dataset_path: Path to test dataset
        max_samples: Maximum samples to evaluate (None = all)
        max_length: Maximum sequence length
    
    Returns:
        Dictionary with test metrics
    """
    print(f"\n🧪 Testing model: {model_path}")
    print(f"   Test dataset: {test_dataset_path}")
    
    # Load model and tokenizer
    print("\n📥 Loading model...")
    tokenizer = AutoTokenizer.from_pretrained(model_path)
    model = AutoModelForCausalLM.from_pretrained(
        model_path,
        torch_dtype=torch.bfloat16 if torch.cuda.get_device_capability()[0] >= 8 else torch.float16,
        device_map="auto"
    )
    model.eval()
    
    # Load test dataset
    print("📥 Loading test dataset...")
    test_dataset = load_dataset("json", data_files=test_dataset_path, split="train")
    if max_samples:
        test_dataset = test_dataset.select(range(min(max_samples, len(test_dataset))))
    
    print(f"   Test samples: {len(test_dataset)}")
    
    # Evaluate
    total_loss = 0.0
    total_tokens = 0
    total_correct = 0
    
    print("\n📊 Evaluating...")
    with torch.no_grad():
        for example in tqdm(test_dataset, desc="Evaluating"):
            # Tokenize
            inputs = tokenizer(
                example["text"],
                return_tensors="pt",
                truncation=True,
                max_length=max_length
            ).to(model.device)
            
            # Forward pass
            outputs = model(**inputs, labels=inputs["input_ids"])
            
            # Accumulate metrics
            seq_len = inputs["input_ids"].size(1)
            loss = outputs.loss.item()
            total_loss += loss * seq_len
            total_tokens += seq_len
            
            # Accuracy (optional)
            logits = outputs.logits
            predictions = torch.argmax(logits[:, :-1, :], dim=-1)
            targets = inputs["input_ids"][:, 1:]
            total_correct += (predictions == targets).sum().item()
    
    # Calculate final metrics
    avg_loss = total_loss / total_tokens
    perplexity = np.exp(avg_loss)
    accuracy = total_correct / total_tokens
    
    print(f"\n" + "=" * 60)
    print(f"📊 TEST SET RESULTS")
    print(f"=" * 60)
    print(f"   Test Loss: {avg_loss:.4f}")
    print(f"   Test Perplexity: {perplexity:.4f}")
    print(f"   Token Accuracy: {accuracy*100:.2f}%")
    print(f"   Total Tokens: {total_tokens:,}")
    print(f"=" * 60)
    
    return {
        "test_loss": avg_loss,
        "test_perplexity": perplexity,
        "test_accuracy": accuracy
    }


def main():
    args = parse_args()
    
    print("=" * 80)
    print("🧪 Gemma 3 270M Model Evaluation")
    print("=" * 80)
    
    results = evaluate_on_test_set(
        model_path=args.model,
        test_dataset_path=args.dataset,
        max_samples=args.max_samples,
        max_length=args.max_length
    )
    
    # Save results
    import json
    results_path = Path(args.model).parent / "test_results.json"
    with open(results_path, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\n✅ Results saved to: {results_path}")


if __name__ == "__main__":
    main()
