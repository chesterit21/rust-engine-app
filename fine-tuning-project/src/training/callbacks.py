"""
Training Callbacks Module

Custom callbacks for VRAM monitoring, early stopping, and logging.

Reference:
- https://huggingface.co/docs/transformers/main_classes/callback
- https://www.runpod.io/articles/guides/avoid-oom-crashes-for-large-models
"""

import torch
import gc
from typing import Dict
from transformers import TrainerCallback, TrainingArguments, TrainerState, TrainerControl
import numpy as np

# Try to import pynvml for VRAM monitoring
try:
    import pynvml
    pynvml.nvmlInit()
    NVML_AVAILABLE = True
except:
    NVML_AVAILABLE = False
    print("⚠️  pynvml not available, VRAM monitoring disabled")


class VRAMMonitorCallback(TrainerCallback):
    """
    Monitor VRAM usage and trigger early warning if approaching OOM.
    
    Args:
        threshold_percent: VRAM usage percentage to trigger warning
    """
    def __init__(self, threshold_percent: float = 95.0):
        self.threshold_percent = threshold_percent
        self.oom_warning_shown = False
        
    def on_step_end(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        **kwargs
    ):
        if not NVML_AVAILABLE or not torch.cuda.is_available():
            return control
        
        try:
            handle = pynvml.nvmlDeviceGetHandleByIndex(0)
            mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
            used_percent = (mem_info.used / mem_info.total) * 100
            
            # Log every 50 steps
            if state.global_step % 50 == 0:
                print(f"💾 VRAM: {mem_info.used / 1e9:.2f}GB / {mem_info.total / 1e9:.2f}GB ({used_percent:.1f}%)")
            
            # OOM Prevention
            if used_percent > self.threshold_percent and not self.oom_warning_shown:
                print(f"\n⚠️  CRITICAL: VRAM usage at {used_percent:.1f}%!")
                print("   Clearing cache to prevent OOM...")
                gc.collect()
                torch.cuda.empty_cache()
                self.oom_warning_shown = True
                
            # Reset warning after VRAM drops
            if used_percent < self.threshold_percent - 5:
                self.oom_warning_shown = False
                
        except Exception as e:
            pass  # Silence errors to not interrupt training
            
        return control


class DynamicConfigCallback(TrainerCallback):
    """Log dynamic configuration at training start."""
    
    def __init__(self, dynamic_config: dict):
        self.dynamic_config = dynamic_config
    
    def on_train_begin(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        **kwargs
    ):
        print(f"\n🎯 Dynamic Configuration Applied:")
        print(f"   Gradient Checkpointing: {self.dynamic_config.get('use_gradient_checkpointing', False)}")
        print(f"   Batch Size: {self.dynamic_config.get('per_device_train_batch_size', 'N/A')}")
        print(f"   Gradient Accumulation: {self.dynamic_config.get('gradient_accumulation_steps', 'N/A')}")
        print(f"   Effective Batch Size: {self.dynamic_config.get('effective_batch_size', 'N/A')}")
        print(f"   Max Seq Length: {self.dynamic_config.get('max_seq_length', 'N/A')}\n")
        return control


class EarlyStoppingCallback(TrainerCallback):
    """
    Early stopping based on validation loss to prevent overfitting.
    
    Args:
        patience: Number of evaluations without improvement before stopping
        min_delta: Minimum change in loss to be considered improvement
    """
    def __init__(self, patience: int = 3, min_delta: float = 0.001):
        self.patience = patience
        self.min_delta = min_delta
        self.best_loss = float('inf')
        self.counter = 0
        
    def on_evaluate(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        metrics: Dict[str, float], 
        **kwargs
    ):
        current_loss = metrics.get("eval_loss")
        
        if current_loss is None:
            return control
        
        # Check improvement
        if current_loss < self.best_loss - self.min_delta:
            self.best_loss = current_loss
            self.counter = 0
            print(f"\n✅ New best validation loss: {current_loss:.4f}")
        else:
            self.counter += 1
            print(f"\n⚠️  No improvement in validation loss ({self.counter}/{self.patience})")
            
            if self.counter >= self.patience:
                print(f"\n🛑 Early stopping triggered! Best loss: {self.best_loss:.4f}")
                control.should_training_stop = True
        
        return control


class ValidationLossLoggerCallback(TrainerCallback):
    """Log validation loss and perplexity at each evaluation."""
    
    def __init__(self):
        self.validation_losses = []
        self.validation_perplexities = []
        
    def on_evaluate(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        metrics: Dict[str, float], 
        **kwargs
    ):
        val_loss = metrics.get("eval_loss")
        val_ppl = metrics.get("eval_perplexity")
        
        if val_ppl is None and val_loss is not None:
            val_ppl = np.exp(val_loss)
        
        if val_loss is not None:
            self.validation_losses.append(val_loss)
            
        if val_ppl is not None:
            self.validation_perplexities.append(val_ppl)
            
        print(f"\n📈 Validation Metrics (Step {state.global_step}):")
        print(f"   Loss: {val_loss:.4f}" if val_loss else "   Loss: N/A")
        print(f"   Perplexity: {val_ppl:.4f}" if val_ppl else "   Perplexity: N/A")
        
        return control
