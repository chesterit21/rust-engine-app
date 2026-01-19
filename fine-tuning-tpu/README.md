# 🚀 Fine-Tuning Gemma 3 270M on Google Colab TPU

Production-grade fine-tuning system using **PyTorch XLA + LoRA** for TPU training.

## 📋 Requirements

- Google Colab with TPU runtime
- Dataset in JSONL format with `messages` field

## 🗂️ Project Structure

```text
fine-tuning-tpu/
├── requirements.txt          # Dependencies
├── config/
│   ├── model_configs.yaml    # Model configurations
│   └── training_configs.yaml # Training templates
├── src/
│   ├── data/                 # Dataset processing
│   ├── models/               # LoRA configuration
│   ├── training/             # TPU-optimized callbacks, metrics
│   └── utils/                # Utilities
├── scripts/
│   ├── train.py              # Main TPU training script
│   └── test_model.py         # Test set evaluation
├── docs/
│   └── guide.md              # TPU optimization guide
└── outputs/                  # Training outputs
```

## 🚀 Quick Start

### 1. Setup Colab TPU

1. Open notebook in Google Colab
2. Go to **Runtime > Change runtime type**
3. Select **TPU** as Hardware accelerator
4. Click **Save**

### 2. Install Dependencies

```python
# Install PyTorch XLA for TPU
!pip install -q cloud-tpu-client==0.10 torch==2.0.0 \
    https://storage.googleapis.com/pytorch-xla-releases/wheels/tpuvm/torch_xla-2.0-cp310-cp310-linux_x86_64.whl

# Install other dependencies
!pip install -q transformers accelerate peft trl datasets sentencepiece
```

### 3. Verify TPU

```python
import torch_xla.core.xla_model as xm
device = xm.xla_device()
print(f"TPU Device: {device}")
print(f"TPU Cores: {xm.xrt_world_size()}")
```

### 4. Prepare Dataset

Create JSONL file with `messages` field:

```json
{"messages": [{"role": "user", "content": "Hello"}, {"role": "assistant", "content": "Hi!"}]}
```

### 5. Run Training

```bash
python scripts/train.py --dataset path/to/data.jsonl
```

## ⚙️ TPU-Specific Optimizations

### Precision

- TPU uses **bfloat16** (NOT fp16!)
- BFloat16 gives 4-47% faster performance

### Batch Size

- **Must be multiples of 128** for optimal TPU efficiency
- Global batch = `batch_size × 8 cores × gradient_accum`
- Default: 128 per replica

### Logging

- Reduced logging frequency (every 50 steps)
- Uses `xm.add_step_closure()` for async logging
- Avoids host-device sync overhead

### Checkpointing

- Saves to Google Drive
- Uses `xm.is_master_ordinal()` to avoid conflicts

## 📊 Configuration

### Model: google/gemma-3-270m-it

- **LoRA Rank (r)**: 8
- **LoRA Alpha**: 16
- **Target Modules**: q_proj, k_proj, v_proj, o_proj, gate_proj, up_proj, down_proj

### Training Settings (TPU Optimized)

| Parameter | Value |
| --------- | ----- |
| Precision | BFloat16 (TPU native) |
| Batch Size | 128 per replica |
| TPU Cores | 8 |
| Gradient Accumulation | Dynamic |
| Learning Rate | 2e-5 (cosine) |
| Logging Frequency | Every 50 steps |
| Eval Frequency | Every 200 steps |

## ⚠️ Key Differences from GPU Version

| Aspect | GPU | TPU |
| ------ | --- | --- |
| Precision | FP16 | BFloat16 |
| Batch Size | Flexible | Multiples of 128 |
| Execution | Eager | Lazy (graph) |
| Gradient Sync | Implicit | `xm.optimizer_step()` |
| Quantization | 4-bit QLoRA | Standard LoRA |
| Logging | Frequent OK | Reduce frequency |

## 📚 References

- [Cloud TPU Performance Guide](https://docs.cloud.google.com/tpu/docs/performance-guide)
- [PyTorch XLA Documentation](https://pytorch.org/xla/release/r2.8/learn/xla-overview.html)
- [BFloat16 on Cloud TPUs](https://cloud.google.com/blog/products/ai-machine-learning/bfloat16-the-secret-to-high-performance-on-cloud-tpus)
- [Gemma 3 Model Card](https://huggingface.co/google/gemma-3-270m-it)
