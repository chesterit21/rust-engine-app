Kita masuk ke **mode Extreme**: konfigurasi **Rust + llama.cpp** CPU‑only yang benar‑benar “out of the box”—menggali sampai ke build toolchain, allocator, BLAS, threading, affinity, batching, caching, kuantisasi, sampai PGO/BOLT. Semua poin di bawah **punya rujukan** agar bisa ditelusuri saat debugging.

> **Tujuan**: throughput & latency **maksimal** di **single‑CPU** (tanpa GPU), stabil, reproducible, serta siap dioperasionalkan di Agent/Tool MCP.

***

## 0) Prinsip arsitektur (kenapa ini “Extreme”)

1. **Pisahkan prefill vs decode**: prefill (matmul batch besar) → **BLAS** + banyak thread; decode (1 token/step) → **lebih sensitif** ke *latency* dan cache CPU → thread lebih sedikit & *affinity* ketat. `llama-server` bahkan menyediakan `--threads` (decode) vs `--threads-batch` (prefill) untuk ini. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
2. **Hindari oversubscription**: kalau sudah multi‑thread di `llama.cpp`, **batasi** thread BLAS (OpenBLAS/MKL) ke 1 supaya tidak terjadi *thread cascades*. [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)
3. **Parallelism lewat multi‑context**: untuk throughput CPU tinggi, lebih efisien jalankan beberapa **context** paralel (masing‑masing dipin ke subset core) ketimbang “memaksakan” satu context dengan thread raksasa. Arsitektur `llama-server` (slot, queue, continuous batching) dijadikan acuan. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/5.2-http-server), [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

***

## 1) Build “gila” untuk **llama.cpp** (CMake)

> **Wajib** pakai CMake (Makefile sudah deprecated). Jalankan **Release** dan aktifkan fitur CPU yang relevan.

### 1.1. CMake dasar + BLAS

```bash
cmake -B build \
  -DCMAKE_BUILD_TYPE=Release \
  -DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS
cmake --build build --config Release -j"$(nproc)"
```

* CMake adalah jalur resmi; `GGML_BLAS`/`GGML_BLAS_VENDOR` mengaktifkan BLAS (OpenBLAS/MKL). **Peningkatan prefill** sering signifikan di CPU. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

**Runtime wajib**:

```bash
export OPENBLAS_NUM_THREADS=1    # atau MKL_NUM_THREADS=1 bila pakai MKL
```

(Resmi: variabel runtime OpenBLAS) [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)

### 1.2. Native ISA (SIMD) & toolchain

> **Distribusi** lintas mesin? Build **khusus target** (mis. x86‑64‑v3) bisa dilakukan untuk project Rust; gagasan sama berlaku—lihat panduan optimasi target CPU di Rust Dev Guide. [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)

### 1.3. Critical Fix: "Fake Native" Patch

**PERINGATAN**: Crate `llama-cpp-sys-2` bawaan (v0.1.x) memiliki logika di `build.rs` yang secara default **mematikan** `GGML_NATIVE` jika mendeteksi cross-compilation atau kondisi tertentu, bahkan jika kita set env var.

Solusi: **Patch manual** `build.rs` untuk menerima override:

```rust
// build.rs (patched)
// ...
    if target_cpu == Some("native".into()) {
        debug_log!("Detected target-cpu=native, compiling with GGML_NATIVE");
        config.define("GGML_NATIVE", "ON");
    } else if let Ok(val) = env::var("GGML_NATIVE") {
        // PATCH: Allow GGML_NATIVE override via env var
        debug_log!("Detected GGML_NATIVE override: {}", val);
        config.define("GGML_NATIVE", &val);
    }
// ...
```

Tanpa patch ini, flag `export GGML_NATIVE=ON` **TIDAK AKAN BERFUNGSI**.

> **Catatan BLAS**: Pada versi crate `llama-cpp-sys-2` v0.1.132, direktori sumber `ggml-blas` tidak disertakan, sehingga `GGML_BLAS=ON` akan gagal build. Fokuskan optimasi pada `GGML_NATIVE` dan tuning thread.

***

## 2) **Rust binary** “no mercy”: Release, LTO, target CPU, allocator

### 2.1. Profil rilis & flags

Di `Cargo.toml` / `.cargo/config.toml`:

```toml
[profile.release]
opt-level = 3     # speed
lto = "thin"      # atau "fat" untuk perf maksimal (build lebih lama)
codegen-units = 1
panic = "abort"

[build]
rustflags = ["-C", "target-cpu=native"]
```

* **Release build** vs dev: perbedaan bisa **10–100×**; `target-cpu=native` mengaktifkan instruksi CPU spesifik perangkat. **LTO** membantu cross‑crate optimizations. [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html), [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)

### 2.2. Global allocator (jemalloc)

```toml
[dependencies]
tikv-jemallocator = "0.5"
```

```rust
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

* **jemalloc** sering lebih stabil untuk workload multithread jangka panjang & fragmentasi kecil; mudah diaktifkan via global allocator. [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/index.html)

> **Eksperimen**: bandingkan juga **mimalloc** jika ada pola alokasi kecil/sangat sering—mekanismenya setara via `#[global_allocator]` (lihat dokumentasi global allocator Rust). [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/index.html)

***

## 3) **Threading & Affinity** (prefill vs decode)

### 3.1. Atur thread per fase

* `--threads` → **decode**.
* `--threads-batch` → **prefill/batch**.  
    Resep awal single‑CPU modern:
* `--threads-batch` ≈ **physical cores** (bukan logical).
* `--threads` ≈ **50–70%** physical cores (decode kurang paralel).  
    Lalu **ukur** & koreksi. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

### 3.2. CPU pinning

* Gunakan `--cpu-mask` (hex) / `--cpu-range` untuk pin thread. Afinitas **mengurangi migrasi** dan menstabilkan latensi; arsitektur thread‑pool CPU backend dijelaskan baik di panduan Arm. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)

