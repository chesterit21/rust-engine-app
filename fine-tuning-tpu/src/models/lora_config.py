"""
Dynamic LoRA Configuration Module

Generate LoRA/QLoRA configuration based on model size.

Reference:
- https://unsloth.ai/docs/get-started/fine-tuning-llms-guide/lora-hyperparameters-guide
- https://devtechtools.org/en/blog/lora-vs-qlora-production-fine-tuning-commodity-gpus
"""

from typing import Dict


def get_dynamic_lora_config(model_name: str, max_seq_length: int) -> Dict:
    """
    Generate dynamic LoRA configuration based on model size.
    
    Args:
        model_name: Name/path of the model
        max_seq_length: Maximum sequence length for training
    
    Returns:
        Dictionary with LoRA configuration
    """
    model_lower = model_name.lower()
    
    # Target modules: QLoRA-All performs best
    target_modules = [
        "q_proj", "k_proj", "v_proj", "o_proj",  # Attention
        "gate_proj", "up_proj", "down_proj"       # FFN/MLP
    ]
    
    # Dynamic rank (r) based on model size
    if "0.5b" in model_lower or "0.6b" in model_lower or "270m" in model_lower:
        r, alpha = 8, 16
    elif "1.5b" in model_lower or "1b" in model_lower or "1.7b" in model_lower:
        r, alpha = 16, 32
    elif "6b" in model_lower or "7b" in model_lower:
        r, alpha = 32, 64
    else:
        r, alpha = 16, 32  # Default
    
    # Use RSLoRA for larger models (>3B)
    use_rslora = "6b" in model_lower or "7b" in model_lower
    
    config = {
        "r": r,
        "lora_alpha": alpha,
        "target_modules": target_modules,
        "lora_dropout": 0,  # Disabled for faster training
        "bias": "none",
        "task_type": "CAUSAL_LM",
        "use_rslora": use_rslora,
        "use_gradient_checkpointing": "unsloth"  # Unsloth GC is 30% more efficient
    }
    
    # Estimate trainable parameters
    estimated_params = r * 2 * len(target_modules) * 2048  # Rough estimate for small models
    
    print(f"\n🔧 Dynamic LoRA Config for {model_name}:")
    print(f"   Rank (r): {r}")
    print(f"   Alpha: {alpha}")
    print(f"   Target Modules: {len(target_modules)} layers")
    print(f"   RSLoRA: {use_rslora}")
    print(f"   Trainable Params: ~{estimated_params / 1e6:.2f}M\n")
    
    return config
