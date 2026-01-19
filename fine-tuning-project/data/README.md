# Dataset Folder

Simpan dataset JSONL di folder ini.

## Format Dataset

File: `train_data.jsonl`

```json
{"text": "<|im_start|>user\nHello<|im_end|>\n<|im_start|>assistant\nHi!<|im_end|>"}
{"text": "<|im_start|>user\nApa itu Python?<|im_end|>\n<|im_start|>assistant\nPython adalah bahasa pemrograman.<|im_end|>"}
```

## Usage

```bash
python scripts/train.py --dataset data/train_data.jsonl
```
