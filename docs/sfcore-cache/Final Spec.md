# Final Spec

### **versi FINAL**. Di bawah ini adalah **Implementation Plan v1 (FINAL, sesuai Final Spec)** yang **langsung bisa di implementasi** lengkap

* **Struktur workspace** (masuk `crates/`)
* **Semua modul & dependency graph**
* **Code snippets per file** (praktik terbaik, konsisten dengan spec)
* **Konfigurasi server** (env + default)
* **Client SDK minimal** (Rust) + contoh usage
* **Deployment** (systemd) + cara jalanin

> Acuan real-world untuk komponen utama:  
> Tokio UDS (`UnixListener/UnixStream`) untuk IPC lokal, `BytesMut` untuk buffer/framing efisien, `tokio::sync::broadcast` untuk pub/sub bounded dengan `Lagged`, dan `MemAvailable` dari `/proc/meminfo` untuk estimasi memory pressure. Untuk penamaan key ber-namespace dengan `:` sebagai delimiter (biar konsisten dan mudah housekeeping), ini praktik umum dan direkomendasikan untuk mengelola dataset schema-less serta mempertimbangkan biaya memori key. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo) [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html) [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html) [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html) [\[github.com\]](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Net.Sockets/src/System/Net/Sockets/UnixDomainSocketEndPoint.Windows.cs), [\[blog.fzankl.de\]](https://blog.fzankl.de/unix-domain-sockets-in-net-6-basics-and-real-world-examples), [\[learn.microsoft.com\]](https://learn.microsoft.com/en-us/aspnet/core/grpc/interprocess?view=aspnetcore-10.0)

***

# A) Workspace Layout (FINAL)

Di root workspace kamu:

    <workspace-root>/
      Cargo.toml
      crates/
        localcached-proto/        # shared protocol: types + encode/decode payload (no tokio)
        localcached-server/       # daemon UDS server (tokio)
        localcached-client/       # Rust client SDK (tokio)

Update root `Cargo.toml`:

```toml
[workspace]
members = [
  "crates/localcached-proto",
  "crates/localcached-server",
  "crates/localcached-client",
]
resolver = "2"
```

***

# B) Dependency Graph (JELAS)

* `localcached-proto`
  * **Tidak** tergantung Tokio
  * Menyediakan: opcode/status enums, payload encoder/decoder, key validator, event decoder

* `localcached-server`
  * Depends: `localcached-proto`
  * Runtime: `tokio`, `bytes`, `dashmap`, `parking_lot`, `tracing`, dll.
  * Modul: server runtime, store, pubsub bus, eviction, meminfo watcher, stats counters

* `localcached-client`
  * Depends: `localcached-proto`
  * Runtime: `tokio`, `bytes`
  * Menyediakan API: `KvClient` (set/get/del/stats), `SubClient` (subscribe stream)

***

# C) Crate 1 — `localcached-proto` (FINAL)

## C.1 `crates/localcached-proto/Cargo.toml`

```toml
[package]
name = "localcached-proto"
version = "0.1.0"
edition = "2021"

[dependencies]
bytes = "1"
thiserror = "2"
```

## C.2 Struktur file

    localcached-proto/src/
      lib.rs
      types.rs
      error.rs
      key.rs
      payload.rs
      stats.rs

## C.3 `src/lib.rs`

```rust
pub mod types;
pub mod error;
pub mod key;
pub mod payload;
pub mod stats;

pub use types::*;
pub use error::*;
pub use key::*;
pub use payload::*;
pub use stats::*;
```

## C.4 `src/types.rs` (FINAL enums)

```rust
#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Opcode {
    Set = 0x01,
    Get = 0x02,
    Del = 0x03,
    Ping = 0x04,
    Stats = 0x05,

    Subscribe = 0x20,
    Unsubscribe = 0x21,

    PushEvent = 0x80,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Ok = 0x00,
    NotFound = 0x01,

    ErrBadPayload = 0x10,
    ErrUnsupportedFormat = 0x11,
    ErrTooLarge = 0x12,
    ErrInternal = 0x13,
    ErrUnauthorized = 0x14,
    ErrLagged = 0x15,
    ErrInvalidKeyFormat = 0x16,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ValueFormat {
    Json = 1,
    MsgPack = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    Invalidate = 1,
    TableChanged = 2, // implicit upsert
}
```

## C.5 `src/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtoError {
    #[error("invalid key format; expected svc:table:pk")]
    InvalidKeyFormat,

    #[error("invalid utf8")]
    InvalidUtf8,

    #[error("bad payload")]
    BadPayload,

    #[error("unsupported format")]
    UnsupportedFormat,
}
```

## C.6 `src/key.rs` (MANDATORY validator + topic derivation)

```rust
use crate::ProtoError;

pub fn validate_key_3parts(key: &str) -> Result<(&str, &str, &str), ProtoError> {
    let mut it = key.splitn(3, ':');
    let svc = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    let table = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    let pk = it.next().ok_or(ProtoError::InvalidKeyFormat)?;
    if svc.is_empty() || table.is_empty() || pk.is_empty() {
        return Err(ProtoError::InvalidKeyFormat);
    }
    Ok((svc, table, pk))
}

pub fn topic_from_key(key: &str) -> Result<String, ProtoError> {
    let (svc, table, _) = validate_key_3parts(key)?;
    Ok(format!("t:{svc}:{table}"))
}

pub fn validate_topic(topic: &str) -> Result<(&str, &str), ProtoError> {
    // expected "t:svc:table"
    let mut it = topic.splitn(3, ':');
    let prefix = it.next().unwrap_or("");
    let svc = it.next().ok_or(ProtoError::BadPayload)?;
    let table = it.next().ok_or(ProtoError::BadPayload)?;
    if prefix != "t" || svc.is_empty() || table.is_empty() {
        return Err(ProtoError::BadPayload);
    }
    Ok((svc, table))
}
```

## C.7 `src/payload.rs` (FINAL payload encode/decode)

```rust
use bytes::{Buf, BufMut, Bytes, BytesMut};
use crate::{Opcode, Status, ValueFormat, EventType, ProtoError};

#[derive(Debug)]
pub struct SetReq {
    pub format: ValueFormat,
    pub suppress_publish: bool, // flags bit0
    pub key: String,
    pub value: Bytes,
    pub ttl_ms: u64,
}

#[derive(Debug)]
pub struct GetReq {
    pub key: String,
}

#[derive(Debug)]
pub struct DelReq {
    pub key: String,
}

#[derive(Debug)]
pub struct SubscribeReq {
    pub topic: String,
}

#[derive(Debug, Clone)]
pub struct PushEvent {
    pub event_type: EventType,
    pub topic: String,
    pub key: String,
    pub ts_ms: u64,
}

