"""
Mixed Precision Handler Module

Setup mixed precision training with fallback strategy:
BF16 (best) → FP16 (fallback) → FP32 (CPU/old GPU)

Reference:
- https://pytorch.org/blog/what-every-user-should-know-about-mixed-precision-training-in-pytorch/
- https://www.runpod.io/articles/guides/fp16-bf16-fp8-mixed-precision-speed-up-my-model-training
"""

import torch
from typing import Tuple


def setup_mixed_precision() -> Tuple[bool, bool, str]:
    """
    Setup mixed precision with fallback strategy.
    
    Returns:
        Tuple of (bf16_support, fp16_support, precision_mode)
    """
    bf16_support = False
    fp16_support = False
    precision_mode = "fp32"
    
    if torch.cuda.is_available():
        # Check BF16 support (Ampere and newer: A100, RTX 3090, RTX 4090)
        # Note: T4 compute capability = 7.5, so bf16 = False
        if torch.cuda.get_device_capability()[0] >= 8:
            bf16_support = True
            precision_mode = "bf16"
            print("✅ BF16 mixed precision ENABLED (best option)")
        else:
            # Fallback to FP16 for older GPUs (T4, V100, etc.)
            fp16_support = True
            precision_mode = "fp16"
            print("✅ FP16 mixed precision ENABLED (fallback mode)")
            print("⚠️  Note: FP16 requires gradient scaling for stability")
            
        # Print GPU info
        gpu_name = torch.cuda.get_device_name(0)
        compute_cap = torch.cuda.get_device_capability(0)
        total_mem = torch.cuda.get_device_properties(0).total_memory / 1e9
        print(f"🖥️  GPU: {gpu_name} (Compute {compute_cap[0]}.{compute_cap[1]}, {total_mem:.1f}GB)")
    else:
        print("⚠️  No CUDA detected, using FP32 (slow)")
    
    return bf16_support, fp16_support, precision_mode
