Yang kita lakukan murni **in‑process**: **Rust** memuat **model GGUF** langsung lewat binding **`llama-cpp-2`** (workspace `utilityai/llama-cpp-rs`) sehingga seluruh kernel `llama.cpp` berjalan di proses Rust Mas. Di bawah ini saya kasih **resep end‑to‑end** yang aman buat PC kentang (**Core i3 Gen‑6 (Skylake), RAM 16 GB**) serta cara memverifikasi bahwa **tidak ada proses server** di belakang layar.

***

## 1) Arsitektur yang dipakai (tanpa `llama-server`)

* **Rust app → crate `llama-cpp-2` → FFI C API `llama.cpp`** → load file **.gguf** → inferensi di **proses yang sama**.
* Build C++‑nya dikendalikan oleh **build‑script crate** (`llama-cpp-sys-2`) yang menjalankan **CMake** `llama.cpp`. Di sinilah kita mengaktifkan **ISA native** (`-march=native`) melalui **`GGML_NATIVE`** (feature `native` di crate), **terpisah** dari `RUSTFLAGS`. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* Parameter performa (threads, batch, ctx, mmap/mlock) disetel via **`llama.h` / context params**, dan diekspos oleh binding. (Nama field bisa beda tipis antar versi crate, tapi sumber kebenaran ada di header **`include/llama.h`** dan ringkasan param di dokumentasi konfigurasi.) [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html), [\[github.com\]](https://github.com/edgenai/llama_cpp-rs)

**Verifikasi “tanpa server”**: saat jalan, **tidak** ada proses `llama-server` di `ps aux`. Aplikasi hanya mem‑`dlopen`/link **`libllama`** (C API), bukan mem‐spawn proses lain. (Arsitektur server & flag‑flag seperti `--threads` dsb berada di `llama-server/README.md`, tapi kita **tidak** memakainya di jalur ini.) [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/19b03o2/using_promptcache_with_llamacpp/)

***

## 2) Setup *minimal* untuk Skylake i3 / 16 GB (CPU‑only)

> Fokus: **stabil + hemat memori**, cocok buat testing & tuning awal. BLAS bisa di-skip dulu (manfaatnya kecil di batch kecil CPU kentang).

### 2.1. `Cargo.toml` (pakai git workspace `utilityai/llama-cpp-rs`)

```toml
[package]
name = "rust-llm-standalone"
version = "0.1.0"
edition = "2021"

[dependencies]
# High-level safe wrapper + sys bindings dari utilityai
llama-cpp-2 = { git = "https://github.com/utilityai/llama-cpp-rs", package = "llama-cpp-2" }
# allocator opsional
tikv-jemallocator = "0.5"

[profile.release]
lto = "thin"
opt-level = 3
codegen-units = 1
panic = "abort"

[build]
rustflags = ["-C", "target-cpu=native"]
```

* `target-cpu=native` **hanya** mengoptimasi **kode Rust**. Untuk C/C++ `llama.cpp` kita aktifkan **feature `native`** (lihat langkah build di bawah) agar build‑script meneruskan **`GGML_NATIVE=ON`** ke CMake → `-march=native` pada backend CPU. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* Global allocator **jemalloc** (opsional; berguna untuk workload multithread). [\[unsloth.ai\]](https://unsloth.ai/docs/basics/inference-and-deployment/speculative-decoding), [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/1cs6u6n/does_llamacpps_speculative_actually_work/)

```rust
// src/main.rs
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    // ... kita isi di bagian §3 (contoh penggunaan)
}
```

### 2.2. Build (aktifkan CPU ISA **native** di sisi C/C++)

```bash
# feature `native` -> build.rs menyetel CMake define GGML_NATIVE=ON untuk llama.cpp
cargo build --release --features native
```

> Konfirmasi di log CMake dari crate sys: ia akan menyetel define untuk CMake. (Dokumentasi build‑script `llama-cpp-sys-2` menyebut **`GGML_NATIVE=ON`** sebagai bagian dari fitur `native`.) [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)

### 2.3. Model & parameter awal (biar tidak OOM di 16 GB)

* Pakai **model kecil** & kuantisasi **Q4\_K\_M** (atau lebih ringan): mis. 0.5–2B class, **bobot < 3 GB**. Kuantisasi K‑quants direkomendasikan untuk CPU; tool **`llama-quantize`** mendukung berbagai tipe (Q2\_K–Q6\_K, IQ…). [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/), [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h)
* Awali dengan **`n_ctx=2048`**, **`n_batch=1024`**, **`n_ubatch=256`**, **`n_threads=2`** (decode ≈ jumlah **physical core** pada i3‑6100 = 2), **mmap=true**, **mlock=false** (hemat RAM). Parameter‑parameter ini memang ada di C API `llama.h` / ringkasan konfigurasi. [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html), [\[github.com\]](https://github.com/edgenai/llama_cpp-rs)

> **Kenapa tidak BLAS dulu?** Di CPU kentang, *prefill batch* juga kecil → benefit OpenBLAS kecil, dan oversubscription berisiko. Nanti kalau sudah stabil dan ingin uji prefill panjang, kita bisa build dengan `-DGGML_BLAS=ON` dan **ingat** set `OPENBLAS_NUM_THREADS=1`. [\[deepwiki.com\]](https://deepwiki.com/utilityai/llama-cpp-rs/3.1.1-build-system-and-ffi-generation), [\[wandb.ai\]](https://wandb.ai/capecape/LLMs/reports/How-to-Run-LLMs-Locally-With-llama-cpp-and-GGML--Vmlldzo0Njg5NzMx)

***

## 3) Contoh kode **in‑process** (load model & tuning param)

> Catatan: Nama tipe/field bisa sedikit berbeda antar versi crate. Prinsipnya—semua mapping turunannya menuju field di **`llama_context_params`**/**`llama_model_params`** `llama.cpp`. Rujuk `include/llama.h` untuk kebenaran nama & arti param. [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)

```rust
use std::io::{self, Write};
// API contoh; sesuaikan dengan versi crate `llama-cpp-2` yang Anda pakai.
use llama_cpp_2::{
    LlamaModel, LlamaContext, // tipe utama
    ModelParams, ContextParams, SamplingParams,
};

fn main() -> anyhow::Result<()> {
    // === 1) Siapkan parameter load model (mmap/mlock dsb) ===
    let mut mparams = ModelParams::default();
    mparams.use_mmap = true;   // cepat start & hemat RSS
    mparams.use_mlock = false; // hemat RAM di 16 GB

    // Muat model GGUF kecil (Q4_K_M)
    let model = LlamaModel::load("/path/to/model.Q4_K_M.gguf", &mparams)?;

    // === 2) Siapkan parameter konteks (threads/batch/ctx) ===
    let mut cparams = ContextParams::default();
    cparams.n_ctx = 2048;       // panjang konteks
    cparams.n_batch = 1024;     // logical batch (prefill)
    cparams.n_ubatch = 256;     // physical batch
    cparams.n_threads = 2;      // decode: i3-6100 = 2 physical cores
    cparams.n_threads_batch = 2;// prefill (di CPU kentang, samakan dulu)

    let mut ctx = LlamaContext::new(&model, &cparams)?;

    // === 3) Prompt cache (opsional) untuk percepat run berikutnya ===
    // Jika crate mengekspos API state/session, panggil save/load state di sini.

    // === 4) Sampling params (bisa fine-tune sesuai kebutuhan) ===
    let sparams = SamplingParams {
        temp: 0.7,
        top_p: 0.95,
        ..Default::default()
    };

    // === 5) Prefill + decode streaming ===
    let prompt = "Tuliskan tiga tips efisien belajar pemrograman.";
    ctx.eval_prompt(prompt)?; // prefill sesuai n_batch/n_ubatch

    let max_tokens = 128usize;
    let mut written = 0usize;

    while written < max_tokens {
        if let Some(tok) = ctx.sample_token(&sparams)? {
            let text = ctx.token_to_str(tok);
            print!("{}", text);
            io::stdout().flush().ok();
            written += 1;

            // Autoregressive step: evaluasi token balik ke context
            ctx.eval_token(tok)?;
            if tok == ctx.token_eos() { break; }
        } else { break; }
    }

    Ok(())
}
```

**Di mana param‑param ini didefinisikan?**

* `n_ctx`, `n_batch`, `n_ubatch`, `n_threads`, `n_threads_batch`, `use_mmap`, `use_mlock` adalah bagian dari **context/model params** di C API `llama.cpp` (lihat **`include/llama.h`**) dan ringkasan **Configuration and Parameters** (menjelaskan efek ke performa & memori). [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html), [\[github.com\]](https://github.com/edgenai/llama_cpp-rs)

***

## 4) Preset awal khusus **i3 Gen‑6 / 16 GB**

1. **Model**: pilih < 3 GB (Q4\_K\_M) — ex: 0.5–2B. Kuantisasi K‑quants direkomendasikan. [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/), [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h)
2. **Context**: `n_ctx=2048`.
3. **Batch**: `n_batch=1024`, `n_ubatch=256`.
4. **Threads**: `n_threads=2`, `n_threads_batch=2`.
5. **Memory mapping**: `use_mmap=true`, `use_mlock=false`.
6. **Allocator**: jemalloc (opsional). [\[unsloth.ai\]](https://unsloth.ai/docs/basics/inference-and-deployment/speculative-decoding)
7. **BLAS**: off dulu; jika mau uji prefill panjang, build `llama.cpp` dengan **`-DGGML_BLAS=ON`** lalu set `OPENBLAS_NUM_THREADS=1`. [\[deepwiki.com\]](https://deepwiki.com/utilityai/llama-cpp-rs/3.1.1-build-system-and-ffi-generation), [\[wandb.ai\]](https://wandb.ai/capecape/LLMs/reports/How-to-Run-LLMs-Locally-With-llama-cpp-and-GGML--Vmlldzo0Njg5NzMx)

> **Tip diagnostik**: Cetak **tokens/s** untuk prefill vs decode. Jika prefill rendah, coba naikkan `n_threads_batch=3` (pakai hyper‑thread), kalau respon terasa “patah‑patah” turunkan lagi. (Skylake i3 cenderung lebih stabil di 2 thread.)

***

## 5) (Opsional) Build `llama.cpp` “portabel” atau *targeted*

* **Portabel** (tanpa `-march=native`) demi bisa dibawa lintas mesin:  
    set **`GGML_NATIVE=OFF`**, atau disable feature `native` pada crate, dan **beri `-march=<target>`** eksplisit via CMake flags (`znver1`, `x86-64-v3`, dst). Ada diskusi upstream soal default & reproducible builds yang menonaktifkan `native` di kondisi tertentu.

***

## 6) Cara membuktikan “tidak pakai server”

1. Jalankan app, lalu cek proses:

    ```bash
    ps aux | egrep 'llama-server|llama.cpp' | grep -v egrep
    ```

    Tidak boleh ada `llama-server`.
2. **Lihat link dinamis**:

    ```bash
    ldd target/release/rust-llm-standalone | grep -i llama
    ```

    Harus terlihat `libllama.so` (atau varian statik), bukan executable server.
3. Periksa **log CMake** di `target/release/build/*/out/` milik `llama-cpp-sys-2`: di sana terlihat define CMake yang dipakai (mis. `GGML_NATIVE=ON`). [\[elitedev.in\]](https://elitedev.in/rust/rusts-global-allocators-how-to-customize-memory-/)

***

### Referensi yang relevan

* **Build system `llama.cpp` (CMake), opsi BLAS & CPU build**. [\[deepwiki.com\]](https://deepwiki.com/utilityai/llama-cpp-rs/3.1.1-build-system-and-ffi-generation)
* **`llama-cpp-rs` (utilityai)** – build‑script menyalurkan fitur **`native → GGML_NATIVE=ON`** dan kompilasi `llama.cpp`. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* **C API `llama.cpp` (`include/llama.h`)** – sumber kebenaran untuk **context/model params** (n\_ctx, n\_batch, n\_ubatch, n\_threads, mmap/mlock, dst.). [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)
* **Ringkasan parameter (n\_ctx/n\_batch/n\_ubatch)** dan dampaknya terhadap performa/memori. [\[github.com\]](https://github.com/edgenai/llama_cpp-rs)
* **OpenBLAS runtime vars** – **`OPENBLAS_NUM_THREADS=1`** untuk menghindari oversubscription. [\[wandb.ai\]](https://wandb.ai/capecape/LLMs/reports/How-to-Run-LLMs-Locally-With-llama-cpp-and-GGML--Vmlldzo0Njg5NzMx)
* **Kuantisasi & `llama-quantize` (K‑quants)** – Q4\_K\_M/Q5\_K\_M disarankan untuk CPU. [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/), [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h)
