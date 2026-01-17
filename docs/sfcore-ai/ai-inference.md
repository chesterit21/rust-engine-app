# SFCore AI Engine - Optimized LLM Inference

Engine inference LLM berbasis **llama.cpp** dengan Rust bindings untuk performa tinggi.

## Prerequisites

### System Dependencies

### 1.1. CMake dasar

```bash
# Install C++ toolchain untuk compile llama.cpp tanpa fitur BLAST
sudo apt-get install -y libclang-dev cmake clang build-essential
```

### 1.1. CMake dasar + BLAS

```bash
cmake -B build \
  -DCMAKE_BUILD_TYPE=Release \
  -DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS
cmake --build build --config Release -j"$(nproc)"
```

### Rust Dependencies

Pastikan Rust sudah terinstall:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Project Structure

```
sfcore-ai/
├── Cargo.toml
├── .cargo/
│   └── config.toml          # Release optimizations
├── crates/
│   ├── engine/              # Core inference engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── llama_engine.rs
│   │   └── Cargo.toml
│   └── cli/                 # Command-line interface
│       ├── src/main.rs
│       └── Cargo.toml
└── models/                  # Place GGUF models here
```

## Installation

### 1. Clone & Build

```bash
cd /home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai

# Build release (first build takes ~5-10 minutes to compile llama.cpp)
cargo build --release
```

### 2. Download Model GGUF

```bash
# Contoh: Qwen2.5-Coder-0.5B-Instruct
curl -L "https://huggingface.co/Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-0.5b-instruct-q6_k.gguf" \
  -o ./models/QwenCoder.gguf
```

## Usage

### Basic Usage

```bash
cargo run -p sfcore-ai-cli --release -- \
  --model ./models/QwenCoder.gguf \
  --prompt "create function calculate_sum in Rust" \
  --max-tokens 128
```

### Advanced Performance Tuning

**1. CPU Affinity (Taskset)**
Gunakan `taskset` untuk mengunci proses ke core tertentu (misal: P-cores saja).

```bash
# Gunakan core 0-3 (4 threads)
taskset -c 0-3 cargo run -p sfcore-ai-cli --release -- \
  --model ./models/QwenCoder.gguf \
  --threads 4 \
  --threads-batch 4
```

**2. Optimize Memory (mlock)**
Kunci model di RAM agar tidak kena swap.

```bash
cargo run -p sfcore-ai-cli --release -- \
  --model ./models/QwenCoder.gguf \
  --mlock
```

**3. Optimize Batching**
Untuk throughput prefill (prompt processing) lebih tinggi:

```bash
cargo run -p sfcore-ai-cli --release -- \
  --model ./models/QwenCoder.gguf \
  --batch-size 2048 \
  --ubatch-size 512
```

### All Options

| Option | Default | Description |
|--------|---------|-------------|
| `--model` | (required) | Path ke file GGUF |
| `--threads` | 3 | Decode threads |
| `--threads-batch` | 3 | Prefill threads |
| `--temperature` | 0.5 | Randomness (0.0-2.0) |
| `--top-k` | 40 | Top-K sampling |
| `--top-p` | 0.9 | Nucleus sampling |
| `--min-p` | 0.05 | Min-P filtering |
| `--repeat-penalty` | 1.0 | Repetition penalty (1.1=on) |
| `--mlock` | false | Lock memory in RAM |

## Performance Benchmarks

Tested on Intel Core i3-6100 (4 threads):

| Model | Settings | Speed | FTL | Memory |
|-------|----------|-------|-----|--------|
| Qwen2.5-Coder-0.5B | Default | 11.8 tok/s | 300ms | 544MB |
| Qwen2.5-Coder-0.5B | Optimized* | **17.6 tok/s** | 93ms | 541MB |

*Optimized: threads=4, threads-batch=4, batch-size=1024, mlock=true