***

## 4) **Batching** (n\_batch, n\_ubatch) & memory

* **`n_batch` (logical)** → memperbesar *throughput* prefill (waspadai RAM).
* **`n_ubatch` (physical)** → ukuran “sub‑batch” nyata; sesuaikan dengan L2/L3 agar tidak *cache thrash*.  
    Default umum: `n_batch=2048`, `n_ubatch=512`. Tuning parameter ini dibahas eksplisit pada ringkasan konfigurasi. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.3-configuration-and-parameters)

***

## 5) **Manajemen memori model** (mmap/mlock) & KV cache

### 5.1. Model mmap/mlock

* **`use_mmap=true`** → OS memory‑map, **cepat start** & hemat RSS.
* **`use_mlock=true`** → kunci ke RAM agar **anti page‑out** (aktifkan hanya bila RAM aman).  
    Field ini tersedia di `llama_context_params` dan di‑expose berbagai binding (Node/.NET). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h), [\[node-llama...withcat.ai\]](https://node-llama-cpp.withcat.ai/api/type-aliases/LlamaModelOptions), [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/)

### 5.2. Prompt cache / session state

* CLI: `--prompt-cache` + `--prompt-cache-ro` mempercepat run ulang prompt besar (menyimpan KV). **Wajib** untuk beban kerja RAG/“big primer”. [\[github.com\]](https://github.com/ggml-org/llama.cpp/discussions/2110)

### 5.3. (Eksperimental) **KV cache quantization**

* Mengurangi jejak KV agar long‑context lebih feasible; hasil komunitas menunjukkan penghematan besar memori—cocok saat CPU bandwidth memori jadi bottleneck (uji dampak ke t/s). [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/1dalkm8/memory_tests_using_llamacpp_kv_cache_quantization/)

***

## 6) **Kuantisasi model** (K‑quants/IQ)

* **Sweet spot CPU**: **Q4\_K\_M** (umum) → keseimbangan kualitas/ukuran/kecepatan; **Q5\_K\_M** bila butuh akurasi lebih. Gunakan `llama-quantize`; pedoman resmi & daftar tipe tersedia. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/6.3-model-quantization)

> Model lebih kecil → lebih **ramah cache** dan decode sering **lebih cepat** di CPU. Uji A/B Q4\_K\_M vs Q5\_K\_M di beban Mas.

***

## 7) **Fitur decoding lanjutan** (opsional, “Extreme”)

### 7.1. **Flash Attention** (`-fa`)

* Aktifkan jika path CPU mendukung; membantu prompt panjang & memori kerja—flag tersedia di CLI/server. **Ukur** karena *impact* bervariasi antar CPU. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [\[manpages.debian.org\]](https://manpages.debian.org/testing/llama.cpp-examples/llama-parallel.1.en.html)

### 7.2. **Speculative decoding**

* Gunakan **draft model** kecil untuk *speculate* beberapa token, diverifikasi paralel oleh model utama. Efektif bila **acceptance** tinggi & *draft* jauh lebih kecil; ada contoh resmi `examples/speculative`. Pada CPU‑only, speed‑up sangat **prompts‑dependent**. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/examples/speculative/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/7.2-speculative-decoding)

***

## 8) **HTTP server vs Rust binding** (arsitektur layanan)

Kalau targetnya *many‑requests* (multi‑user/agent), pertimbangkan **meniru arsitektur `llama-server`** (slot queue, continuous batching, parallel decoding). Bisa **integrasi dari Rust** dengan:

* a) Binding langsung (pool `llama_context` paralel + pinning), atau
* b) **Endpoint HTTP** internal (OpenAI‑compatible) lalu *reverse proxy*.  
    Dokumentasi server & arsitekturnya tersedia lengkap untuk jadi *blueprint*. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/5.2-http-server)

