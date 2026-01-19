# 🚀 Fine-Tuning Qwen3-0.6B on Google Colab T4

Production-grade fine-tuning system using **Unsloth + QLoRA** for efficient training.

## 📋 Requirements

- Google Colab with T4 GPU (16GB VRAM)
- Dataset in JSONL format with `text` field

## 🗂️ Project Structure

```
fine-tuning-project/
├── requirements.txt          # Dependencies
├── config/
│   ├── model_configs.yaml    # Model configurations
│   └── training_configs.yaml # Training templates
├── src/
│   ├── data/                 # Dataset processing
│   ├── models/               # LoRA configuration
│   ├── training/             # Callbacks, metrics
│   └── utils/                # Utilities
├── scripts/
│   ├── train.py              # Main training script
│   └── test_model.py         # Test set evaluation
├── notebooks/
│   └── colab_training.ipynb  # Colab notebook
└── outputs/                  # Training outputs
```

## 🚀 Quick Start

### 1. Upload to Colab

Upload this folder to Google Drive and mount in Colab.

### 2. Install Dependencies

```bash
!pip install -q torch transformers accelerate bitsandbytes peft trl datasets
!pip install -q "unsloth[colab-new] @ git+https://github.com/unslothai/unsloth.git"
```

### 3. Prepare Dataset

Create JSONL file with `text` field:

```json
{"text": "<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant\nHi!<|im_end|>"}
{"text": "<|im_start|>user\nWhat is Python?<|im_end|>\n<|im_start|>assistant\nPython is a programming language.<|im_end|>"}
```

### 4. Run Training

```bash
python scripts/train.py --dataset path/to/data.jsonl --use_unsloth
```

### 5. Evaluate on Test Set

```bash
python scripts/test_model.py --model ./outputs/final_model --dataset ./outputs/test_dataset.json
```

## ⚙️ Configuration

### Model: Qwen/Qwen3-0.6B

- **LoRA Rank (r)**: 8
- **LoRA Alpha**: 16
- **Target Modules**: q_proj, k_proj, v_proj, o_proj, gate_proj, up_proj, down_proj

### Training Settings

- **Batch Size**: Dynamic based on dataset
- **Gradient Accumulation**: Dynamic
- **Learning Rate**: 2e-5 with cosine schedule
- **Mixed Precision**: FP16 (T4 fallback)
- **Early Stopping**: Patience 5

## 📊 Expected Results

| Metric | Typical Range |
|--------|---------------|
| Training Loss | 1.0 - 2.0 |
| Validation Perplexity | 10 - 30 |
| VRAM Usage | 10 - 14 GB |
| Training Time (10K samples) | ~1-2 hours |

## 📚 References

- [Unsloth Documentation](https://github.com/unslothai/unsloth)
- [Qwen3 Model Card](https://huggingface.co/Qwen/Qwen3-0.6B)
- [TRL SFTTrainer](https://huggingface.co/docs/trl/sft_trainer)
