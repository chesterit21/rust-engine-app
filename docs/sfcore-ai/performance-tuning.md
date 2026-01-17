## Berikut **panduan praktik terbaik “production‑grade” untuk performa ekstrem di single‑CPU** saat pakai **llama.cpp via Rust binding**. Saya susun dari *build → link → runtime tuning → arsitektur concurrency → caching → kuantisasi → profiling → contoh konfigurasi*—lengkap dengan referensi resmi agar Mas bisa dalami setiap poinnya

> Target: **throughput & latency terbaik** di CPU tunggal (tanpa GPU), stabil, reproducible, dan mudah dioperasionalkan.

***

## 1) Build & link: pastikan binari `llama.cpp` optimal untuk CPU

1. **Bangun dari source (CMake)**, bukan Makefile lawas

    ```bash
    cmake -B build -DCMAKE_BUILD_TYPE=Release
    cmake --build build --config Release -j $(nproc)
    ```

    Petunjuk build resmi berada di `docs/build.md` (CMake sekarang menjadi jalur utama). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

2. **Aktifkan BLAS (OpenBLAS/MKL) untuk prompt prefill**
    * Prefill (matmul batch besar) di CPU sering **lebih cepat** dengan BLAS; aktifkan saat build:

        ```bash
        cmake -B build -DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS
        cmake --build build --config Release -j $(nproc)
        ```

        (Atau set vendor ke `MKL` jika pakai Intel MKL). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

    * Untuk memastikan BLAS aktif dan **hindari oversubscription** thread (lihat §3): set environment **sebelum** run:

        ```bash
        export OPENBLAS_NUM_THREADS=1   # atau MKL_NUM_THREADS=1 jika pakai MKL
        ```

        Variabel runtime OpenBLAS didokumentasikan resmi di sini. [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)
    > Catatan: BLAS paling berdampak di **prefill** (batch besar); fase **decode** (1 token/step) manfaatnya terbatas. Ini umum di CPU‑LLM. (Lihat juga contoh diskusi build BLAS yang konsisten dengan flag `GGML_BLAS/GGML_BLAS_VENDOR`.) [\[stackoverflow.com\]](https://stackoverflow.com/questions/79844758/how-to-build-and-install-a-blas-enabled-llama-cpp-python-ggml-blas-on-on-wsl)

3. **Aktifkan kernel CPU native ISA (SIMD) saat build**  
    CMake modern menambahkan varian **`-march=native`** untuk backend CPU (“ggml-cpu: -march=native” terlihat pada konfigurasi). Pastikan toolchain Mas mem-build *Release* dan memanfaatkan instruksi AVX2/AVX‑512 bila tersedia. [\[discuss.hu...ingface.co\]](https://discuss.huggingface.co/t/issues-when-trying-to-build-llama-cpp/148603)

4. (**Opsional**) Link statis/huruskan artefak yang diperlukan  
    Jika mengemas sebagai service, build **shared lib** `libllama.so` + CLI tools (`llama-cli`, `llama-server`) dari CMake tree. Dokumentasi instalasi/artefak ada di *Installation*. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.1-installation)

***

## 2) Tuning di sisi **Rust binary** (compiler & allocator)

1. **Release + LTO + target CPU**  
    Di `Cargo.toml` / `.cargo/config.toml`:

    ```toml
    [profile.release]
    lto = "thin"        # atau "fat" untuk maksimal (waktu build lebih lama)
    codegen-units = 1
    panic = "abort"
    opt-level = 3

    [build]
    rustflags = ["-C", "target-cpu=native"]
    ```

    * **Release** vs dev dapat beda **10–100×** cepatnya; `target-cpu=native` membuka instruksi SIMD CPU target. [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html)
    * **LTO** sering memberi speed‑up tambahan (trade‑off ukuran binary). [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html), [\[stackoverflow.com\]](https://stackoverflow.com/questions/52291006/why-does-using-lto-increase-the-size-of-my-rust-binary)

2. (**Opsional**) **Allocator** untuk workload multithread panjang  
    Gunakan **jemalloc** (via `tikv-jemallocator`) sebagai global allocator:

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

    Dokumentasi crate & pedoman penggunaan global allocator Rust: [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/index.html)

***

## 3) Runtime knobs (CPU‑only) yang paling berdampak

> Intinya: **pisahkan** konfigurasi untuk **prefill** vs **decode**, **batasi** thread BLAS, dan **pasang CPU affinity** yang konsisten.

1. **Threads untuk compute**

    * `--threads` = thread untuk **decode** (token‑by‑token).
    * `--threads-batch` = thread untuk **prefill/batch** (prompt processing).  
        Dua opsi ini tersedia di **`llama-server`** dan memetakan ke parameter CPU internal. Gunakan nilai berbeda untuk menyeimbangkan throughput. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

    **Resep awal (single‑CPU modern):**

    * `--threads-batch` ≈ jumlah **physical cores** (bukan logical).
    * `--threads` sedikit lebih kecil (mis. 50–70% dari physical cores), karena decode kurang paralelisasi; ini memberi headroom OS/IO.  
        Penjelasan pola MT di backend CPU dan bagaimana work dibagi antar thread ada di panduan Arm (struktur thread‑pool, afinitas). [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)

2. **Batching**
    * `--batch-size` (**n\_batch/logical**) dan `--ubatch-size` (**n\_ubatch/physical**) mengontrol ukuran batch prefill; default umum: **n\_batch=2048**, **n\_ubatch=512**. Besarkan **n\_batch** untuk throughput prefill (hati‑hati RAM), atur **n\_ubatch** sesuai L2/L3 agar tidak “thrash”. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.3-configuration-and-parameters)

3. **CPU affinity / pinning**
    * Gunakan `--cpu-mask` (heksadesimal) atau `--cpu-range` untuk mengikat thread ke inti tertentu (misal, hanya P‑cores pada CPU hybrid). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
    * Ini menstabilkan latency & menghindari migrasi thread. Panduan analisis afinitas & pengamatan pola MT di CPU backend tersedia di Arm Learning Path. [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)

4. **BLAS thread = 1** (hindari oversubscription)
    * Bila build dengan **OpenBLAS/MKL**, set **`OPENBLAS_NUM_THREADS=1`** (atau **`MKL_NUM_THREADS=1`**) karena `llama.cpp` sendiri sudah mem‑*parallelize* operator; menyalakan thread BLAS juga akan menimbulkan dua level threading yang saling berebut core. [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)

5. **mmap/mlock** (manajemen memori file model)
    * `use_mmap=true` → OS **memory‑map** model; cepat start & hemat RSS.
    * `use_mlock=true` → kunci ke RAM (hindari swap/page‑out) bila RAM cukup.  
        Field ini ada di **`llama_context_params`** dan diekspos di berbagai binding (Node/Rust/.NET). Gunakan **mmap** sebagai default, aktifkan **mlock** hanya jika sistem Mas stabil dengan RAM cukup. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h), [\[node-llama...withcat.ai\]](https://node-llama-cpp.withcat.ai/api/type-aliases/LlamaModelOptions), [\[scisharp.github.io\]](https://scisharp.github.io/LLamaSharp/0.4/xmldocs/llama.native.llamacontextparams/)

6. **Fitur server relevan CPU**
    * `-fa/--flash-attn` (aktifkan bila backend/CPU path mendukung, kadang membantu pada prompt panjang).
    * `--swa-full` (Sliding Window Attention cache penuh) untuk *long context* tertentu.
    * Cek *manpage/README server* untuk daftar lengkap flag & semantics. [\[manpages.debian.org\]](https://manpages.debian.org/testing/llama.cpp-examples/llama-parallel.1.en.html), [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

***

## 4) Desain **concurrency** “CPU‑only” yang scalable

> **Single context = single stream**. Untuk **throughput** tinggi di CPU, lebih efektif **jalankan multi‑context** paralel (tiap context terpin ke subset core) daripada memaksa satu context dengan thread sangat besar.

* Arsitektur `llama-server` sudah mendukung **parallel decoding**, **continuous batching**, **slot scheduling**; bisa dijadikan referensi desain service internal. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/5.2-http-server)
* Jika bikin service Rust sendiri (stdin/HTTP), tiru pola ini: buat **pool** N konteks, masing‑masing **cpu‑mask** berbeda (mis. 4‑6 core per konteks) → antrikan request per konteks → agregasi metrics. (Model referensi thread/slot queue di *server\_context* docs.) [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/5.2-http-server)

***

## 5) **Prompt/KV caching** untuk memangkas latensi

1. **Prompt cache / session save‑load**
    * CLI `--prompt-cache`/`--prompt-cache-ro` menyimpan KV untuk prompt panjang agar run berikutnya **langsung lanjut** tanpa re‑ingest. [\[github.com\]](https://github.com/ggml-org/llama.cpp/discussions/2110)
    * API low‑level menyediakan **save/load state** (session file). Banyak wrapper memperlihatkan contoh (diskusi & isu terkait penggunaan di Python wrapper). Terapkan pola yang sama di Rust binding. [\[reddit.com\]](https://www.reddit.com/r/LocalLLaMA/comments/14xzb7a/is_there_a_way_to_persist_llamacpppython_caches/), [\[github.com\]](https://github.com/abetlen/llama-cpp-python/issues/44)

2. **KV cache management**
    * Pahami sistem KV unified/per‑slot (untuk multi‑request) dan operasi defrag/shift; ini penting saat main **continuous batching** dan **context shift**. Dokumentasi arsitektur KV cache tersedia. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/3.4-memory-management)

***

## 6) **Kuantisasi**: pilih format paling “worth it” untuk CPU

* Gunakan **K‑quants** (mis. **Q4\_K\_M** sebagai “sweet‑spot” kualitas vs ukuran; **Q5\_K\_M** bila butuh akurasi lebih)—ini rekomendasi umum di ekosistem `llama.cpp`. Tool `llama-quantize` dan README jenis kuantisasi disediakan di repo resmi. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* Kuantisasi yang lebih agresif → model lebih kecil → **lebih ramah cache** → **lebih cepat** di CPU (terutama pada decode). Rujukan ringkas per jenis kuantisasi (K‑quants/IQ/legacy) ada di dokumentasi komunitas teknis `DeepWiki`. [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/6.3-model-quantization)

***

## 7) **Speculative decoding** (opsional di CPU‑only)

* Teknik **speculative decoding** (model draft kecil mem‑*speculate* beberapa token, lalu diverifikasi oleh model utama) **dapat** mempercepat decode bila *acceptance rate* tinggi—tersedia contoh di `examples/speculative`. Untuk CPU‑only, manfaatnya tergantung bottleneck (sering memerlukan draft sangat kecil). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/examples/speculative/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/7.2-speculative-decoding)

***

## 8) **Profiling & benchmarking**

* Gunakan **`llama-bench`** untuk baseline throughput per konfigurasi build/flag. (Tool berada dalam repositori resmi `llama.cpp`.) [\[github.com\]](https://github.com/ggml-org/llama.cpp)
* Untuk analisis thread & afinitas di CPU: gunakan *perf*/**Arm Streamline**; referensi pola multi‑threading `ggml` membantu identifikasi under‑utilization. [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)

***

## 9) **Contoh resep konfigurasi** (CPU mobile high‑end, single instance)

> Asumsi: 14 core CPU hybrid; **tujuan** latency pendek + stabil saat prompt 1–2k token, *batch inference* ringan.

* Build: `-DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS` (OpenBLAS), Release. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
* Env:

    ```bash
    export OPENBLAS_NUM_THREADS=1         # penting
    ```

     [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)
* Jalankan (server/CLI setara):

    ```bash
    llama-cli \
      -m /models/phi-3-mini.Q4_K_M.gguf \
      --threads 8 \
      --threads-batch 12 \
      --batch-size 2048 \
      --ubatch-size 512 \
      -fa \
      -C 0xFFF            # contoh mask: pin ke 12 core pertama
    ```

    Opsi `--threads/--threads-batch/--cpu-mask` berasal dari README server dan dapat disesuaikan sesuai layout core fisik. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)

***

## 10) **Mengaitkan dari Rust (binding `llama-cpp-rs`)**

* `llama-cpp-rs` (utilityai) mengekspos API yang **dekat** dengan C API `llama.cpp`; update‑nya cepat dan ada contoh `simple` CLI. Gunakan crate ini jika Mas ingin *closest‑to‑upstream*. [\[github.com\]](https://github.com/utilityai/llama-cpp-rs)
* Parameter yang perlu di‑*plumb* dari Rust ke `llama.cpp`: **`n_ctx`, `n_batch`, `n_ubatch`, `n_threads`, `n_threads_batch`, `use_mmap`, `use_mlock`** (nama bidang ada di header **`include/llama.h`**; masing‑masing binding menyajikannya dengan nama serupa). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h)
* Jika binding mengompilasi `llama.cpp` sebagai submodule, pastikan **flag CMake** BLAS & native ISA disalurkan (atau arahkan ke `libllama.so` yang Mas build sendiri dengan opsi di §1). Lihat dokumentasi build di repo utama. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)

***

## Ringkasan *checklist* cepat

* [ ] Build Release `llama.cpp` via CMake + **BLAS ON** (OpenBLAS/MKL). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
* [ ] **OPENBLAS\_NUM\_THREADS=1** (atau **MKL\_NUM\_THREADS=1**). [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)
* [ ] Pisahkan `--threads` (decode) dan `--threads-batch` (prefill). [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
* [ ] Tuning `--batch-size` & `--ubatch-size` (mulai 2048/512). [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.3-configuration-and-parameters)
* [ ] Pin thread: `--cpu-mask` / `--cpu-range`. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
* [ ] `use_mmap=true` (default), `use_mlock=true` bila RAM aman. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/include/llama.h), [\[node-llama...withcat.ai\]](https://node-llama-cpp.withcat.ai/api/type-aliases/LlamaModelOptions)
* [ ] Pakai **Q4\_K\_M**/**Q5\_K\_M** sesuai kebutuhan. [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* [ ] **Prompt cache/session state** untuk beban kerja berulang. [\[github.com\]](https://github.com/ggml-org/llama.cpp/discussions/2110)
* [ ] Rust: **Release+LTO** + `target-cpu=native`, pertimbangkan jemalloc. [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html), [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html), [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md)

***

## Mau saya bantu “operasionalisasikan” ke kode Mas?

Kalau Mas kirim:

* **Nama crate binding** yang dipakai (mis. `llama-cpp-2` / `llama_cpp` / `utilityai/llama-cpp-rs`), dan
* **Cara Mas mem‑*spawn*** konteks + parameter sekarang,

saya bisa **tulis snippet Rust** yang:

1. mengatur **context params** (threads/batch/mmap/mlock),
2. set **env** anti‑oversubscription BLAS,
3. menambahkan **prompt‑cache** (save/load), dan
4. menyiapkan **pools** multi‑context + **CPU mask** berbeda per context (siap dipanggil sebagai Tool MCP via stdin/stdout).

Kalau prefer **endpoint HTTP** per Agent (sesuai rencana Mas), saya bisa bikinkan *Axum service* tipis yang menerapkan pola scheduling ala `llama-server` (queue → slot → stream). [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/5.2-http-server)

***

**Referensi utama untuk pendalaman**

* **Build & opsi CMake (resmi)** — `docs/build.md` [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)
* **Server runtime & flags (threads, cpu‑mask, batching)** — `tools/server/README.md` [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
* **Parameter konteks (`n_ctx`, `n_batch`, `n_ubatch`)** — ringkasan parameter & default [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/2.3-configuration-and-parameters)
* **OpenBLAS runtime vars** — `OPENBLAS_NUM_THREADS`, dll. [\[Runtime va...- OpenBLAS\]](http://www.openmathlib.org/OpenBLAS/docs/runtime_variables/)
* **Kuantisasi & tool `llama-quantize`** — README quantize [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
* **Speculative decoding** — contoh & penjelasan konsep [\[github.com\]](https://github.com/ggml-org/llama.cpp/blob/master/examples/speculative/README.md), [\[deepwiki.com\]](https://deepwiki.com/ggml-org/llama.cpp/7.2-speculative-decoding)
* **Arsitektur threading CPU backend & afinitas** — Arm Learning Path [\[learn.arm.com\]](https://learn.arm.com/learning-paths/servers-and-cloud-computing/llama_cpp_streamline/6_multithread_analyze/)
* **Rust build perf (Release/LTO/target‑cpu)** — Rust Performance Book, Rustc Dev Guide [\[nnethercot....github.io\]](https://nnethercote.github.io/perf-book/build-configuration.html), [\[rustc-dev-...t-lang.org\]](https://rustc-dev-guide.rust-lang.org/building/optimized-build.html)
* **Rust global allocator (jemalloc)** — docs.rs & std alloc docs [\[docs.rs\]](https://docs.rs/crate/jemallocator/latest/source/README.md), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/index.html)
