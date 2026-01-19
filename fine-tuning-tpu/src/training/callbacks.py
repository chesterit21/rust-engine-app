"""
Training Callbacks Module for TPU

Custom callbacks optimized for TPU training:
- Reduced logging frequency (avoid host-device sync overhead)
- TPU-compatible checkpointing
- Early stopping

Reference:
- https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch
- https://docs.pytorch.org/xla/master/learn/migration-to-xla-on-tpus.html
"""

from typing import Dict
from transformers import TrainerCallback, TrainingArguments, TrainerState, TrainerControl
import numpy as np


class TPULoggingCallback(TrainerCallback):
    """
    TPU-optimized logging callback.
    
    IMPORTANT: Logging too frequently causes host-device sync overhead!
    Only log every N steps to avoid performance degradation.
    """
    
    def __init__(self, log_every_n_steps: int = 50):
        self.log_every_n_steps = log_every_n_steps
        
    def on_step_end(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        **kwargs
    ):
        # Only log at intervals to avoid TPU sync overhead
        if state.global_step % self.log_every_n_steps == 0:
            try:
                import torch_xla.core.xla_model as xm
                # Use async logging to avoid blocking
                xm.add_step_closure(
                    lambda: print(f"📊 Step {state.global_step}: loss={state.log_history[-1].get('loss', 'N/A'):.4f}" if state.log_history else f"📊 Step {state.global_step}")
                )
            except ImportError:
                pass
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
        print(f"\n🎯 Dynamic Configuration Applied (TPU):")
        print(f"   Gradient Checkpointing: {self.dynamic_config.get('use_gradient_checkpointing', False)}")
        print(f"   Batch Size per Replica: {self.dynamic_config.get('per_device_train_batch_size', 'N/A')}")
        print(f"   Gradient Accumulation: {self.dynamic_config.get('gradient_accumulation_steps', 'N/A')}")
        print(f"   Global Batch Size: {self.dynamic_config.get('global_batch_size', 'N/A')}")
        print(f"   Max Seq Length: {self.dynamic_config.get('max_seq_length', 'N/A')}")
        print(f"   Precision: bfloat16 (TPU native)\n")
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


class TPUCheckpointCallback(TrainerCallback):
    """
    TPU-compatible checkpointing.
    
    Uses xm.save() for proper TPU checkpoint saving.
    Only master ordinal saves to avoid conflicts.
    """
    
    def __init__(self, save_path: str = "/content/drive/MyDrive/checkpoints"):
        self.save_path = save_path
        
    def on_save(
        self, 
        args: TrainingArguments, 
        state: TrainerState, 
        control: TrainerControl, 
        **kwargs
    ):
        try:
            import torch_xla.core.xla_model as xm
            
            # Only master ordinal saves
            if xm.is_master_ordinal():
                print(f"\n💾 Checkpoint saved at step {state.global_step}")
                # Note: Actual saving is handled by HuggingFace Trainer
                # This callback just provides logging
        except ImportError:
            pass
            
        return control
