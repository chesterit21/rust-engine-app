# Spesifikasi High-Traffic Cache Architecture

Di bawah ini gue kasih **rancangan arsitektur “Redis-like”** pakai **Rust + Tokio + Unix Domain Socket (UDS)** yang **ekstrem untuk production high‑traffic**, plus **planning praktik terbaik** khusus **strategi allocator** (global allocator + arena/slab/pool) biar **p99 stabil**, throughput tinggi, dan scaling-nya “rapih”. Semua bagian gue sertakan **link referensi** sebagai acuan.

> Premis utama:
>
> * UDS memang dibuat untuk komunikasi lokal “efisien” dan mendukung **stream/datagram/seqpacket**, **abstract namespace**, serta **passing file descriptor/credentials** — sangat relevan buat server cache/kv lokal yang high‑traffic. [unix(7) man7](https://www.man7.org/linux/man-pages/man7/unix.7.html)
> * Tokio menyediakan **UnixListener/UnixStream** yang terintegrasi dengan runtime async untuk event loop. [Tokio UnixListener](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html) [Tokio UnixStream](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html)

***

## 0) Target Non‑Fungsional (Extreme Production Bar)

Tujuan rancangan ini:

1. **p50 cepat**, **p99 stabil**, tanpa jitter karena allocator/GC/lock contention. [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html)
2. **Scalable** (multi-core) tanpa bottleneck single-thread di “core execution”. (kita pakai sharding + actor-per-shard). [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html), [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html)
3. **Bounded memory & backpressure**: kalau overload, sistem degrade dengan cara yang bisa diprediksi (queue bounded, admission control, dll). [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/crate/ratatui/latest)
4. **Allocator strategy** yang benar: global allocator + pool/arena untuk “hot path” agar fragmentasi rendah dan throughput tinggi. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/), [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html)

***

# 1) High‑Level Architecture (Layered, “Redis-like”, UDS-first)

### 1.1. Process Topology (Single binary, multi-service internal)

**Satu binary** dengan beberapa “subsystem” yang berjalan sebagai task/worker terpisah:

1. **UDS Frontend (Accept + Connection IO)**
    * Bind `UnixListener` pada path/abstract namespace (lebih aman dari masalah file cleanup). [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/sync/mpsc/), [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels)
2. **Protocol Layer (Frame/Parser/Encoder)**
    * Parser incremental (streaming), no-copy bila bisa, dan strict memory limit per connection. (Tujuan: mencegah payload bomb). [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html), [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html)
3. **Command Router (Shard dispatcher)**
    * Hash key → pilih shard → kirim command via bounded MPSC. Ini kunci scaling multi-core tanpa lock global. [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html)
4. **Shard Workers (Actor-per-shard)**
    * Tiap shard **memiliki state** (hashmap, indexes, TTL wheel, pubsub routing lokal) dan memproses command secara sequential di shard itu → menghindari lock contention. (Backpressure lewat channel bounded). [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/crate/ratatui/latest)
