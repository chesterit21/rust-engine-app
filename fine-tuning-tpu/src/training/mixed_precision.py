"""
Mixed Precision Handler Module for TPU

Setup mixed precision training for TPU with bfloat16.
TPU ONLY supports bfloat16, not fp16.

Reference:
- https://cloud.google.com/blog/products/ai-machine-learning/bfloat16-the-secret-to-high-performance-on-cloud-tpus
- https://lightning.ai/docs/pytorch/1.5.9/advanced/mixed_precision.html
"""

import os
from typing import Tuple


def check_tpu_available() -> bool:
    """Check if TPU is available."""
    try:
        import torch_xla.core.xla_model as xm
        device = xm.xla_device()
        return True
    except Exception:
        return False


def setup_mixed_precision_tpu() -> Tuple[bool, bool, str]:
    """
    Setup mixed precision for TPU with bfloat16.
    
    TPU ONLY supports bfloat16, not fp16!
    
    Returns:
        Tuple of (bf16_support, fp16_support, precision_mode)
    """
    bf16_support = True   # TPU always uses bfloat16
    fp16_support = False  # TPU does NOT support fp16
    precision_mode = "bf16"
    
    if check_tpu_available():
        print("✅ TPU BFloat16 mixed precision ENABLED")
        print("ℹ️  TPU native precision: bfloat16 (NOT fp16)")
        
        try:
            import torch_xla.core.xla_model as xm
            device = xm.xla_device()
            print(f"🖥️  TPU Device: {device}")
            
            # Get number of TPU cores
            world_size = xm.xrt_world_size()
            print(f"🔢 TPU Cores: {world_size}")
            
        except Exception as e:
            print(f"⚠️  Could not get TPU details: {e}")
    else:
        print("⚠️  TPU not available, will try to initialize on training start")
        print("ℹ️  Make sure to set Runtime > Change runtime type > TPU")
    
    return bf16_support, fp16_support, precision_mode


def get_tpu_device():
    """
    Get XLA device for TPU.
    
    Returns:
        XLA device for TPU
    """
    try:
        import torch_xla.core.xla_model as xm
        return xm.xla_device()
    except ImportError:
        raise RuntimeError(
            "torch_xla not installed. Run:\n"
            "!pip install cloud-tpu-client torch torch_xla"
        )
    except Exception as e:
        raise RuntimeError(f"Failed to get TPU device: {e}")


def get_tpu_world_size() -> int:
    """Get number of TPU cores (replicas)."""
    try:
        import torch_xla.core.xla_model as xm
        return xm.xrt_world_size()
    except Exception:
        return 8  # Default TPU v2/v3 has 8 cores