***

## 9) **Rust‑level Extreme**: PGO & BOLT (profil‑driven)

* **PGO (Profile‑Guided Optimization)** & **BOLT** (post‑link optimizer) dapat menambah performa double‑digit bila *hot path* jelas.  
    **Strategi**:
    1. Build Rust app (dan kalau memungkinkan, shared `libllama`) dengan **instrumen PGO**,
    2. Jalankan *representative workload* (prefill & decode campur),
    3. Rebuild dengan profil,
    4. Terapkan **BOLT** pada binari final.  
        Pedoman resmi Rust Dev Guide untuk PGO/BOLT (meski bahasannya pada `rustc`, prinsip & tooling sama). [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)

> Catatan: PGO untuk `llama.cpp` sendiri belum ada panduan resmi; lakukan PGO di **binary Rust** (yang memanggil `libllama`) dan **ukur**. Referensi tetap Rust Dev Guide. [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)

***

## 10) **Profiling & benchmarking** (jangan terbang buta)

* **`llama-bench`** untuk baseline tiap kombinasi flag/model/threads. [\[github.com\]](https://github.com/ggml-org/llama.cpp)
* Analisis **threading & affinity** CPU: gunakan perf/**Arm Streamline**—panduan memperlihatkan cara melihat distribusi thread dan *bottleneck* operator (mis. `MUL_MAT`). [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)

***

## 11) **Contoh “Preset Extreme”** (CPU high‑core; 1 konteks)

> Asumsi: 16 physical cores, OpenBLAS, Q4\_K\_M; target latensi rendah + sustained throughput.

**Build**: CMake Release + `-DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS`.
**Env**: [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

```bash
export OPENBLAS_NUM_THREADS=1
```

 [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)

**Run (CLI/server sepadan):**

```bash
llama-cli \
  -m /models/your-model.Q4_K_M.gguf \
  --threads 10 \
  --threads-batch 16 \
  --batch-size 2048 \
  --ubatch-size 512 \
  -fa \
  -C 0xFFFF            # Pin 16 core pertama (sesuaikan topologi CPU)
```

Flag & semantik threads/CPU mask sesuai README server/manpage. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [\[manpages.debian.org\]](https://manpages.debian.org/testing/llama.cpp-examples/llama-parallel.1.en.html)

**Rust (build perf):**

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"

[build]
rustflags = ["-C", "target-cpu=native"]

[dependencies]
tikv-jemallocator = "0.5"
```

Global allocator & pedoman optimizer ada di docs resmi. [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md), [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html)

***

## 12) **Ekstra yang sering dilupakan (tapi krusial)**

* **Tokenizer reuse**: gunakan *tokenizer* yang **persisten** (jangan re‑init per request). `llama.h` memuat API untuk vocab/tokenizer yang dipakai lintas sesi; server juga menyediakan endpoint `/tokenize` untuk amortisasi. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h), [\[huggingface.co\]](https://huggingface.co/docs/inference-endpoints/engines/llama_cpp)
* **Long context**: untuk konteks di atas *training size*, gunakan **RoPE scaling** (`--rope-scaling {linear,yarn}`) bila diperlukan; ukur dampak performanya karena ada biaya ekstra. [\[manpages.debian.org\]](https://manpages.debian.org/testing/llama.cpp-examples/llama-parallel.1.en.html)
* **Cache prompt**: untuk RAG/primer besar, **wajib** `--prompt-cache`/`--prompt-cache-ro` agar start‑latency turun drastis di run berikutnya. [\[github.com\]](https://github.com/ggml-org/llama.cpp/discussions/2110)

***

# Referensi (sesuai poin di atas)

* **Build & CMake & BLAS**: llama.cpp **Build docs** (CMake), opsi BLAS vendor. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
* **Server flags & threading** (`--threads`, `--threads-batch`, `--cpu-mask`, batching, parallel decoding, continuous batching): **tools/server/README.md**. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
* **Parameter `n_batch`/`n_ubatch`/`n_ctx`**: ringkasan **Configuration and Parameters**. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.3-configuration-and-parameters)
* **OpenBLAS runtime** (`OPENBLAS_NUM_THREADS` dkk) & usage: **Runtime variables** & **USAGE.md**. [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/), [\[github.com\]](https://github.com/OpenMathLib/OpenBLAS/blob/develop/USAGE.md)
* **CPU multithreading arsitektur & affinity**: Arm Learning Path (thread‑pool, `MUL_MAT` parallelization). [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)
* **mmap/mlock** & konteks params: `include/llama.h` + binding Node/.NET docs. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h), [\[node-llama...withcat.ai\]](https://node-llama-cpp.withcat.ai/api/type-aliases/LlamaModelOptions), [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/)
* **Prompt cache/session**: diskusi resmi & contoh penggunaan. [\[github.com\]](https://github.com/ggml-org/llama.cpp/discussions/2110), [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/19b03o2/using_promptcache_with_llamacpp/)
* **Kuantisasi (K‑quants)**: **tools/quantize/README.md** + ringkasannya di dokumentasi teknis. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/6.3-model-quantization)
* **Speculative decoding**: contoh & teori (README examples & DeepWiki). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/examples/speculative/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/7.2-speculative-decoding)
* **Rust build perf** (Release/LTO/`target-cpu=native`) & PGO/BOLT: Rust Performance Book + Rustc Dev Guide. [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html), [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)
* **Global allocator** (jemalloc/tikv‑jemallocator) & global allocator framework di Rust std: **docs.rs** + **std::alloc**. [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/index.html)
* **KV cache quantization (eksperimen & hasil)**: laporan komunitas. [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/1dalkm8/memory_tests_using_llamacpp_kv_cache_quantization/)
* **Hugging Face “llama.cpp engine” & endpoints** (untuk integrasi tokenisasi / OAI‑compatible): **HF endpoints llama.cpp**. [\[huggingface.co\]](https://huggingface.co/docs/inference-endpoints/engines/llama_cpp)

***
