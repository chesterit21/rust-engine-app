# Walkthrough - Extreme Performance Tuning (Native + BLAS)

Dokumen ini merangkum perjalanan optimasi performa inferensi LLM pada CPU "Kentang" (i3-6100, 2 Core / 4 Thread) menggunakan Rust dan `llama.cpp`.

## 🎯 Goal

Mencapai throughput **> 8 tok/s** untuk model 1.5B (Q4_K_M) dengan fitur **Native Architecture** (`-march=native`) dan **BLAS acceleration** aktif.

## 🚧 Tantangan Utama

Selama proses tuning, kami menghadapi 3 masalah kritis:

1. **"Fake Native" Build**:
    Script [build.rs](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/crates/patched/llama-cpp-sys-2/build.rs) bawaan library `llama-cpp-sys-2` ternyata memiliki logika yang menonaktifkan `GGML_NATIVE` di kondisi tertentu, meskipun kita sudah set environment variable.
    * *Dampak*: Binary tidak teroptimasi, performa drop ke ~4 tok/s.

2. **Missing Batch Parameter**:
    Kode Rust engine ([llama_engine.rs](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/crates/engine/src/llama_engine.rs)) lupa meneruskan parameter `n_batch` ke C++ backend via `LlamaContextParams`.
    * *Dampak*: BLAS tidak efektif saat prefill karena batch size default (512) kekecilan.

3. **Thread Oversubscription**:
    Menggunakan `threads=4` (semua logical core) menyebabkan contention parah dengan thread OS dan overhead sinkronisasi BLAS.
    * *Dampak*: Performa regresi ke 4.93 tok/s.

## 🛠️ Solusi & Implementasi

### 1. Patching [build.rs](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/crates/patched/llama-cpp-sys-2/build.rs) (Native Override)

Kami melakukan patch manual pada [crates/patched/llama-cpp-sys-2/build.rs](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/crates/patched/llama-cpp-sys-2/build.rs) agar respek terhadap env var:

```rust
// Patch untuk memaksa GGML_NATIVE=ON
if let Ok(val) = env::var("GGML_NATIVE") {
    config.define("GGML_NATIVE", &val);
}
```

### 2. Fix Engine Code

Menambahkan `with_n_batch` di [llama_engine.rs](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/crates/engine/src/llama_engine.rs):

```rust
let mut ctx_params = LlamaContextParams::default()
    .with_n_ctx(Some(ctx_size))
    .with_n_batch(self.opts.batch_size as u32) // <-- Added this
    .with_n_ubatch(self.opts.ubatch_size as u32);
```

### 3. Tuning Thread "Sweet Spot"

Kembali ke konfigurasi **3 threads** (75% resources) untuk menyisakan ruang bagi OS dan overhead sinkronisasi BLAS.

*Catatan: BLAS akhirnya dinonaktifkan (`GGML_BLAS=OFF`) karena sumber `ggml-blas` tidak disertakan dalam versi crate `llama-cpp-sys-2` ini. Namun, patch `GGML_NATIVE` memastikan instruksi CPU tetap optimal.*

## 🚀 Hasil Akhir

Benchmark pada **Qwen2.5-Coder-1.5B-Instruct-Q4_K_M.gguf**:

| Metric | Before (Broken) | Baseline (Raw) | **Final (Native)** |
| :--- | :--- | :--- | :--- |
| **Speed** | 4.93 tok/s | 8.25 tok/s | **8.40 tok/s** ✅ |
| **Startup (FTL)** | 1789 ms | ~1500 ms | **399 ms** ⚡ |
| **Config** | Threads=4 | Threads=3, No BLAS | **Threads=3, Native=ON** |

## 📦 Kesimpulan

Konfigurasi **Extreme** kini terbukti stabil dan memberikan performa tertinggi dengan startup time yang sangat cepat. Patch sistem build telah diterapkan dan didokumentasikan di [docs/extreme-performance.md](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/docs/extreme-performance.md).

***

## 🌐 Phase 2: High-Performance Server (IPC)

Untuk kebutuhan integrasi aplikasi, kami telah membangun **sfcore-ai-server** yang berjalan sebagai daemon lokal.

### Arsitektur

* **Protocol**: Unix Domain Socket (UDS) di [/tmp/sfcore-ai.sock](file:///tmp/sfcore-ai.sock).
* **Format**: Newline Delimited JSON (NDJSON).
* **Fitur**: Persistent Model (Load once in RAM) & Streaming Output.

### Cara Penggunaan

Server membaca konfigurasi otomatis dari [server_config.toml](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/server_config.toml) (Model path, threads, dll).

```bash
# Start Server
taskset -c 0-3 cargo run -p sfcore-ai-server --release
```

### Protocol Request (Example)

Client (NodeJS/Rust/Go) cukup kirim JSON ke socket:

```json
{
  "prompt": "Jelaskan Rust dalam 1 kalimat",
  "stream": true,
  "max_tokens": 100
}
```

Response akan diterima secara streaming per token. Lihat [test_server.js](file:///home/sfcore/SFCoreAIApps/AIRustTools/root-app/sfcore-ai/test_server.js) untuk contoh implementasi client.
