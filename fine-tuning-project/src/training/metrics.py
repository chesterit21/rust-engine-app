"""
Metrics Module

Compute evaluation metrics for language model fine-tuning.

Reference:
- https://huggingface.co/docs/transformers/perplexity
- https://github.com/huggingface/transformers/issues/32307
"""

import torch
import numpy as np
from typing import Dict
from transformers import EvalPrediction


def compute_metrics(eval_pred: EvalPrediction) -> Dict[str, float]:
    """
    Compute evaluation metrics for language model.
    
    Metrics:
    - Perplexity: exp(loss) - main metric for language models
    - Token Accuracy: Token-level accuracy (optional)
    
    Args:
        eval_pred: Evaluation predictions from trainer
    
    Returns:
        Dictionary with perplexity, eval_loss, and token_accuracy
    """
    logits = eval_pred.predictions
    labels = eval_pred.label_ids
    
    # Reshape for loss calculation
    # Flatten: (batch_size * seq_len, vocab_size) and (batch_size * seq_len,)
    logits_flat = logits.reshape(-1, logits.shape[-1])
    labels_flat = labels.reshape(-1)
    
    # Filter padding tokens (usually -100)
    mask = labels_flat != -100
    logits_flat = logits_flat[mask]
    labels_flat = labels_flat[mask]
    
    # Calculate Cross Entropy Loss
    loss_fct = torch.nn.CrossEntropyLoss()
    loss = loss_fct(
        torch.from_numpy(logits_flat).float(),
        torch.from_numpy(labels_flat).long()
    )
    
    # Perplexity = exp(loss)
    perplexity = torch.exp(loss).item()
    
    # Token-level accuracy (optional)
    predictions = np.argmax(logits_flat, axis=-1)
    accuracy = (predictions == labels_flat).mean()
    
    return {
        "perplexity": perplexity,
        "eval_loss": loss.item(),
        "token_accuracy": float(accuracy)
    }


def compute_perplexity_only(eval_pred: EvalPrediction) -> Dict[str, float]:
    """
    Simplified version - only compute perplexity from loss.
    Faster because no accuracy computation.
    
    Args:
        eval_pred: Evaluation predictions from trainer
    
    Returns:
        Dictionary with perplexity only
    """
    logits = eval_pred.predictions
    labels = eval_pred.label_ids
    
    # Reshape
    logits_flat = logits.reshape(-1, logits.shape[-1])
    labels_flat = labels.reshape(-1)
    
    # Filter padding
    mask = labels_flat != -100
    logits_flat = logits_flat[mask]
    labels_flat = labels_flat[mask]
    
    # Loss
    loss_fct = torch.nn.CrossEntropyLoss()
    loss = loss_fct(
        torch.from_numpy(logits_flat).float(),
        torch.from_numpy(labels_flat).long()
    )
    
    # Perplexity
    perplexity = torch.exp(loss).item()
    
    return {
        "perplexity": perplexity,
    }