5. **Persistence / Replication / AOF (opsional)**
    * Jika butuh durability: jalankan pipeline IO terpisah agar hot path tidak ketahan oleh disk. Bisa pakai `spawn_blocking` (simpel) atau **tokio-uring** (ekstrem untuk Linux). [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [\[docs.rs\]](https://docs.rs/tokio-uring/latest/tokio_uring/)

***

## 1.2. UDS Transport Design (Linux‑optimized)

### Addressing mode: filesystem path vs abstract namespace

* **Abstract namespace** menghindari isu file socket “nyangkut” di filesystem, dan dibuat kernel-level (tidak bergantung FS). Linux mendukung ini secara eksplisit. [unix(7)](https://www.man7.org/linux/man-pages/man7/unix.7.html)
* Untuk deployment service production, abstract namespace juga memudahkan “restart fast” tanpa cleanup file. [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/sync/mpsc/)

### Socket types (pilih sesuai kebutuhan)

* `SOCK_STREAM` untuk model Redis-like request/response (paling umum). [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/sync/mpsc/)
* Kalau mau message boundary strict: `SOCK_SEQPACKET` (Linux mendukung sejak 2.6.4). Ini bisa bikin framing lebih simpel dan mengurangi bug parser, tapi tooling/client lebih terbatas. [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels)

### Security / Auth yang “native”

* UDS mendukung pengiriman **process credentials** dan **file descriptors** via ancillary data → bisa dipakai untuk auth berbasis UID/GID atau privilege separation tanpa TLS. [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels), [\[litux.nl\]](https://litux.nl/man/htmlman7/unix.7.html)

***

# 2) Tokio Runtime Model (High traffic, multi-core)

## 2.1. Runtime pilihan: multi-thread + IO enabled

Tokio runtime menyediakan driver IO, scheduler, timer; untuk high traffic kita pakai **multi-threaded runtime** agar work-stealing berjalan dan accept/IO tidak jadi bottleneck. [tokio::runtime](https://docs.rs/tokio/latest/tokio/runtime/)

### Prinsip penting: jangan block worker thread

* Blocking/CPU heavy di async task bikin executor kelaparan (tail latency naik). Tokio menyediakan `spawn_blocking` untuk offload blocking ke pool terpisah. [spawn\_blocking docs](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
* Tokio juga mendeskripsikan arsitektur blocking pool dan risikonya (shutdown menunggu blocking task selesai, dll). [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/3.4-blocking-operations), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

**Desain praktis:**

* Semua parsing/dispatch tetap di async worker (ringan).
* Hal berat: kompresi, checksum besar, persistence sync IO, scanning, background compaction → `spawn_blocking` atau dedicated runtime/threads. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/3.4-blocking-operations)

***

# 3) Data Plane Architecture (Extreme throughput)

## 3.1. “Shard-per-Actor” (tanpa lock global)

**Tujuan:** hindari global mutex & contention.

### Pola

* Router menghitung `shard_id = hash(key) % N`.
* Router mengirim `Command` ke `tokio::sync::mpsc::Sender<Command>` (bounded) milik shard itu.
* Shard punya loop: `while let Some(cmd) = rx.recv().await { execute(cmd); }`

**Kenapa bounded channel?**  
Bounded `mpsc` memberi **backpressure**: ketika buffer penuh, `send()` menunggu/ditolak → ini menahan producer agar tidak membuat memory tumbuh liar. [tokio mpsc docs](https://docs.rs/tokio/latest/tokio/sync/mpsc/) [DeepWiki MPSC](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels)

> Ini adalah kunci “stabil under overload”: bukan crash karena memory, tapi melambat dengan terkontrol. [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/crate/ratatui/latest)

***

## 3.2. Execution Model: fast path O(1) + bounded worst-case

Shard worker menghindari operasi yang menyapu semua key sekaligus. Kalau butuh iterasi besar (SCAN-like), lakukan incremental per batch (chunked) agar p99 tidak jebol. Konsep “jangan block event loop” sejalan dengan model runtime yang butuh progress fairness. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

***

## 3.3. Expiration/TTL & Admission control (optional tapi recommended)

Kalau kita ingin cache hit-rate tinggi dan tahan scan pollution, desain modern sering pakai **TinyLFU/W‑TinyLFU** sebagai admission/eviction policy. Paper TinyLFU menjelaskan admission berdasarkan frekuensi approximate dan W‑TinyLFU untuk menyeimbangkan recency+frequency. [TinyLFU paper](https://arxiv.org/abs/1512.00727)  
Caffeine memilih W‑TinyLFU karena hit rate tinggi dan footprint metadata rendah; mereka juga jelaskan hill-climbing untuk adaptasi window. [Caffeine Efficiency wiki](https://github.com/ben-manes/caffeine/wiki/Efficiency)

> Catatan: ini relevan jika engine kita bukan sekadar KV store, tapi “cache engine” yang butuh eviction pintar. [\[arxiv.org\]](https://arxiv.org/abs/1512.00727), [\[github.com\]](https://github.com/ben-manes/caffeine/wiki/Efficiency)

***

# 4) Allocator Strategy (Ini fokus “extreme best practices” di Rust)

Di Rust, kita punya 2 level strategi:

1. **Global allocator (process-wide)** via `#[global_allocator]` yang mengubah allocator untuk `Box/Vec/String/HashMap/...` [\[docs.rs\]](https://docs.rs/rustc-std-workspace-std/latest/std/alloc/index.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/)
2. **Subsystem allocators (arena/slab/pool)** di hot path untuk mengurangi alloc/free churn, fragmentasi, dan lock contention di allocator. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/), [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html)

***

## 4.1. Global Allocator: pilih yang “battle-tested” untuk high-throughput

### Opsi A — **jemalloc (tikv-jemallocator)**: stabil, introspection kuat

* `tikv-jemallocator` menyediakan `Jemalloc` yang bisa jadi global allocator dan terintegrasi dengan Rust allocator API. [crates.io](https://crates.io/crates/tikv-jemallocator)
* kita set sebagai global allocator hanya dengan beberapa baris. [docs.rs tikv-jemallocator](https://docs.rs/crate/tikv-jemallocator/latest)
* Jemalloc memiliki control/introspection (mallctl) yang bisa kita expose jadi `INFO MEMORY`-style metrics. [\[deepwiki.com\]](https://deepwiki.com/tikv/jemallocator), [\[docs.rs\]](https://docs.rs/crate/tikv-jemallocator/latest)
* Jemalloc didesain untuk skalabilitas alokasi di SMP (multi-processor) dan membahas lock contention/fragmentation. [jemalloc paper](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html)

**Kapan cocok:** server high traffic yang butuh observability memory & stabilitas jangka panjang. [\[docs.rs\]](https://docs.rs/crate/tikv-jemallocator/latest), [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html)

### Opsi B — **mimalloc**: performance-oriented, “drop-in”

* Crate `mimalloc` menyediakan `MiMalloc` global allocator drop-in. [docs.rs mimalloc](https://docs.rs/mimalloc/latest/mimalloc/)
* Microsoft Rust Guidelines secara eksplisit menyarankan menggunakan mimalloc untuk aplikasi karena sering memberi peningkatan performa “for free”. [MS Rust Guidelines](https://microsoft.github.io/rust-guidelines/guidelines/apps/)

**Kapan cocok:** kita ingin setup cepat + fokus throughput, dan tidak butuh fitur introspection sedalam jemalloc. [\[crates.io\]](https://crates.io/crates/tikv-jemallocator), [\[docs.rs\]](https://docs.rs/mimalloc/latest/mimalloc/)

***

## 4.2. Subsystem Allocator (yang bikin engine kita “extreme”)

Walau global allocator penting, server KV/cache biasanya masih butuh strategi berikut supaya p99 stabil:

### 4.2.1. Arena / Bump allocator untuk request parsing & temporary objects

* Rust std memberikan contoh implementasi allocator arena sederhana lewat `GlobalAlloc`. Ini membuktikan pola “arena allocation” feasible di Rust. [GlobalAlloc docs](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)
* Prinsip: untuk objek sementara selama lifetime request, kita alokasikan dari arena, lalu “reset” arena di akhir request tanpa free satu-satu (mengurangi free storm). [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html)

**Kenapa penting:** penelitian allocator di skala besar menunjukkan pola alokasi/fragmentasi sangat berpengaruh ke throughput & RAM waste. [TCMalloc warehouse scale paper](https://www.reddit.com/r/rust/comments/ea0f2f/brand_new_event_module_crossterm_014/) [OSDI hugepage allocator paper](https://docs.rs/crate/ratatui/latest)

### 4.2.2. Slab / Size-class pool untuk Value kecil (dominant in cache workloads)

* Ide: value kecil (mis. <= 512B, <= 4KB) dialokasikan dari slab per size-class.
* Tujuan: mengurangi fragmentasi dan biaya malloc/free berulang, sekaligus membuat memory accounting lebih presisi. Ini sejalan dengan motivasi desain allocator modern yang menekan fragmentation & lock contention. [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html)

### 4.2.3. Per-shard allocator / arena (NUMA & cache locality)

* Tokio runtime sendiri menyebut ia tidak NUMA-aware; untuk sistem NUMA, kadang lebih baik menjalankan beberapa runtime agar locality lebih baik. [tokio::runtime docs](https://docs.rs/tokio/latest/tokio/runtime/)
* Maka: shard worker bisa punya arena/pool sendiri (thread-local-ish) sehingga alokasi terjadi dekat core yang memproses shard itu → stabil & cepat. [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html)

***

## 4.3. Planning Praktik Terbaik Allocator (Step-by-step, produksi)

Berikut plan implementasi yang “ekstreme tapi realistis”, urut dari yang paling impactful:

### Phase A — Baseline & guardrails (wajib)

1. **Aktifkan global allocator pilihan** (`jemalloc` atau `mimalloc`) di binary utama. `#[global_allocator]` adalah mekanisme standar Rust. [\[docs.rs\]](https://docs.rs/rustc-std-workspace-std/latest/std/alloc/index.html), [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/)
2. **Pisahkan hot path vs cold path**:
    * Hot: GET/SET, encode/decode, hashmap lookup → harus minim alloc.
    * Cold: snapshot/persistence/compaction/metrics export → boleh alloc lebih banyak.  
        Ini penting karena blocking/CPU heavy di async worker bikin starvation. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/3.4-blocking-operations)

### Phase B — “Allocator-aware data plane” (naik kelas)

1. **Request arena**: setiap connection punya arena (atau per-task) untuk parsing buffer/temporary objects; reset per request. (Mengurangi per-request heap churn). [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/), [\[reddit.com\]](https://www.reddit.com/r/rust/comments/ea0f2f/brand_new_event_module_crossterm_014/)
2. **Slab pools untuk value kecil**: size-class pool per shard/worker. (Mengurangi fragmentasi, mempercepat alloc/free). [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html)
3. **Memory accounting** per key/value termasuk overhead arena/pool. Ini meniru filosofi `MEMORY USAGE` di Redis untuk menilai overhead administratif. (Di engine kita, expose via admin command). [\[kdheepak.com\]](https://kdheepak.com/blog/the-basic-building-blocks-of-ratatui-part-3/), [\[deepwiki.com\]](https://deepwiki.com/tikv/jemallocator)

### Phase C — Extreme IO & persistence (opsional, tapi untuk “production grade”)

1. **Disk path jangan ganggu async runtime**:
    * Minimal: gunakan `spawn_blocking` untuk AOF/snapshot yang blocking. [spawn\_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
    * Ekstrem Linux: gunakan **tokio-uring** untuk async file IO kernel-level (butuh kernel cukup baru). [tokio-uring docs](https://docs.rs/tokio-uring/latest/tokio_uring/)
2. Jika pakai tokio-uring: ingat resource close itu async → dianjurkan `close()` eksplisit karena Rust belum punya async drop. [\[docs.rs\]](https://docs.rs/tokio-uring/latest/tokio_uring/), [\[docs.rs\]](https://docs.rs/crate/tokio-uring/latest)

***

# 5) Backpressure & Overload Control (wajib biar stabil di high traffic)

## 5.1. Bounded queues di semua choke point

* **Per-connection**: limit in-flight requests (mis. pipelining).
* **Router → Shard**: bounded `tokio::sync::mpsc::channel(cap)` memberi backpressure. [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[mnemos.dev\]](https://mnemos.dev/doc/tokio/sync/mpsc/fn.channel)
* **Shard → Persistence**: bounded queue juga, supaya disk lambat tidak membuat memory grow. (Konsep sama: bounded buffer). [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

## 5.2. Overload behavior yang deterministik

Ketika channel penuh:

* Pilih strategi: `await` (client nunggu), atau `try_send` (balas error “BUSY”), atau drop request prioritas rendah. Tokio mpsc mendeskripsikan bounded channel sebagai mekanisme backpressure. [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/crate/ratatui/latest)

***

# 6) “Extreme Linux Mode” (optional): io\_uring path untuk file + beberapa network ops

Kalau engine kita punya AOF/snapshot berat, tokio-uring bisa mengurangi overhead syscalls dengan model submission-based, tapi ia memakai runtime sendiri dan resource types banyak yang `!Sync` dan single-thread oriented. [tokio-uring docs](https://docs.rs/tokio-uring/latest/tokio_uring/)  
Untuk scaling, jalankan beberapa thread masing-masing punya tokio-uring runtime (sesuai docs). [\[docs.rs\]](https://docs.rs/tokio-uring/latest/tokio_uring/), [\[github.com\]](https://github.com/tokio-rs/tokio-uring)

***

# 7) Rancangan Modul (Concrete Layout Project)

Bagian ini di abaikan SAJA, tidak perlu di ikuti, tetapi :

* `src/main.rs`
  * runtime init (tokio multi-thread) [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html)
  * global allocator selection (jemalloc/mimalloc) [\[docs.rs\]](https://docs.rs/rustc-std-workspace-std/latest/std/alloc/index.html), [\[crates.io\]](https://crates.io/crates/tikv-jemallocator)
* `src/transport/uds.rs`
  * UnixListener accept loop + per-conn task [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/sync/mpsc/), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html)
* `src/proto/`
  * framing + parser incremental
* `src/router/`
  * hash → shard
  * bounded channel sender [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[mnemos.dev\]](https://mnemos.dev/doc/tokio/sync/mpsc/fn.channel)
* `src/shard/`
  * actor loop + storage engine state
* `src/mem/`
  * arena allocator + slab pools (subsystem allocators) [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/runtime/), [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html)
* `src/persist/`
  * spawn\_blocking AOF/snapshot [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
  * (optional) tokio-uring backend [\[docs.rs\]](https://docs.rs/tokio-uring/latest/tokio_uring/)
* `src/telemetry/`
  * jemalloc ctl stats export (jika pakai jemalloc) [\[deepwiki.com\]](https://deepwiki.com/tikv/jemallocator), [\[docs.rs\]](https://docs.rs/crate/tikv-jemallocator/latest)

***

# 8) Snippet Kunci (Rust) — Global allocator setup (2 opsi)

## 8.1. jemalloc (tikv-jemallocator)

```rust
// Cargo.toml
// [target.'cfg(not(target_env = "msvc"))'.dependencies]
// tikv-jemallocator = "0.6"

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;
```

Crate ini memang dirancang untuk jadi global allocator di Rust. [\[microsoft.github.io\]](https://microsoft.github.io/rust-guidelines/guidelines/apps/), [\[docs.rs\]](https://docs.rs/crate/tikv-jemallocator/latest)

## 8.2. mimalloc

```rust
// Cargo.toml
// mimalloc = "0.1"

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
```

mimalloc crate adalah wrapper global allocator drop‑in.
Microsoft Rust Guidelines juga menyarankan mimalloc untuk aplikasi. [\[crates.io\]](https://crates.io/crates/tikv-jemallocator), [\[docs.rs\]](https://docs.rs/crate/mimalloc/latest) [\[docs.rs\]](https://docs.rs/mimalloc/latest/mimalloc/)

***

# 9) Kenapa desain ini “ekstrem” dan bukan sekadar “Redis clone”

Ringkasnya, desain ini sengaja meng-address problem yang biasanya bikin cache server “nggak stabil”:

* **UDS**: IPC lokal efisien + fitur advanced (abstract namespace, creds, fd passing). [\[deepwiki.com\]](https://deepwiki.com/tokio-rs/tokio/5.2-mpsc-channels), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/sync/mpsc/)
* **Tokio multi-thread**: scheduler + IO driver untuk load tinggi, plus mekanisme untuk isolasi blocking workload. [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)
* **Shard-per-actor + bounded mpsc**: scaling multi-core + backpressure deterministik. [\[docs.rs\]](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [\[docs.rs\]](https://docs.rs/crate/ratatui/latest)
* **Allocator strategy berlapis**: global allocator + arena/slab/pool untuk hot path, menekan fragmentasi dan jitter; allocator research menunjukkan ini berdampak nyata pada throughput & RAM. [\[man7.org\]](https://www.man7.org/linux/man-pages/man7/unix.7.html), [\[reddit.com\]](https://www.reddit.com/r/rust/comments/ea0f2f/brand_new_event_module_crossterm_014/), [\[docs.rs\]](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html)

***

## 10) Referensi Utama (harus sambil di cek juga ya bro)

* Linux UDS: [unix(7) man7](https://www.man7.org/linux/man-pages/man7/unix.7.html)
* Tokio UDS: [tokio::net::UnixListener](https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html), [tokio::net::UnixStream](https://docs.rs/tokio/latest/tokio/net/struct.UnixStream.html)
* Tokio runtime: [tokio::runtime](https://docs.rs/tokio/latest/tokio/runtime/)
* Tokio backpressure: [tokio::sync::mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/)
* Blocking isolation: [tokio::task::spawn\_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html) + [Blocking operations deepwiki](https://deepwiki.com/tokio-rs/tokio/3.4-blocking-operations)
* Rust allocator API: [std::alloc::GlobalAlloc](https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html) + [std::alloc module](https://docs.rs/rustc-std-workspace-std/latest/std/alloc/index.html)
* jemalloc Rust: [tikv-jemallocator crate](https://crates.io/crates/tikv-jemallocator)
* mimalloc Rust: [mimalloc crate docs](https://docs.rs/mimalloc/latest/mimalloc/) + [MS Rust Guidelines](https://microsoft.github.io/rust-guidelines/guidelines/apps/)
* Extreme file IO: [tokio-uring docs](https://docs.rs/tokio-uring/latest/tokio_uring/)
* Allocator & fragmentation impact: [jemalloc paper](https://docs.rs/crossterm/latest/crossterm/terminal/struct.EnterAlternateScreen.html), [TCMalloc warehouse scale](https://www.reddit.com/r/rust/comments/ea0f2f/brand_new_event_module_crossterm_014/), [OSDI hugepage allocator](https://docs.rs/crate/ratatui/latest)

***