pub fn encode_request(op: Opcode, payload: &[u8]) -> BytesMut {
    // frame: [u32 len][u8 opcode][payload]
    let len = 1 + payload.len();
    let mut out = BytesMut::with_capacity(4 + len);
    out.put_u32_le(len as u32);
    out.put_u8(op as u8);
    out.extend_from_slice(payload);
    out
}

pub fn encode_response(status: Status, payload: &[u8]) -> BytesMut {
    let len = 1 + payload.len();
    let mut out = BytesMut::with_capacity(4 + len);
    out.put_u32_le(len as u32);
    out.put_u8(status as u8);
    out.extend_from_slice(payload);
    out
}

pub fn decode_set_payload(mut p: &[u8]) -> Result<SetReq, ProtoError> {
    if p.remaining() < 1 + 1 + 2 { return Err(ProtoError::BadPayload); }
    let fmt = p.get_u8();
    let flags = p.get_u8();
    let format = match fmt {
        1 => ValueFormat::Json,
        2 => ValueFormat::MsgPack,
        _ => return Err(ProtoError::UnsupportedFormat),
    };
    let suppress_publish = (flags & 0b0000_0001) != 0;

    let key_len = p.get_u16_le() as usize;
    if p.remaining() < key_len + 4 + 8 { return Err(ProtoError::BadPayload); }
    let key_bytes = &p[..key_len];
    let key = std::str::from_utf8(key_bytes).map_err(|_| ProtoError::InvalidUtf8)?.to_string();
    p.advance(key_len);

    let val_len = p.get_u32_le() as usize;
    if val_len == 0 || p.remaining() < val_len + 8 { return Err(ProtoError::BadPayload); }
    let val = Bytes::copy_from_slice(&p[..val_len]);
    p.advance(val_len);

    let ttl_ms = p.get_u64_le();
    Ok(SetReq { format, suppress_publish, key, value: val, ttl_ms })
}

pub fn encode_set_payload(format: ValueFormat, suppress_publish: bool, key: &str, value: &[u8], ttl_ms: u64) -> BytesMut {
    let flags = if suppress_publish { 1u8 } else { 0u8 };
    let mut out = BytesMut::with_capacity(1+1+2+key.len()+4+value.len()+8);
    out.put_u8(format as u8);
    out.put_u8(flags);
    out.put_u16_le(key.len() as u16);
    out.extend_from_slice(key.as_bytes());
    out.put_u32_le(value.len() as u32);
    out.extend_from_slice(value);
    out.put_u64_le(ttl_ms);
    out
}

pub fn decode_key_only(mut p: &[u8]) -> Result<String, ProtoError> {
    if p.remaining() < 2 { return Err(ProtoError::BadPayload); }
    let klen = p.get_u16_le() as usize;
    if p.remaining() < klen { return Err(ProtoError::BadPayload); }
    let key = std::str::from_utf8(&p[..klen]).map_err(|_| ProtoError::InvalidUtf8)?.to_string();
    Ok(key)
}

pub fn encode_key_only(key: &str) -> BytesMut {
    let mut out = BytesMut::with_capacity(2 + key.len());
    out.put_u16_le(key.len() as u16);
    out.extend_from_slice(key.as_bytes());
    out
}

pub fn decode_subscribe_payload(mut p: &[u8]) -> Result<String, ProtoError> {
    if p.remaining() < 2 { return Err(ProtoError::BadPayload); }
    let tlen = p.get_u16_le() as usize;
    if p.remaining() < tlen { return Err(ProtoError::BadPayload); }
    let topic = std::str::from_utf8(&p[..tlen]).map_err(|_| ProtoError::InvalidUtf8)?.to_string();
    Ok(topic)
}

pub fn encode_subscribe_payload(topic: &str) -> BytesMut {
    let mut out = BytesMut::with_capacity(2 + topic.len());
    out.put_u16_le(topic.len() as u16);
    out.extend_from_slice(topic.as_bytes());
    out
}

pub fn decode_push_event_payload(mut p: &[u8]) -> Result<PushEvent, ProtoError> {
    if p.remaining() < 1 + 2 { return Err(ProtoError::BadPayload); }
    let et = p.get_u8();
    let event_type = match et {
        1 => EventType::Invalidate,
        2 => EventType::TableChanged,
        _ => return Err(ProtoError::BadPayload),
    };
    let tlen = p.get_u16_le() as usize;
    if p.remaining() < tlen + 2 { return Err(ProtoError::BadPayload); }
    let topic = std::str::from_utf8(&p[..tlen]).map_err(|_| ProtoError::InvalidUtf8)?.to_string();
    p.advance(tlen);

    let klen = p.get_u16_le() as usize;
    if p.remaining() < klen + 8 { return Err(ProtoError::BadPayload); }
    let key = std::str::from_utf8(&p[..klen]).map_err(|_| ProtoError::InvalidUtf8)?.to_string();
    p.advance(klen);

    let ts_ms = p.get_u64_le();
    Ok(PushEvent { event_type, topic, key, ts_ms })
}

pub fn encode_push_event_payload(ev: &PushEvent) -> BytesMut {
    let mut out = BytesMut::with_capacity(1 + 2 + ev.topic.len() + 2 + ev.key.len() + 8);
    out.put_u8(ev.event_type as u8);
    out.put_u16_le(ev.topic.len() as u16);
    out.extend_from_slice(ev.topic.as_bytes());
    out.put_u16_le(ev.key.len() as u16);
    out.extend_from_slice(ev.key.as_bytes());
    out.put_u64_le(ev.ts_ms);
    out
}
```

## C.8 `src/stats.rs` (FINAL binary stats struct)

```rust
use bytes::{Buf, BufMut, BytesMut};
use crate::ProtoError;

#[derive(Debug, Clone, Copy)]
pub struct StatsV1 {
    pub uptime_ms: u64,
    pub keys_count: u64,
    pub approx_mem_bytes: u64,
    pub evictions_total: u64,
    pub pubsub_topics: u64,
    pub events_published_total: u64,
    pub events_lagged_total: u64,
    pub invalid_key_total: u64,
    pub mem_pressure_bp: u16,
}

pub fn encode_stats_v1(s: &StatsV1) -> BytesMut {
    let mut out = BytesMut::with_capacity(1 + 8*8 + 2 + 2);
    out.put_u8(1); // stats_version
    out.put_u64_le(s.uptime_ms);
    out.put_u64_le(s.keys_count);
    out.put_u64_le(s.approx_mem_bytes);
    out.put_u64_le(s.evictions_total);
    out.put_u64_le(s.pubsub_topics);
    out.put_u64_le(s.events_published_total);
    out.put_u64_le(s.events_lagged_total);
    out.put_u64_le(s.invalid_key_total);
    out.put_u16_le(s.mem_pressure_bp);
    out.put_u16_le(0); // reserved
    out
}

