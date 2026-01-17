# Optimizations Applied - llama.cpp Engine

Dokumen ini menjelaskan optimisasi yang diterapkan pada SFCore AI Engine.

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  CLI (main.rs)                  │
│   - Parse arguments                             │
│   - Call engine                                 │
│   - Display output & metrics                    │
└─────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────┐
│              Engine (llama_engine.rs)           │
│   - LlamaCppOptions (config)                    │
│   - LlamaCppEngine (model, context)             │
│   - Sampler chain (penalties, top_p, temp)      │
└─────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────┐
│              llama-cpp-2 (Rust bindings)        │
│   - LlamaBackend                                │
│   - LlamaModel, LlamaContext                    │
│   - LlamaBatch, LlamaSampler                    │
└─────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────┐
│              llama.cpp (C++ library)            │
│   - GGUF loading                                │
│   - Quantized inference (Q4_K, Q6_K, etc)       │
│   - Flash Attention                             │
│   - SIMD optimizations (AVX2, etc)              │
└─────────────────────────────────────────────────┘
```

## Optimizations Applied

### 1. Backend Selection: llama.cpp vs Candle

| Aspect | Candle (removed) | llama.cpp |
|--------|------------------|-----------|
| Speed | ~2.7 tok/s | **17.6 tok/s** |
| FTL | 1344 ms | **93 ms** |
| Optimizations | Basic | AVX2, Flash Attention |
| Quantization | Basic GGUF | Full Q4_K_M support |

**Keputusan**: Hapus Candle, fokus 100% ke llama.cpp untuk performa maksimal.

### 2. Thread Configuration

```rust
// Default: 4 threads (optimal untuk 2-core + HT)
LlamaCppOptions {
    threads: Some(4),
    // ...
}
```

### 3. Batch Size Optimization

```rust
// Larger batch = better throughput
batch_size: 1024,  // dari 512
```

### 4. Sampler Chain (Anti-Repetition)

```rust
// Order penting: penalties → top_p → temp → dist
LlamaSampler::chain_simple([
    // 1. Repetition penalty (cegah loop)
    LlamaSampler::penalties(64, 1.1, 0.0, 0.0),
    // 2. Top-P nucleus sampling
    LlamaSampler::top_p(0.9, 1),
    // 3. Temperature (randomness)
    LlamaSampler::temp(0.6),
    // 4. Final sampling
    LlamaSampler::dist(seed),
])
```

### 5. Build Optimizations (.cargo/config.toml)

```toml
[profile.release]
codegen-units = 1    # Single codegen unit for better optimization
lto = "thin"         # Link-time optimization
opt-level = 3        # Maximum optimization
panic = "abort"      # Smaller binary

[build]
rustflags = ["-C", "target-cpu=x86-64-v2"]
```

## Performance Results

### Before Optimization (Candle)

```
[metrics] ftl: 1344 ms, tokens: 32, time: 14213 ms, speed: 2.25 tok/s
[memory] rss: 1978 MB
```

### After Optimization (llama.cpp)

```
[metrics] ftl: 93 ms, tokens: 64, time: 3729 ms, speed: 17.59 tok/s
[memory] rss: 541 MB
```

### Improvement Summary

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Speed | 2.7 tok/s | 17.6 tok/s | **+552%** |
| FTL | 1344 ms | 93 ms | **-93%** |
| Memory | 1978 MB | 541 MB | **-73%** |
| Build time | ~7 min | ~2 min | **-71%** |

## Configuration Options

### LlamaCppOptions

```rust
pub struct LlamaCppOptions {
    pub threads: Option<i32>,      // CPU threads (default: 4)
    pub context_length: u32,       // Context window (default: 2048)
    pub batch_size: usize,         // Batch size (default: 1024)
    pub seed: u32,                 // Random seed (default: 1234)
    pub temperature: f32,          // Randomness (default: 0.6)
    pub top_p: f32,                // Nucleus sampling (default: 0.9)
    pub repeat_penalty: f32,       // Anti-repetition (default: 1.1)
    pub repeat_last_n: i32,        // Tokens to check (default: 64)
}
```

## Tips untuk Performa Maksimal

1. **Gunakan quantization yang tepat**:
   - Q4_K_M: Balance antara speed dan quality
   - Q6_K: Lebih berkualitas, sedikit lebih lambat
   - Q8_0: Paling berkualitas, paling lambat

2. **Sesuaikan threads dengan CPU**:
   - 2-core CPU: threads = 2-4
   - 4-core CPU: threads = 4-8
   - 8-core CPU: threads = 8-16

3. **Turunkan temperature untuk output fokus**:
   - 0.3-0.5: Lebih deterministic (coding)
   - 0.6-0.8: Balance
   - 0.9-1.0: Lebih kreatif

4. **Gunakan model yang sesuai**:
   - Coding: Qwen2.5-Coder, Tessa-Rust
   - Reasoning: DeepSeek-R1-Distill
   - General: Llama-3, Mistral