pub fn decode_stats_v1(mut p: &[u8]) -> Result<StatsV1, ProtoError> {
    if p.remaining() < 1 { return Err(ProtoError::BadPayload); }
    let ver = p.get_u8();
    if ver != 1 { return Err(ProtoError::BadPayload); }
    if p.remaining() < 8*8 + 2 + 2 { return Err(ProtoError::BadPayload); }

    Ok(StatsV1{
        uptime_ms: p.get_u64_le(),
        keys_count: p.get_u64_le(),
        approx_mem_bytes: p.get_u64_le(),
        evictions_total: p.get_u64_le(),
        pubsub_topics: p.get_u64_le(),
        events_published_total: p.get_u64_le(),
        events_lagged_total: p.get_u64_le(),
        invalid_key_total: p.get_u64_le(),
        mem_pressure_bp: p.get_u16_le(),
    })
}
```

***

# D) Crate 2 — `localcached-server` (FINAL)

## D.1 `crates/localcached-server/Cargo.toml`

```toml
[package]
name = "localcached-server"
version = "0.1.0"
edition = "2021"

[dependencies]
localcached-proto = { path = "../localcached-proto" }

tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "sync", "time", "signal", "io-util"] }
bytes = "1"
dashmap = "6"
parking_lot = "0.12"
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

mimalloc = { version = "0.1", optional = true }

[features]
default = []
alloc_mimalloc = ["mimalloc"]
```

> Tokio UDS, broadcast, dan BytesMut dipakai persis sesuai docs real-world mereka. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo), [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide)

## D.2 Struktur file

    localcached-server/src/
      main.rs
      config.rs
      time.rs
      framing.rs
      metrics.rs
      sys/meminfo.rs
      store/{mod.rs,entry.rs,kv.rs,eviction.rs}
      pubsub/{mod.rs,bus.rs}
      server/{mod.rs,conn_kv.rs,conn_sub.rs}

***

## D.3 `src/main.rs`

```rust
#[cfg(feature = "alloc_mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use tracing_subscriber::EnvFilter;

mod config;
mod time;
mod framing;
mod metrics;
mod sys;
mod store;
mod pubsub;
mod server;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::from_env();
    server::run(cfg).await
}
```

***

## D.4 `src/config.rs` (FINAL config via env)

```rust
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub socket_path: String,
    pub max_frame_bytes: usize,
    pub pressure_hot: f64,
    pub pressure_cool: f64,
    pub pubsub_capacity: usize,
    pub pressure_poll_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_path: "/run/localcached.sock".to_string(),
            max_frame_bytes: 8 * 1024 * 1024,
            pressure_hot: 0.85,
            pressure_cool: 0.80,
            pubsub_capacity: 256,
            pressure_poll_ms: 150,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let mut c = Self::default();
        if let Ok(v) = env::var("LOCALCACHED_SOCKET") { c.socket_path = v; }
        if let Ok(v) = env::var("LOCALCACHED_MAX_FRAME") { c.max_frame_bytes = v.parse().unwrap_or(c.max_frame_bytes); }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_HOT") { c.pressure_hot = v.parse().unwrap_or(c.pressure_hot); }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_COOL") { c.pressure_cool = v.parse().unwrap_or(c.pressure_cool); }
        if let Ok(v) = env::var("LOCALCACHED_PUBSUB_CAP") { c.pubsub_capacity = v.parse().unwrap_or(c.pubsub_capacity); }
        if let Ok(v) = env::var("LOCALCACHED_PRESSURE_POLL_MS") { c.pressure_poll_ms = v.parse().unwrap_or(c.pressure_poll_ms); }
        c
    }
}
```

***

## D.5 `src/time.rs`

```rust
pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}
```

***

## D.6 `src/framing.rs` (FINAL async framing)

Tokio + BytesMut untuk read frame. Ini sesuai pattern networking umum dan memanfaatkan buffer efisien. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)

```rust
use bytes::{BytesMut, Buf};
use tokio::io::AsyncReadExt;

pub async fn read_frame<R: AsyncReadExt + Unpin>(
    r: &mut R,
    max_frame: usize,
    buf: &mut BytesMut,
) -> std::io::Result<Option<BytesMut>> {
    while buf.len() < 4 {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    let len = (&buf[..4]).get_u32_le() as usize;
    if len == 0 || len > max_frame {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
    }
    let total = 4 + len;
    while buf.len() < total {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    Ok(Some(buf.split_to(total)))
}
```

***

## D.7 `src/metrics.rs` (FINAL counters untuk STATS)

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use crate::time::now_ms;

pub struct Metrics {
    start_ms: u64,
    pub evictions_total: AtomicU64,
    pub events_published_total: AtomicU64,
    pub events_lagged_total: AtomicU64,
    pub invalid_key_total: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            start_ms: now_ms(),
            evictions_total: AtomicU64::new(0),
            events_published_total: AtomicU64::new(0),
            events_lagged_total: AtomicU64::new(0),
            invalid_key_total: AtomicU64::new(0),
        }
    }

    pub fn uptime_ms(&self) -> u64 { now_ms().saturating_sub(self.start_ms) }

    pub fn inc_evictions(&self, n: u64) { self.evictions_total.fetch_add(n, Ordering::Relaxed); }
    pub fn inc_published(&self) { self.events_published_total.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_lagged(&self) { self.events_lagged_total.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_invalid_key(&self) { self.invalid_key_total.fetch_add(1, Ordering::Relaxed); }
}
```

***

## D.8 `src/sys/meminfo.rs` (FINAL mem pressure)

MemAvailable & MemTotal dari `/proc/meminfo` sesuai definisi man page. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

```rust
use std::fs;

#[derive(Debug, Clone, Copy)]
pub struct MemInfo {
    pub mem_total_kb: u64,
    pub mem_available_kb: u64,
}

pub fn read_meminfo() -> std::io::Result<MemInfo> {
    let s = fs::read_to_string("/proc/meminfo")?;
    let mut total = 0u64;
    let mut avail = 0u64;

    for line in s.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            avail = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
        }
    }
    Ok(MemInfo { mem_total_kb: total, mem_available_kb: avail })
}

pub fn pressure_bp(mi: MemInfo) -> u16 {
    if mi.mem_total_kb == 0 { return 0; }
    let avail = mi.mem_available_kb as f64;
    let total = mi.mem_total_kb as f64;
    let p = 1.0 - (avail / total);
    let bp = (p * 10000.0).clamp(0.0, 10000.0) as u16;
    bp
}

pub fn pressure(mi: MemInfo) -> f64 {
    if mi.mem_total_kb == 0 { return 0.0; }
    1.0 - (mi.mem_available_kb as f64 / mi.mem_total_kb as f64)
}
```

***

## D.9 `src/store/mod.rs`

```rust
pub mod entry;
pub mod kv;
pub mod eviction;

pub use kv::KvStore;
pub use eviction::Evictor;
```

## D.10 `src/store/entry.rs`

```rust
use bytes::Bytes;
use std::sync::atomic::{AtomicU64, Ordering};
use localcached_proto::ValueFormat;

pub struct Entry {
    pub format: ValueFormat,
    pub value: Bytes,
    pub expires_at_ms: u64, // 0 none
    pub touched_ms: AtomicU64,
    pub size_bytes: usize,
}

impl Entry {
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at_ms != 0 && now >= self.expires_at_ms
    }
    pub fn touch(&self, now: u64) { self.touched_ms.store(now, Ordering::Relaxed); }
}
```

## D.11 `src/store/kv.rs` (DashMap store)

DashMap dipakai sebagai concurrent HashMap yang praktis untuk tahap v1. [\[docs.rs\]](https://docs.rs/crate/parking_lot/latest), [\[docs.serai.exchange\]](https://docs.serai.exchange/rust/parking_lot/index.html)

```rust
use dashmap::DashMap;
use bytes::Bytes;
use localcached_proto::ValueFormat;
use crate::store::entry::Entry;

pub struct KvStore {
    map: DashMap<String, Entry>,
}

impl Default for KvStore {
    fn default() -> Self {
        Self { map: DashMap::new() }
    }
}

impl KvStore {
    pub fn set(&self, key: String, format: ValueFormat, value: Bytes, expires_at_ms: u64, now_ms: u64) {
        let size_bytes = key.len() + value.len();
        let e = Entry {
            format,
            value,
            expires_at_ms,
            touched_ms: now_ms.into(),
            size_bytes,
        };
        self.map.insert(key, e);
    }

    pub fn get(&self, key: &str, now_ms: u64) -> Option<(ValueFormat, Bytes, u64)> {
        let g = self.map.get(key)?;
        if g.is_expired(now_ms) {
            drop(g);
            self.map.remove(key);
            return None;
        }
        g.touch(now_ms);
        let ttl_rem = if g.expires_at_ms == 0 { 0 } else { g.expires_at_ms.saturating_sub(now_ms) };
        Some((g.format, g.value.clone(), ttl_rem))
    }

    pub fn del(&self, key: &str) -> bool {
        self.map.remove(key).is_some()
    }

    pub fn len(&self) -> u64 { self.map.len() as u64 }

    pub fn approx_mem_bytes(&self) -> u64 {
        // Estimasi: sum size_bytes + overhead 64 bytes per entry (konstanta kasar)
        let mut sum = 0u64;
        for r in self.map.iter() {
            sum = sum.saturating_add(r.size_bytes as u64 + 64);
        }
        sum
    }
}
```

***

## D.12 `src/store/eviction.rs` (FINAL pressure eviction loop)

Menggunakan MemAvailable pressure untuk evict saat >85% hingga turun. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)

```rust
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::config::Config;
use crate::metrics::Metrics;
use crate::store::KvStore;
use crate::sys::meminfo::{read_meminfo, pressure};

pub struct Evictor {
    ring: Mutex<VecDeque<String>>,
    store: Arc<KvStore>,
    metrics: Arc<Metrics>,
    cfg: Config,
}

impl Evictor {
    pub fn new(store: Arc<KvStore>, metrics: Arc<Metrics>, cfg: Config) -> Self {
        Self { ring: Mutex::new(VecDeque::new()), store, metrics, cfg }
    }

    pub fn on_write(&self, key: &str) {
        self.ring.lock().push_back(key.to_string());
    }

    pub async fn run(self: Arc<Self>) {
        loop {
            sleep(Duration::from_millis(self.cfg.pressure_poll_ms)).await;

            let mi = match read_meminfo() { Ok(x) => x, Err(_) => continue };
            let p = pressure(mi);

            if p < self.cfg.pressure_hot { continue; }

            // HOT: evict until cool
            let mut evicted = 0u64;
            while let Ok(mi2) = read_meminfo() {
                if pressure(mi2) < self.cfg.pressure_cool { break; }
                if !self.evict_one() { break; }
                evicted += 1;
                if evicted % 32 == 0 {
                    // yield sedikit
                    tokio::task::yield_now().await;
                }
            }
            if evicted > 0 { self.metrics.inc_evictions(evicted); }
        }
    }

    fn evict_one(&self) -> bool {
        let key = self.ring.lock().pop_front();
        let Some(k) = key else { return false; };
        let _ = self.store.del(&k);
        true
    }
}
```

***

## D.13 `src/pubsub/mod.rs`

```rust
pub mod bus;
pub use bus::Bus;
```

## D.14 `src/pubsub/bus.rs` (tokio broadcast)

Tokio broadcast channel bounded + lagging behavior documented. [\[exchangetuts.com\]](https://exchangetuts.com/index.php/tcp-loopback-connection-vs-unix-domain-socket-performance-1639495325509681), [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

```rust
use dashmap::DashMap;
use tokio::sync::broadcast;
use localcached_proto::PushEvent;

pub struct Bus {
    topics: DashMap<String, broadcast::Sender<PushEvent>>,
    capacity: usize,
}

impl Bus {
    pub fn new(capacity: usize) -> Self {
        Self { topics: DashMap::new(), capacity }
    }

    pub fn topics_count(&self) -> u64 { self.topics.len() as u64 }

    fn get_or_create(&self, topic: &str) -> broadcast::Sender<PushEvent> {
        if let Some(s) = self.topics.get(topic) { return s.clone(); }
        let (tx, _rx) = broadcast::channel(self.capacity);
        self.topics.insert(topic.to_string(), tx.clone());
        tx
    }

    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<PushEvent> {
        self.get_or_create(topic).subscribe()
    }

    pub fn publish(&self, ev: PushEvent) -> usize {
        let tx = self.get_or_create(&ev.topic);
        tx.send(ev).map(|n| n).unwrap_or(0)
    }
}
```

***

## D.15 `src/server/mod.rs` (daemon runtime)

Tokio UDS listen + accept loop. [\[doc.rust-lang.org\]](https://doc.rust-lang.org/std/os/unix/net/struct.UnixListener.html), [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo)

```rust
use std::{path::Path, sync::Arc};
use tokio::net::UnixListener;

use crate::config::Config;
use crate::metrics::Metrics;
use crate::store::{KvStore, Evictor};
use crate::pubsub::Bus;

pub mod conn_kv;
pub mod conn_sub;

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    if Path::new(&cfg.socket_path).exists() {
        let _ = std::fs::remove_file(&cfg.socket_path);
    }

    let listener = UnixListener::bind(&cfg.socket_path)?;
    tracing::info!("localcached listening on {}", cfg.socket_path);

    let metrics = Arc::new(Metrics::new());
    let store = Arc::new(KvStore::default());
    let bus = Arc::new(Bus::new(cfg.pubsub_capacity));

    let evictor = Arc::new(Evictor::new(store.clone(), metrics.clone(), cfg.clone()));
    tokio::spawn(evictor.clone().run());

    loop {
        let (stream, _addr) = listener.accept().await?;
        let cfg2 = cfg.clone();
        let store2 = store.clone();
        let bus2 = bus.clone();
        let metrics2 = metrics.clone();
        let ev2 = evictor.clone();

        tokio::spawn(async move {
            if let Err(e) = conn_kv::handle(stream, cfg2, store2, bus2, metrics2, ev2).await {
                tracing::debug!("conn ended: {e:?}");
            }
        });
    }
}
```

***

## D.16 `src/server/conn_kv.rs` (FINAL CRUD + strict key + default publish ON)

Ini handler utama Conn-A. Jika opcode pertama SUBSCRIBE, kita “upgrade” ke conn\_sub (untuk kompatibilitas). UDS + framing + BytesMut pattern. [\[dev.to\]](https://dev.to/leapcell/optimizing-rust-performance-with-jemalloc-36lo), [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide)

```rust
use std::sync::Arc;
use bytes::{BytesMut, Buf, BufMut};
use tokio::net::UnixStream;
use tokio::io::AsyncWriteExt;

use localcached_proto::{
    Opcode, Status, ValueFormat, EventType,
    decode_set_payload, decode_key_only, encode_response, encode_stats_v1, StatsV1,
    topic_from_key, validate_key_3parts, PushEvent,
};

use crate::config::Config;
use crate::framing::read_frame;
use crate::metrics::Metrics;
use crate::store::KvStore;
use crate::pubsub::Bus;
use crate::store::Evictor;
use crate::time::now_ms;
use crate::sys::meminfo::{read_meminfo, pressure_bp};

pub async fn handle(
    mut stream: UnixStream,
    cfg: Config,
    store: Arc<KvStore>,
    bus: Arc<Bus>,
    metrics: Arc<Metrics>,
    evictor: Arc<Evictor>,
) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(8 * 1024);

    loop {
        let Some(frame) = read_frame(&mut stream, cfg.max_frame_bytes, &mut buf).await? else {
            return Ok(());
        };

        let mut rd = &frame[4..]; // [opcode][payload]
        if rd.remaining() < 1 {
            stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
            continue;
        }
        let opcode = rd.get_u8();
        let payload = &rd[..];

        match opcode {
            x if x == Opcode::Ping as u8 => {
                stream.write_all(&encode_response(Status::Ok, &[])).await?;
            }

            x if x == Opcode::Stats as u8 => {
                let mi = read_meminfo().ok();
                let bp = mi.map(pressure_bp).unwrap_or(0);

                let s = StatsV1 {
                    uptime_ms: metrics.uptime_ms(),
                    keys_count: store.len(),
                    approx_mem_bytes: store.approx_mem_bytes(),
                    evictions_total: metrics.evictions_total.load(std::sync::atomic::Ordering::Relaxed),
                    pubsub_topics: bus.topics_count(),
                    events_published_total: metrics.events_published_total.load(std::sync::atomic::Ordering::Relaxed),
                    events_lagged_total: metrics.events_lagged_total.load(std::sync::atomic::Ordering::Relaxed),
                    invalid_key_total: metrics.invalid_key_total.load(std::sync::atomic::Ordering::Relaxed),
                    mem_pressure_bp: bp,
                };

                let payload = encode_stats_v1(&s);
                stream.write_all(&encode_response(Status::Ok, &payload)).await?;
            }

            x if x == Opcode::Get as u8 => {
                let key = match decode_key_only(payload) {
                    Ok(k) => k,
                    Err(_) => {
                        stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
                        continue;
                    }
                };

                if validate_key_3parts(&key).is_err() {
                    metrics.inc_invalid_key();
                    stream.write_all(&encode_response(Status::ErrInvalidKeyFormat, &[])).await?;
                    continue;
                }

                let now = now_ms();
                if let Some((fmt, val, ttl_rem)) = store.get(&key, now) {
                    let mut out = BytesMut::with_capacity(1 + 4 + val.len() + 8);
                    out.put_u8(fmt as u8);
                    out.put_u32_le(val.len() as u32);
                    out.extend_from_slice(&val);
                    out.put_u64_le(ttl_rem);
                    stream.write_all(&encode_response(Status::Ok, &out)).await?;
                } else {
                    stream.write_all(&encode_response(Status::NotFound, &[])).await?;
                }
            }

            x if x == Opcode::Del as u8 => {
                let key = match decode_key_only(payload) {
                    Ok(k) => k,
                    Err(_) => {
                        stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
                        continue;
                    }
                };

                if validate_key_3parts(&key).is_err() {
                    metrics.inc_invalid_key();
                    stream.write_all(&encode_response(Status::ErrInvalidKeyFormat, &[])).await?;
                    continue;
                }

                let existed = store.del(&key);
                if existed {
                    // publish invalidate (MUST)
                    let topic = match topic_from_key(&key) {
                        Ok(t) => t,
                        Err(_) => {
                            metrics.inc_invalid_key();
                            stream.write_all(&encode_response(Status::ErrInvalidKeyFormat, &[])).await?;
                            continue;
                        }
                    };
                    let ev = PushEvent {
                        event_type: EventType::Invalidate,
                        topic,
                        key: key.clone(),
                        ts_ms: now_ms(),
                    };
                    let _ = bus.publish(ev);
                    metrics.inc_published();

                    stream.write_all(&encode_response(Status::Ok, &[])).await?;
                } else {
                    stream.write_all(&encode_response(Status::NotFound, &[])).await?;
                }
            }

            x if x == Opcode::Set as u8 => {
                let req = match decode_set_payload(payload) {
                    Ok(r) => r,
                    Err(e) => {
                        let st = match e {
                            localcached_proto::ProtoError::UnsupportedFormat => Status::ErrUnsupportedFormat,
                            _ => Status::ErrBadPayload
                        };
                        stream.write_all(&encode_response(st, &[])).await?;
                        continue;
                    }
                };

                if validate_key_3parts(&req.key).is_err() {
                    metrics.inc_invalid_key();
                    stream.write_all(&encode_response(Status::ErrInvalidKeyFormat, &[])).await?;
                    continue;
                }

                let now = now_ms();
                let expires_at = if req.ttl_ms == 0 { 0 } else { now.saturating_add(req.ttl_ms) };
                store.set(req.key.clone(), req.format, req.value, expires_at, now);
                evictor.on_write(&req.key);

                // Default publish ON (unless suppress_publish)
                if !req.suppress_publish {
                    let topic = match topic_from_key(&req.key) {
                        Ok(t) => t,
                        Err(_) => {
                            metrics.inc_invalid_key();
                            stream.write_all(&encode_response(Status::ErrInvalidKeyFormat, &[])).await?;
                            continue;
                        }
                    };
                    let ev = PushEvent {
                        event_type: EventType::TableChanged,
                        topic,
                        key: req.key.clone(),
                        ts_ms: now_ms(),
                    };
                    let _ = bus.publish(ev);
                    metrics.inc_published();
                }

                stream.write_all(&encode_response(Status::Ok, &[])).await?;
            }

            // Compat upgrade: if client mistakenly uses this connection for subscribe
            x if x == Opcode::Subscribe as u8 || x == Opcode::Unsubscribe as u8 => {
                // switch handler
                return crate::server::conn_sub::handle(stream, cfg, bus, metrics).await;
            }

            _ => {
                stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
            }
        }
    }
}
```

***

## D.17 `src/server/conn_sub.rs` (FINAL subscribe stream + lagged)

Menggunakan `broadcast::Receiver` per topic. Kita implement “multi-subscribe” dengan **task-per-subscription forward** ke mpsc agar simpel & robust, dan `Lagged` diterjemahkan jadi PUSH\_EVENT status `ERR_LAGGED`. Mekanisme lagging sudah didokumentasikan tokio broadcast. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

```rust
use std::collections::HashMap;
use std::sync::Arc;
use bytes::{BytesMut, Buf, BufMut};
use tokio::net::UnixStream;
use tokio::sync::{broadcast, mpsc, watch};
use tokio::io::AsyncWriteExt;

use localcached_proto::{
    Opcode, Status, encode_response, encode_subscribe_payload, decode_subscribe_payload,
    validate_topic, encode_push_event_payload,
};

use crate::config::Config;
use crate::framing::read_frame;
use crate::metrics::Metrics;
use crate::pubsub::Bus;

type StopTx = watch::Sender<bool>;

struct SubHandle {
    stop: StopTx,
}

pub async fn handle(
    mut stream: UnixStream,
    cfg: Config,
    bus: Arc<Bus>,
    metrics: Arc<Metrics>,
) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(8 * 1024);

    // all events forwarded to this single queue for this connection
    let (evt_tx, mut evt_rx) = mpsc::channel::<Result<localcached_proto::PushEvent, Status>>(1024);

    let mut subs: HashMap<String, SubHandle> = HashMap::new();

    loop {
        tokio::select! {
            // incoming control frames
            frame = read_frame(&mut stream, cfg.max_frame_bytes, &mut buf) => {
                let Some(frame) = frame? else { return Ok(()); };
                let mut rd = &frame[4..];
                if rd.remaining() < 1 { stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?; continue; }
                let opcode = rd.get_u8();
                let payload = &rd[..];

                match opcode {
                    x if x == Opcode::Subscribe as u8 => {
                        let topic = match decode_subscribe_payload(payload) {
                            Ok(t) => t,
                            Err(_) => { stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?; continue; }
                        };

                        if validate_topic(&topic).is_err() {
                            stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
                            continue;
                        }

                        if subs.contains_key(&topic) {
                            stream.write_all(&encode_response(Status::Ok, &[])).await?;
                            continue;
                        }

                        let mut rx = bus.subscribe(&topic);
                        let (stop_tx, mut stop_rx) = watch::channel(false);
                        let tx2 = evt_tx.clone();
                        tokio::spawn(async move {
                            loop {
                                tokio::select! {
                                    _ = stop_rx.changed() => {
                                        if *stop_rx.borrow() { break; }
                                    }
                                    ev = rx.recv() => {
                                        match ev {
                                            Ok(e) => { let _ = tx2.send(Ok(e)).await; }
                                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                                let _ = tx2.send(Err(Status::ErrLagged)).await;
                                            }
                                            Err(broadcast::error::RecvError::Closed) => break,
                                        }
                                    }
                                }
                            }
                        });

                        subs.insert(topic, SubHandle { stop: stop_tx });
                        stream.write_all(&encode_response(Status::Ok, &[])).await?;
                    }

                    x if x == Opcode::Unsubscribe as u8 => {
                        let topic = match decode_subscribe_payload(payload) {
                            Ok(t) => t,
                            Err(_) => { stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?; continue; }
                        };

                        if let Some(h) = subs.remove(&topic) {
                            let _ = h.stop.send(true);
                        }
                        stream.write_all(&encode_response(Status::Ok, &[])).await?;
                    }

                    _ => {
                        stream.write_all(&encode_response(Status::ErrBadPayload, &[])).await?;
                    }
                }
            }

            // outgoing events
            maybe = evt_rx.recv() => {
                let Some(item) = maybe else { return Ok(()); };
                match item {
                    Ok(ev) => {
                        // build PUSH_EVENT frame: [len][opcode=0x80][status=OK][payload]
                        let payload = encode_push_event_payload(&ev);
                        let len = 1 + 1 + payload.len();
                        let mut out = BytesMut::with_capacity(4 + len);
                        out.put_u32_le(len as u32);
                        out.put_u8(Opcode::PushEvent as u8);
                        out.put_u8(Status::Ok as u8);
                        out.extend_from_slice(&payload);
                        stream.write_all(&out).await?;
                    }
                    Err(Status::ErrLagged) => {
                        metrics.inc_lagged();
                        let len = 1 + 1;
                        let mut out = BytesMut::with_capacity(4 + len);
                        out.put_u32_le(len as u32);
                        out.put_u8(Opcode::PushEvent as u8);
                        out.put_u8(Status::ErrLagged as u8);
                        stream.write_all(&out).await?;
                    }
                    Err(st) => {
                        // other status not used here
                        stream.write_all(&encode_response(st, &[])).await?;
                    }
                }
            }
        }
    }
}
```

***

# E) Crate 3 — `localcached-client` (FINAL)

## E.1 `crates/localcached-client/Cargo.toml`

```toml
[package]
name = "localcached-client"
version = "0.1.0"
edition = "2021"

[dependencies]
localcached-proto = { path = "../localcached-proto" }
tokio = { version = "1", features = ["net", "io-util", "macros", "rt-multi-thread"] }
bytes = "1"
anyhow = "1"
```

## E.2 Struktur file

    localcached-client/src/
      lib.rs
      framing.rs
      kv_client.rs
      sub_client.rs

## E.3 `src/lib.rs`

```rust
pub mod framing;
pub mod kv_client;
pub mod sub_client;

pub use kv_client::KvClient;
pub use sub_client::{SubClient, SubEvent};
```

## E.4 `src/framing.rs` (client framing)

Sama prinsipnya: read frame pakai `BytesMut` untuk efisiensi. [\[generalist...rammer.com\]](https://generalistprogrammer.com/tutorials/bytes-rust-crate-guide), [\[docs.rs\]](https://docs.rs/nix/latest/nix/sys/socket/index.html)

```rust
use bytes::{BytesMut, Buf};
use tokio::io::AsyncReadExt;

pub async fn read_frame<R: AsyncReadExt + Unpin>(
    r: &mut R,
    max_frame: usize,
    buf: &mut BytesMut,
) -> std::io::Result<Option<BytesMut>> {
    while buf.len() < 4 {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    let len = (&buf[..4]).get_u32_le() as usize;
    if len == 0 || len > max_frame {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
    }
    let total = 4 + len;
    while buf.len() < total {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    Ok(Some(buf.split_to(total)))
}
```

## E.5 `src/kv_client.rs` (FINAL KV client API)

```rust
use anyhow::Result;
use bytes::{BytesMut, Buf};
use tokio::net::UnixStream;
use tokio::io::AsyncWriteExt;

use localcached_proto::{
    Opcode, Status, ValueFormat,
    encode_request, encode_set_payload, encode_key_only,
    decode_stats_v1, StatsV1, validate_key_3parts,
};

use crate::framing::read_frame;

pub struct KvClient {
    stream: UnixStream,
    max_frame: usize,
    buf: BytesMut,
}

impl KvClient {
    pub async fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream, max_frame: 8*1024*1024, buf: BytesMut::with_capacity(8*1024) })
    }

    pub fn with_max_frame(mut self, bytes: usize) -> Self {
        self.max_frame = bytes; self
    }

    pub async fn ping(&mut self) -> Result<()> {
        let req = encode_request(Opcode::Ping, &[]);
        self.stream.write_all(&req).await?;
        self.expect_ok().await?;
        Ok(())
    }

    pub async fn set_json(&mut self, key: &str, json_bytes: &[u8], ttl_ms: u64, suppress_publish: bool) -> Result<()> {
        validate_key_3parts(key).map_err(|_| anyhow::anyhow!("invalid key format"))?;
        let payload = encode_set_payload(ValueFormat::Json, suppress_publish, key, json_bytes, ttl_ms);
        let req = encode_request(Opcode::Set, &payload);
        self.stream.write_all(&req).await?;
        self.expect_ok().await?;
        Ok(())
    }

    pub async fn set_msgpack(&mut self, key: &str, msgpack_bytes: &[u8], ttl_ms: u64, suppress_publish: bool) -> Result<()> {
        validate_key_3parts(key).map_err(|_| anyhow::anyhow!("invalid key format"))?;
        let payload = encode_set_payload(ValueFormat::MsgPack, suppress_publish, key, msgpack_bytes, ttl_ms);
        let req = encode_request(Opcode::Set, &payload);
        self.stream.write_all(&req).await?;
        self.expect_ok().await?;
        Ok(())
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<(ValueFormat, Vec<u8>, u64)>> {
        validate_key_3parts(key).map_err(|_| anyhow::anyhow!("invalid key format"))?;
        let payload = encode_key_only(key);
        let req = encode_request(Opcode::Get, &payload);
        self.stream.write_all(&req).await?;

        let (status, payload) = self.read_response().await?;
        match status {
            Status::Ok => {
                let mut p = &payload[..];
                let fmt = p.get_u8();
                let format = match fmt {
                    1 => ValueFormat::Json,
                    2 => ValueFormat::MsgPack,
                    _ => return Err(anyhow::anyhow!("bad format from server")),
                };
                let vlen = p.get_u32_le() as usize;
                if p.remaining() < vlen + 8 { return Err(anyhow::anyhow!("bad payload")); }
                let val = p[..vlen].to_vec();
                p.advance(vlen);
                let ttl_rem = p.get_u64_le();
                Ok(Some((format, val, ttl_rem)))
            }
            Status::NotFound => Ok(None),
            _ => Err(anyhow::anyhow!("server error: {:?}", status)),
        }
    }

    pub async fn del(&mut self, key: &str) -> Result<bool> {
        validate_key_3parts(key).map_err(|_| anyhow::anyhow!("invalid key format"))?;
        let payload = encode_key_only(key);
        let req = encode_request(Opcode::Del, &payload);
        self.stream.write_all(&req).await?;
        let (status, _) = self.read_response().await?;
        match status {
            Status::Ok => Ok(true),
            Status::NotFound => Ok(false),
            _ => Err(anyhow::anyhow!("server error: {:?}", status)),
        }
    }

    pub async fn stats(&mut self) -> Result<StatsV1> {
        let req = encode_request(Opcode::Stats, &[]);
        self.stream.write_all(&req).await?;
        let (status, payload) = self.read_response().await?;
        if status != Status::Ok { return Err(anyhow::anyhow!("server error: {:?}", status)); }
        Ok(decode_stats_v1(&payload)?)
    }

    async fn expect_ok(&mut self) -> Result<()> {
        let (status, _) = self.read_response().await?;
        if status != Status::Ok {
            return Err(anyhow::anyhow!("server error: {:?}", status));
        }
        Ok(())
    }

    async fn read_response(&mut self) -> Result<(Status, Vec<u8>)> {
        let Some(frame) = read_frame(&mut self.stream, self.max_frame, &mut self.buf).await? else {
            return Err(anyhow::anyhow!("server closed"));
        };
        let mut rd = &frame[4..];
        let st = rd.get_u8();
        let status = unsafe { std::mem::transmute::<u8, Status>(st) };
        let payload = rd.to_vec();
        Ok((status, payload))
    }
}
```

***

## E.6 `src/sub_client.rs` (FINAL Pub/Sub client)

```rust
use anyhow::Result;
use bytes::{BytesMut, Buf};
use tokio::net::UnixStream;
use tokio::io::AsyncWriteExt;

use localcached_proto::{
    Opcode, Status, EventType,
    encode_request, encode_subscribe_payload, decode_push_event_payload, validate_topic,
};

use crate::framing::read_frame;

#[derive(Debug, Clone)]
pub struct SubEvent {
    pub event_type: EventType,
    pub topic: String,
    pub key: String,
    pub ts_ms: u64,
}

pub struct SubClient {
    stream: UnixStream,
    max_frame: usize,
    buf: BytesMut,
}

impl SubClient {
    pub async fn connect(socket_path: &str) -> Result<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        Ok(Self { stream, max_frame: 8*1024*1024, buf: BytesMut::with_capacity(8*1024) })
    }

    pub async fn subscribe(&mut self, topic: &str) -> Result<()> {
        validate_topic(topic).map_err(|_| anyhow::anyhow!("invalid topic format"))?;
        let payload = encode_subscribe_payload(topic);
        let req = encode_request(Opcode::Subscribe, &payload);
        self.stream.write_all(&req).await?;
        // response is normal response frame: [len][status][payload]
        let (status, _) = self.read_response().await?;
        if status != Status::Ok { return Err(anyhow::anyhow!("subscribe failed: {:?}", status)); }
        Ok(())
    }

    pub async fn unsubscribe(&mut self, topic: &str) -> Result<()> {
        let payload = encode_subscribe_payload(topic);
        let req = encode_request(Opcode::Unsubscribe, &payload);
        self.stream.write_all(&req).await?;
        let (status, _) = self.read_response().await?;
        if status != Status::Ok { return Err(anyhow::anyhow!("unsubscribe failed: {:?}", status)); }
        Ok(())
    }

    /// Wait next PUSH_EVENT; returns None on EOF
    pub async fn next_event(&mut self) -> Result<Option<SubEvent>> {
        let Some(frame) = read_frame(&mut self.stream, self.max_frame, &mut self.buf).await? else {
            return Ok(None);
        };
        let mut rd = &frame[4..];
        let opcode = rd.get_u8();
        if opcode != Opcode::PushEvent as u8 {
            // ignore unexpected frames
            return Ok(None);
        }
        let st = rd.get_u8();
        let status = unsafe { std::mem::transmute::<u8, Status>(st) };
        if status == Status::ErrLagged {
            // lagged signal only
            return Ok(Some(SubEvent{
                event_type: EventType::Invalidate, // placeholder; you can handle lagged separately
                topic: "".to_string(),
                key: "".to_string(),
                ts_ms: 0,
            }));
        }
        if status != Status::Ok { return Ok(None); }
        let ev = decode_push_event_payload(&rd)?;
        Ok(Some(SubEvent { event_type: ev.event_type, topic: ev.topic, key: ev.key, ts_ms: ev.ts_ms }))
    }

    async fn read_response(&mut self) -> Result<(Status, Vec<u8>)> {
        let Some(frame) = read_frame(&mut self.stream, self.max_frame, &mut self.buf).await? else {
            return Err(anyhow::anyhow!("server closed"));
        };
        let mut rd = &frame[4..];
        let st = rd.get_u8();
        let status = unsafe { std::mem::transmute::<u8, Status>(st) };
        Ok((status, rd.to_vec()))
    }
}
```

> `broadcast` bounded + `Lagged` adalah perilaku yang memang didesain untuk mengatasi slow receiver dengan membuang message lama dan menandai receiver sebagai lagged. [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)

***

# F) Cara Pakai (Client App) — FINAL

Contoh `main.rs` di aplikasi client (mis. microservice):

```rust
use localcached_client::{KvClient, SubClient};
use tokio::task;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let socket = "/run/localcached.sock";

    // Conn-B subscribe
    let mut sub = SubClient::connect(socket).await?;
    sub.subscribe("t:billing:invoice").await?;

    task::spawn(async move {
        let mut s = sub;
        while let Ok(Some(ev)) = s.next_event().await {
            if ev.topic.is_empty() {
                // lagged signal - handle by resync strategy if needed
                continue;
            }
            println!("EVENT {:?} topic={} key={} ts={}", ev.event_type, ev.topic, ev.key, ev.ts_ms);
            // On table_changed/upsert: you may choose to refresh local caches or mark dirty.
            // On invalidate: you can drop local derived caches related to key.
        }
    });

    // Conn-A CRUD
    let mut kv = KvClient::connect(socket).await?;
    kv.ping().await?;

    // Default publish ON: suppress_publish=false
    let key = "billing:invoice:12345";
    let json = br#"{"id":12345,"amount":1000}"#;
    kv.set_json(key, json, 0, false).await?;

    // Read back
    if let Some((_fmt, value, _ttl)) = kv.get(key).await? {
        println!("GET: {}", String::from_utf8_lossy(&value));
    }

    // Delete -> emits invalidate event
    kv.del(key).await?;

    // Stats
    let s = kv.stats().await?;
    println!("STATS: keys={}, mem={} bytes, pressure_bp={}", s.keys_count, s.approx_mem_bytes, s.mem_pressure_bp);

    Ok(())
}
```

***

# G) Build & Run (Server) — FINAL

## G.1 Build

```bash
cargo build -p localcached-server --release
```

## G.2 Run

```bash
sudo RUST_LOG=info \
  LOCALCACHED_SOCKET=/run/localcached.sock \
  ./target/release/localcached-server
```

***

# H) Deployment (systemd) — FINAL

`/etc/systemd/system/localcached.service`:

```ini
[Unit]
Description=LocalCached UDS KV + PubSub
After=network.target

[Service]
Type=simple
ExecStart=/opt/localcached/localcached-server
Restart=always
RestartSec=1
Environment=RUST_LOG=info
Environment=LOCALCACHED_SOCKET=/run/localcached.sock
Environment=LOCALCACHED_MAX_FRAME=8388608
Environment=LOCALCACHED_PRESSURE_HOT=0.85
Environment=LOCALCACHED_PRESSURE_COOL=0.80
Environment=LOCALCACHED_PUBSUB_CAP=256
Environment=LOCALCACHED_PRESSURE_POLL_MS=150

NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

Aktifkan:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now localcached
sudo systemctl status localcached
```

> `/run` lazim dipakai untuk socket file (umumnya tmpfs) sehingga cocok untuk UDS. UDS sendiri adalah mekanisme IPC lokal yang efisien (bukan network). [\[stackoverflow.com\]](https://stackoverflow.com/questions/9898961/is-there-a-way-to-get-the-uid-of-the-other-end-of-a-unix-socket-connection), [\[docs.rs\]](https://docs.rs/bytes/latest/bytes/struct.BytesMut.html)

***

# I) “Wajib” Checklist (FINAL, tidak ada opsi lagi)

1. **Key strict** `svc:table:pk` untuk GET/SET/DEL (server & client SDK enforce) — kalau tidak `ERR_INVALID_KEY_FORMAT`.
2. **Payload JSON/msgpack wajib serialized** (server tidak parse).
3. **SET default publish ON** (`suppress_publish=0`), hanya disable untuk bulk.
4. **DEL publish invalidate** jika sukses.
5. **PubSub bounded** via `tokio::sync::broadcast` + `Lagged` handling (`ERR_LAGGED`). [\[developers...redhat.com\]](https://developers.redhat.com/articles/2023/04/12/why-you-should-use-iouring-network-io), [\[man7.org\]](https://man7.org/linux/man-pages/man7/io_uring.7.html)
6. **Memory pressure** pakai `/proc/meminfo MemAvailable` sebagai estimasi available memory tanpa swap; trigger eviction saat hot. [\[man.archlinux.org\]](https://man.archlinux.org/man/proc_meminfo.5.en), [\[manpages.ubuntu.com\]](https://manpages.ubuntu.com/manpages/noble/man5/proc_meminfo.5.html)
7. **STATS** fixed binary v1 (stable contract).

***

* Spec sudah locked,
* server + client SDK sudah diturunkan dalam modul lengkap,
* semua handler konsisten dengan Final Spec,
* deployment juga jelas.
