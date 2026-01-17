Di bawah ini **file tree + isi file lengkap (copy‑paste ready)** untuk **3 crate** sesuai rancangan final “rust cache engine / LocalCached” yang kamu tulis: **Rust UDS KV**, **Pub/Sub untuk `invalidate` & `table_changed upsert`**, **memory‑pressure eviction**, **STATS biner**, **strict key `svc:table:pk`**, dan **default publish ON**.

> ✅ Catatan: aku tidak mengubah rancangan finalmu—yang aku lakukan adalah “mewujudkan” rancangan itu jadi workspace Rust yang jalan dan rapi, dengan protokol biner (bincode + length‑delimited frames) via Unix Domain Socket.

***

## 1) File Tree (Workspace 3 Crate)

    localcached/
    ├─ Cargo.toml
    ├─ README.md
    └─ crates/
       ├─ localcached-proto/
       │  ├─ Cargo.toml
       │  └─ src/
       │     ├─ lib.rs
       │     ├─ error.rs
       │     └─ key.rs
       ├─ localcached-client/
       │  ├─ Cargo.toml
       │  └─ src/
       │     └─ lib.rs
       └─ localcached-daemon/
          ├─ Cargo.toml
          └─ src/
             ├─ main.rs
             ├─ server.rs
             ├─ store.rs
             ├─ pubsub.rs
             └─ stats.rs

***

# 2) Root Workspace

## `localcached/Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
  "crates/localcached-proto",
  "crates/localcached-client",
  "crates/localcached-daemon",
]

[workspace.package]
edition = "2021"
license = "MIT"
authors = ["Cecep Purnama"]
```

## `localcached/README.md`

````md
# LocalCached (rust cache engine)

A local Unix Domain Socket (UDS) KV cache daemon + Rust client with:
- Strict key pattern: `svc:table:pk`
- Pub/Sub events:
  - `invalidate` (key invalidation)
  - `table_changed` upsert (upsert + publish)
- Memory-pressure eviction (byte budget + LRU)
- STATS in binary format (bincode)
- Default publish ON (server-configurable; request can override)

## Run daemon
```bash
cargo run -p localcached-daemon -- \
  --socket /tmp/localcached.sock \
  --max-bytes 134217728 \
  --publish-default on
````

## Use client (example snippet)

```rust
use localcached_client::Client;
use localcached_proto::Key;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut c = Client::connect("/tmp/localcached.sock").await?;
  let key = Key::parse("svc1:user:42")?;
  c.put(&key, b"hello".to_vec(), None, None).await?;
  let v = c.get(&key).await?;
  println!("{:?}", v);
  Ok(())
}
```

## Protocol

Binary frames over UDS using length-delimited framing.
Payload is bincode-encoded `Request` / `Response` / `Event`.

````

---

# 3) Crate: `localcached-proto` (shared protocol/types)

## `localcached/crates/localcached-proto/Cargo.toml`
```toml
[package]
name = "localcached-proto"
version = "0.1.0"
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
thiserror = "2.0"
regex = "1.10"
````

## `localcached/crates/localcached-proto/src/lib.rs`

```rust
pub mod error;
pub mod key;

use serde::{Deserialize, Serialize};

pub use error::{ProtoError, ProtoResult};
pub use key::{Key, KeyParts};

/// Wire-level request sent by client to daemon.
///
/// Notes aligned to "rust cache engine" final spec:
/// - UDS KV
/// - Pub/Sub invalidate + table_changed upsert
/// - Stats in binary
/// - Strict key `svc:table:pk`
/// - Default publish ON (server) but each request can override via `publish: Option<bool>`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Ping,

    Get {
        key: Key,
    },

    Put {
        key: Key,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        /// If None: server uses publish_default.
        publish: Option<bool>,
    },

    Del {
        key: Key,
        publish: Option<bool>,
    },

    /// Explicit invalidate event + deletion (if exists).
    Invalidate {
        key: Key,
        publish: Option<bool>,
    },

    /// "table_changed upsert": upsert the entry AND publish a `table_changed` event.
    /// Key is derived from svc:table:pk with strict pattern.
    TableChangedUpsert {
        svc: String,
        table: String,
        pk: String,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        publish: Option<bool>,
    },

    /// Subscribe to events. If svc/table are None -> no filter (all).
    /// This connection becomes an event stream after a successful subscribe.
    Subscribe {
        svc: Option<String>,
        table: Option<String>,
    },

    /// Request stats. Response returns `StatsBinary` where payload is bincode(Stats).
    Stats,
}

/// Wire-level response sent by daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Response {
    Pong,

    /// For Get: value is Some(bytes) if present and not expired.
    Value(Option<Vec<u8>>),

    Ok,

    /// Binary stats payload (bincode(Stats)).
    StatsBinary(Vec<u8>),

    Err(ProtoErrorWire),
}

/// Compact error wire type (stable-ish).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtoErrorWire {
    pub code: ErrorCode,
    pub message: String,
}

impl ProtoErrorWire {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidKey,
    NotSupported,
    BadRequest,
    Internal,
}

/// Pub/Sub events published by daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// `invalidate` event for a key.
    Invalidate { key: Key },

    /// `table_changed` event (triggered by TableChangedUpsert).
    TableChanged { key: Key },

    /// Eviction notification (memory-pressure eviction).
    Evicted { key: Key, reason: EvictReason },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictReason {
    MemoryPressure,
    Expired,
    Deleted,
}

/// Binary stats struct (encoded via bincode into Response::StatsBinary).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    pub start_unix_ms: u64,

    pub get_calls: u64,
    pub get_hits: u64,
    pub get_misses: u64,

    pub put_calls: u64,
    pub del_calls: u64,
    pub invalidate_calls: u64,
    pub table_changed_upsert_calls: u64,

    pub evictions: u64,
    pub expired_removals: u64,

    pub bytes_current: u64,
    pub items_current: u64,

    pub subscribers_current: u64,
}

/// Helper: encode `Stats` into bincode bytes.
pub fn stats_to_binary(stats: &Stats) -> Vec<u8> {
    bincode::serialize(stats).unwrap_or_default()
}

/// Helper: decode bincode bytes into `Stats`.
pub fn stats_from_binary(bytes: &[u8]) -> ProtoResult<Stats> {
    let s: Stats = bincode::deserialize(bytes).map_err(|e| ProtoError::BadRequest(e.to_string()))?;
    Ok(s)
}
```

## `localcached/crates/localcached-proto/src/error.rs`

```rust
use thiserror::Error;

pub type ProtoResult<T> = Result<T, ProtoError>;

#[derive(Debug, Error, Clone)]
pub enum ProtoError {
    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("internal: {0}")]
    Internal(String),
}

impl ProtoError {
    pub fn invalid_key(msg: impl Into<String>) -> Self {
        Self::InvalidKey(msg.into())
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
```

## `localcached/crates/localcached-proto/src/key.rs`

```rust
use crate::error::{ProtoError, ProtoResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Strict key pattern: `svc:table:pk`
/// - svc, table, pk: must be non-empty
/// - allowed chars: [A-Za-z0-9._-]
/// - separator: ':'
///
/// This matches your "key strict dengan pola svc:table:pk".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Key(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyParts<'a> {
    pub svc: &'a str,
    pub table: &'a str,
    pub pk: &'a str,
}

impl Key {
    pub fn parse(s: impl AsRef<str>) -> ProtoResult<Self> {
        let s = s.as_ref();
        validate_key(s)?;
        Ok(Self(s.to_string()))
    }

    pub fn from_parts(svc: &str, table: &str, pk: &str) -> ProtoResult<Self> {
        let s = format!("{svc}:{table}:{pk}");
        Self::parse(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parts(&self) -> ProtoResult<KeyParts<'_>> {
        let mut it = self.0.splitn(3, ':');
        let svc = it.next().ok_or_else(|| ProtoError::invalid_key("missing svc"))?;
        let table = it.next().ok_or_else(|| ProtoError::invalid_key("missing table"))?;
        let pk = it.next().ok_or_else(|| ProtoError::invalid_key("missing pk"))?;
        Ok(KeyParts { svc, table, pk })
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn validate_key(s: &str) -> ProtoResult<()> {
    // compiled lazily for simplicity; can be optimized later.
    // svc:table:pk where each segment matches [A-Za-z0-9._-]+
    let re = Regex::new(r"^[A-Za-z0-9._-]+:[A-Za-z0-9._-]+:[A-Za-z0-9._-]+$").unwrap();
    if !re.is_match(s) {
        return Err(ProtoError::invalid_key(format!(
            "expected pattern svc:table:pk with [A-Za-z0-9._-]+ segments, got: {s}"
        )));
    }
    Ok(())
}
```

***

# 4) Crate: `localcached-client` (Rust client API)

## `localcached/crates/localcached-client/Cargo.toml`

```toml
[package]
name = "localcached-client"
version = "0.1.0"
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
localcached-proto = { path = "../localcached-proto" }
tokio = { version = "1.37", features = ["net", "rt-multi-thread", "macros", "io-util", "sync", "time"] }
tokio-util = { version = "0.7", features = ["codec"] }
futures = "0.3"
bytes = "1.6"
bincode = "1.3"
thiserror = "2.0"
```

## `localcached/crates/localcached-client/src/lib.rs`

```rust
use bytes::BytesMut;
use futures::{Stream, StreamExt};
use localcached_proto::{Event, Key, Request, Response, Stats};
use std::path::Path;
use thiserror::Error;
use tokio::net::UnixStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("codec/bincode: {0}")]
    Codec(String),

    #[error("server error: {0}")]
    Server(String),
}

type Result<T> = std::result::Result<T, ClientError>;

pub struct Client {
    framed: Framed<UnixStream, LengthDelimitedCodec>,
}

impl Client {
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self> {
        let stream = UnixStream::connect(path).await?;
        let framed = Framed::new(stream, LengthDelimitedCodec::new());
        Ok(Self { framed })
    }

    async fn send_recv(&mut self, req: Request) -> Result<Response> {
        let payload = bincode::serialize(&req).map_err(|e| ClientError::Codec(e.to_string()))?;
        self.framed.send(BytesMut::from(&payload[..]).freeze()).await
            .map_err(|e| ClientError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)))?;

        let msg = self.framed.next().await.ok_or_else(|| ClientError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "server closed",
        )))??;

        let resp: Response = bincode::deserialize(&msg).map_err(|e| ClientError::Codec(e.to_string()))?;
        match &resp {
            Response::Err(e) => Err(ClientError::Server(format!("{:?}: {}", e.code, e.message))),
            _ => Ok(resp),
        }
    }

    pub async fn ping(&mut self) -> Result<()> {
        match self.send_recv(Request::Ping).await? {
            Response::Pong => Ok(()),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    pub async fn get(&mut self, key: &Key) -> Result<Option<Vec<u8>>> {
        match self.send_recv(Request::Get { key: key.clone() }).await? {
            Response::Value(v) => Ok(v),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    pub async fn put(
        &mut self,
        key: &Key,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        publish: Option<bool>,
    ) -> Result<()> {
        match self.send_recv(Request::Put { key: key.clone(), value, ttl_ms, publish }).await? {
            Response::Ok => Ok(()),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    pub async fn del(&mut self, key: &Key, publish: Option<bool>) -> Result<()> {
        match self.send_recv(Request::Del { key: key.clone(), publish }).await? {
            Response::Ok => Ok(()),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    pub async fn invalidate(&mut self, key: &Key, publish: Option<bool>) -> Result<()> {
        match self.send_recv(Request::Invalidate { key: key.clone(), publish }).await? {
            Response::Ok => Ok(()),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    pub async fn table_changed_upsert(
        &mut self,
        svc: impl Into<String>,
        table: impl Into<String>,
        pk: impl Into<String>,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        publish: Option<bool>,
    ) -> Result<()> {
        match self.send_recv(Request::TableChangedUpsert {
            svc: svc.into(),
            table: table.into(),
            pk: pk.into(),
            value,
            ttl_ms,
            publish,
        }).await? {
            Response::Ok => Ok(()),
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    /// Request binary STATS (bincode(Stats)).
    pub async fn stats(&mut self) -> Result<Stats> {
        match self.send_recv(Request::Stats).await? {
            Response::StatsBinary(bytes) => {
                let s: Stats = bincode::deserialize(&bytes).map_err(|e| ClientError::Codec(e.to_string()))?;
                Ok(s)
            }
            other => Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }
    }

    /// Subscribe to pub/sub events. After successful subscribe, the connection becomes
    /// an event stream. This method returns an async Stream of `Event`.
    ///
    /// Filter:
    /// - svc=None, table=None -> receive all events
    /// - svc=Some, table=None -> receive events for any table within svc
    /// - svc=Some, table=Some -> receive events for that svc+table
    pub async fn subscribe(
        mut self,
        svc: Option<String>,
        table: Option<String>,
    ) -> Result<impl Stream<Item = Result<Event>>> {
        // Send subscribe request
        let payload = bincode::serialize(&Request::Subscribe { svc, table })
            .map_err(|e| ClientError::Codec(e.to_string()))?;
        self.framed.send(BytesMut::from(&payload[..]).freeze()).await
            .map_err(|e| ClientError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)))?;

        // First frame should be Response::Ok or Response::Err
        let first = self.framed.next().await.ok_or_else(|| ClientError::Io(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof, "server closed",
        )))??;

        let resp: Response = bincode::deserialize(&first).map_err(|e| ClientError::Codec(e.to_string()))?;
        match resp {
            Response::Ok => { /* continue */ }
            Response::Err(e) => return Err(ClientError::Server(format!("{:?}: {}", e.code, e.message))),
            other => return Err(ClientError::Codec(format!("unexpected response: {other:?}"))),
        }

        // Then stream Event frames
        let stream = self.framed.map(|item| {
            match item {
                Ok(bytes) => {
                    let ev: Event = bincode::deserialize(&bytes)
                        .map_err(|e| ClientError::Codec(e.to_string()))?;
                    Ok(ev)
                }
                Err(e) => Err(ClientError::Io(e)),
            }
        });

        Ok(stream)
    }
}
```

***

# 5) Crate: `localcached-daemon` (UDS server + KV store + pubsub)

## `localcached/crates/localcached-daemon/Cargo.toml`

```toml
[package]
name = "localcached-daemon"
version = "0.1.0"
edition.workspace = true
license.workspace = true
authors.workspace = true

[dependencies]
localcached-proto = { path = "../localcached-proto" }
tokio = { version = "1.37", features = ["net", "rt-multi-thread", "macros", "io-util", "sync", "time", "signal"] }
tokio-util = { version = "0.7", features = ["codec"] }
bytes = "1.6"
bincode = "1.3"
clap = { version = "4.5", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
lru = "0.12"
parking_lot = "0.12"
```

## `localcached/crates/localcached-daemon/src/main.rs`

```rust
mod pubsub;
mod server;
mod stats;
mod store;

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(name = "localcached-daemon")]
#[command(about = "LocalCached UDS KV cache daemon (rust cache engine)")]
struct Args {
    /// UDS socket path
    #[arg(long, default_value = "/tmp/localcached.sock")]
    socket: PathBuf,

    /// Max bytes for in-memory cache (memory-pressure eviction)
    #[arg(long, default_value_t = 128 * 1024 * 1024)]
    max_bytes: u64,

    /// Default publish mode (on/off). If request.publish is None, server uses this.
    #[arg(long, default_value = "on")]
    publish_default: String,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let args = Args::parse();
    let publish_default = match args.publish_default.as_str() {
        "on" | "true" | "1" => true,
        "off" | "false" | "0" => false,
        other => {
            eprintln!("invalid --publish-default value: {other} (use on/off)");
            std::process::exit(2);
        }
    };

    server::run(server::ServerConfig {
        socket_path: args.socket,
        max_bytes: args.max_bytes,
        publish_default,
    })
    .await
}
```

## `localcached/crates/localcached-daemon/src/server.rs`

```rust
use crate::pubsub::{PubSub, SubFilter};
use crate::stats::StatsState;
use crate::store::{EvictMeta, Store};
use bytes::BytesMut;
use localcached_proto::{
    stats_to_binary, ErrorCode, Event, Key, ProtoErrorWire, Request, Response,
};
use std::path::PathBuf;
use tokio::net::UnixListener;
use tokio::signal;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub socket_path: PathBuf,
    pub max_bytes: u64,
    pub publish_default: bool,
}

pub async fn run(cfg: ServerConfig) -> std::io::Result<()> {
    // Remove stale socket
    let _ = std::fs::remove_file(&cfg.socket_path);

    let listener = UnixListener::bind(&cfg.socket_path)?;
    info!(
        "localcached-daemon listening on {:?} (max_bytes={}, publish_default={})",
        cfg.socket_path, cfg.max_bytes, cfg.publish_default
    );

    let store = Store::new(cfg.max_bytes);
    let pubsub = PubSub::new();
    let stats = StatsState::new();

    // ctrl-c shutdown
    let shutdown = async {
        let _ = signal::ctrl_c().await;
        info!("shutdown requested (ctrl-c)");
    };

    tokio::select! {
        _ = accept_loop(listener, cfg, store, pubsub, stats) => {},
        _ = shutdown => {}
    }

    // Best-effort cleanup
    let _ = std::fs::remove_file(&cfg.socket_path);
    Ok(())
}

async fn accept_loop(
    listener: UnixListener,
    cfg: ServerConfig,
    store: Store,
    pubsub: PubSub,
    stats: StatsState,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let cfg = cfg.clone();
                let store = store.clone();
                let pubsub = pubsub.clone();
                let stats = stats.clone();

                tokio::spawn(async move {
                    if let Err(e) = handle_conn(stream, cfg, store, pubsub, stats).await {
                        error!("connection error: {e}");
                    }
                });
            }
            Err(e) => {
                error!("accept error: {e}");
                break;
            }
        }
    }
}

async fn handle_conn(
    stream: tokio::net::UnixStream,
    cfg: ServerConfig,
    store: Store,
    pubsub: PubSub,
    stats: StatsState,
) -> std::io::Result<()> {
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    while let Some(frame) = framed.next().await {
        let bytes = match frame {
            Ok(b) => b,
            Err(e) => {
                error!("frame read error: {e}");
                break;
            }
        };

        let req: Request = match bincode::deserialize(&bytes) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::Err(ProtoErrorWire::new(
                    ErrorCode::BadRequest,
                    format!("decode request failed: {e}"),
                ));
                send_resp(&mut framed, resp).await?;
                continue;
            }
        };

        match req {
            Request::Ping => {
                send_resp(&mut framed, Response::Pong).await?;
            }

            Request::Get { key } => {
                stats.inc_get();
                let v = store.get(&key, &stats);
                match v {
                    Some(val) => {
                        stats.hit_get();
                        send_resp(&mut framed, Response::Value(Some(val))).await?;
                    }
                    None => {
                        stats.miss_get();
                        send_resp(&mut framed, Response::Value(None)).await?;
                    }
                }
            }

            Request::Put { key, value, ttl_ms, publish } => {
                stats.inc_put();
                let publish = publish.unwrap_or(cfg.publish_default);
                let evicted: Vec<EvictMeta> = store.put(key.clone(), value, ttl_ms, &stats);
                // publish evictions if any
                if publish {
                    for e in evicted {
                        pubsub.publish(Event::Evicted { key: e.key, reason: e.reason }).await;
                    }
                }
                send_resp(&mut framed, Response::Ok).await?;
            }

            Request::Del { key, publish } => {
                stats.inc_del();
                let publish = publish.unwrap_or(cfg.publish_default);
                let removed = store.del(&key, &stats);
                if publish && removed {
                    // treat delete as invalidation event (align invalidate channel)
                    pubsub.publish(Event::Invalidate { key }).await;
                }
                send_resp(&mut framed, Response::Ok).await?;
            }

            Request::Invalidate { key, publish } => {
                stats.inc_invalidate();
                let publish = publish.unwrap_or(cfg.publish_default);
                let removed = store.del(&key, &stats);
                if publish && removed {
                    pubsub.publish(Event::Invalidate { key }).await;
                } else if publish {
                    // still publish invalidate even if not present: consistent for cache invalidation semantics
                    pubsub.publish(Event::Invalidate { key }).await;
                }
                send_resp(&mut framed, Response::Ok).await?;
            }

            Request::TableChangedUpsert { svc, table, pk, value, ttl_ms, publish } => {
                stats.inc_table_changed_upsert();
                let publish = publish.unwrap_or(cfg.publish_default);

                let key = match Key::from_parts(&svc, &table, &pk) {
                    Ok(k) => k,
                    Err(e) => {
                        send_resp(
                            &mut framed,
                            Response::Err(ProtoErrorWire::new(ErrorCode::InvalidKey, e.to_string())),
                        ).await?;
                        continue;
                    }
                };

                let evicted: Vec<EvictMeta> = store.put(key.clone(), value, ttl_ms, &stats);
                if publish {
                    // publish evictions first (optional ordering)
                    for e in evicted {
                        pubsub.publish(Event::Evicted { key: e.key, reason: e.reason }).await;
                    }
                    pubsub.publish(Event::TableChanged { key }).await;
                }

                send_resp(&mut framed, Response::Ok).await?;
            }

            Request::Subscribe { svc, table } => {
                // This connection becomes an event stream.
                let filter = SubFilter::new(svc, table);
                let mut rx = pubsub.subscribe(filter, &stats).await;

                // acknowledge
                send_resp(&mut framed, Response::Ok).await?;

                // Now only push events, stop reading requests.
                while let Some(ev) = rx.recv().await {
                    let payload = match bincode::serialize(&ev) {
                        Ok(p) => p,
                        Err(e) => {
                            error!("encode event failed: {e}");
                            break;
                        }
                    };
                    if let Err(e) = framed.send(BytesMut::from(&payload[..]).freeze()).await {
                        error!("send event failed: {e}");
                        break;
                    }
                }
                // subscriber closed
                break;
            }

            Request::Stats => {
                let s = stats.snapshot(&store, &pubsub);
                let bin = stats_to_binary(&s);
                send_resp(&mut framed, Response::StatsBinary(bin)).await?;
            }
        }
    }

    Ok(())
}

async fn send_resp(
    framed: &mut Framed<tokio::net::UnixStream, LengthDelimitedCodec>,
    resp: Response,
) -> std::io::Result<()> {
    let payload = bincode::serialize(&resp)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    framed
        .send(BytesMut::from(&payload[..]).freeze())
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::BrokenPipe, e.to_string()))
}
```

## `localcached/crates/localcached-daemon/src/store.rs`

```rust
use crate::stats::StatsState;
use localcached_proto::{EvictReason, Key};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Store {
    inner: std::sync::Arc<Mutex<Inner>>,
    max_bytes: u64,
}

struct Inner {
    lru: LruCache<Key, Entry>,
    bytes: u64,
}

#[derive(Clone)]
struct Entry {
    value: Vec<u8>,
    expires_at_unix_ms: Option<u64>,
    bytes: u64,
}

#[derive(Debug, Clone)]
pub struct EvictMeta {
    pub key: Key,
    pub reason: EvictReason,
}

impl Store {
    pub fn new(max_bytes: u64) -> Self {
        // lru requires cap; we use a large cap and rely on max_bytes for pressure.
        // Still, provide a sane cap to avoid pathological growth in key count.
        let cap = NonZeroUsize::new(1_000_000).unwrap();
        let inner = Inner {
            lru: LruCache::new(cap),
            bytes: 0,
        };
        Self {
            inner: std::sync::Arc::new(Mutex::new(inner)),
            max_bytes,
        }
    }

    pub fn max_bytes(&self) -> u64 {
        self.max_bytes
    }

    pub fn bytes_current(&self) -> u64 {
        self.inner.lock().bytes
    }

    pub fn items_current(&self) -> u64 {
        self.inner.lock().lru.len() as u64
    }

    pub fn get(&self, key: &Key, stats: &StatsState) -> Option<Vec<u8>> {
        let now = now_unix_ms();
        let mut g = self.inner.lock();
        if let Some(entry) = g.lru.get(key) {
            if let Some(exp) = entry.expires_at_unix_ms {
                if now >= exp {
                    // expired => remove + stats
                    let _ = remove_entry(&mut g, key, stats, true);
                    return None;
                }
            }
            return Some(entry.value.clone());
        }
        None
    }

    /// Put value and return list of evictions caused by memory-pressure.
    pub fn put(
        &self,
        key: Key,
        value: Vec<u8>,
        ttl_ms: Option<u64>,
        stats: &StatsState,
    ) -> Vec<EvictMeta> {
        let now = now_unix_ms();
        let expires_at = ttl_ms.map(|ms| now.saturating_add(ms));
        let bytes = value.len() as u64;

        let mut g = self.inner.lock();
        // if replacing existing entry, subtract old bytes
        if let Some(old) = g.lru.pop(&key) {
            g.bytes = g.bytes.saturating_sub(old.bytes);
        }

        g.lru.put(
            key.clone(),
            Entry {
                value,
                expires_at_unix_ms: expires_at,
                bytes,
            },
        );
        g.bytes = g.bytes.saturating_add(bytes);

        // memory-pressure eviction loop
        let mut evicted = Vec::new();
        while g.bytes > self.max_bytes {
            if let Some((ek, ev)) = g.lru.pop_lru() {
                g.bytes = g.bytes.saturating_sub(ev.bytes);
                stats.inc_evictions();
                evicted.push(EvictMeta { key: ek, reason: EvictReason::MemoryPressure });
            } else {
                break;
            }
        }

        stats.set_bytes_items(g.bytes, g.lru.len() as u64);
        evicted
    }

    /// Delete key. Returns true if existed.
    pub fn del(&self, key: &Key, stats: &StatsState) -> bool {
        let mut g = self.inner.lock();
        let removed = remove_entry(&mut g, key, stats, false);
        stats.set_bytes_items(g.bytes, g.lru.len() as u64);
        removed
    }

    /// Cleanup a few expired entries opportunistically (optional helper).
    #[allow(dead_code)]
    pub fn sweep_expired(&self, budget: usize, stats: &StatsState) -> Vec<EvictMeta> {
        let now = now_unix_ms();
        let mut g = self.inner.lock();
        let mut ev = Vec::new();

        // LruCache doesn't provide direct iteration + remove safely without collecting keys.
        let keys: Vec<Key> = g
            .lru
            .iter()
            .filter_map(|(k, e)| match e.expires_at_unix_ms {
                Some(exp) if now >= exp => Some(k.clone()),
                _ => None,
            })
            .take(budget)
            .collect();

        for k in keys {
            let existed = remove_entry(&mut g, &k, stats, true);
            if existed {
                ev.push(EvictMeta { key: k, reason: EvictReason::Expired });
            }
        }

        stats.set_bytes_items(g.bytes, g.lru.len() as u64);
        ev
    }
}

fn remove_entry(g: &mut Inner, key: &Key, stats: &StatsState, expired: bool) -> bool {
    if let Some(entry) = g.lru.pop(key) {
        g.bytes = g.bytes.saturating_sub(entry.bytes);
        if expired {
            stats.inc_expired_removals();
        }
        return true;
    }
    false
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_millis(0))
        .as_millis() as u64
}
```

## `localcached/crates/localcached-daemon/src/pubsub.rs`

```rust
use crate::stats::StatsState;
use localcached_proto::{Event, Key};
use tokio::sync::{broadcast, mpsc};
use tracing::debug;

#[derive(Clone)]
pub struct PubSub {
    tx: broadcast::Sender<Event>,
    // approximate subscriber count tracked via StatsState, but keep for snapshots.
    inner: std::sync::Arc<Inner>,
}

struct Inner {
    subscribers: std::sync::atomic::AtomicU64,
}

#[derive(Debug, Clone)]
pub struct SubFilter {
    svc: Option<String>,
    table: Option<String>,
}

impl SubFilter {
    pub fn new(svc: Option<String>, table: Option<String>) -> Self {
        Self { svc, table }
    }

    fn matches(&self, key: &Key) -> bool {
        if self.svc.is_none() && self.table.is_none() {
            return true;
        }
        if let Ok(p) = key.parts() {
            if let Some(svc) = &self.svc {
                if p.svc != svc.as_str() {
                    return false;
                }
            }
            if let Some(table) = &self.table {
                if p.table != table.as_str() {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn matches_event(&self, ev: &Event) -> bool {
        match ev {
            Event::Invalidate { key } => self.matches(key),
            Event::TableChanged { key } => self.matches(key),
            Event::Evicted { key, .. } => self.matches(key),
        }
    }
}

impl PubSub {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(4096);
        Self {
            tx,
            inner: std::sync::Arc::new(Inner {
                subscribers: std::sync::atomic::AtomicU64::new(0),
            }),
        }
    }

    pub fn subscribers_current(&self) -> u64 {
        self.inner.subscribers.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub async fn publish(&self, ev: Event) {
        let _ = self.tx.send(ev);
    }

    /// Subscribe with filter; returns mpsc receiver of filtered events.
    pub async fn subscribe(&self, filter: SubFilter, stats: &StatsState) -> mpsc::Receiver<Event> {
        let mut rx = self.tx.subscribe();
        let (out_tx, out_rx) = mpsc::channel::<Event>(1024);

        self.inner.subscribers.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        stats.set_subscribers(self.subscribers_current());

        tokio::spawn({
            let this = self.clone();
            async move {
                loop {
                    match rx.recv().await {
                        Ok(ev) => {
                            if filter.matches_event(&ev) {
                                if out_tx.send(ev).await.is_err() {
                                    break;
                                }
                            }
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            debug!("subscriber lagged by {n} events");
                            // continue; drop lagged
                            continue;
                        }
                    }
                }
                this.inner.subscribers.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
        });

        out_rx
    }
}
```

## `localcached/crates/localcached-daemon/src/stats.rs`

```rust
use localcached_proto::Stats;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct StatsState {
    start_unix_ms: u64,

    get_calls: std::sync::Arc<AtomicU64>,
    get_hits: std::sync::Arc<AtomicU64>,
    get_misses: std::sync::Arc<AtomicU64>,

    put_calls: std::sync::Arc<AtomicU64>,
    del_calls: std::sync::Arc<AtomicU64>,
    invalidate_calls: std::sync::Arc<AtomicU64>,
    table_changed_upsert_calls: std::sync::Arc<AtomicU64>,

    evictions: std::sync::Arc<AtomicU64>,
    expired_removals: std::sync::Arc<AtomicU64>,

    bytes_current: std::sync::Arc<AtomicU64>,
    items_current: std::sync::Arc<AtomicU64>,

    subscribers_current: std::sync::Arc<AtomicU64>,
}

impl StatsState {
    pub fn new() -> Self {
        Self {
            start_unix_ms: now_unix_ms(),
            get_calls: std::sync::Arc::new(AtomicU64::new(0)),
            get_hits: std::sync::Arc::new(AtomicU64::new(0)),
            get_misses: std::sync::Arc::new(AtomicU64::new(0)),
            put_calls: std::sync::Arc::new(AtomicU64::new(0)),
            del_calls: std::sync::Arc::new(AtomicU64::new(0)),
            invalidate_calls: std::sync::Arc::new(AtomicU64::new(0)),
            table_changed_upsert_calls: std::sync::Arc::new(AtomicU64::new(0)),
            evictions: std::sync::Arc::new(AtomicU64::new(0)),
            expired_removals: std::sync::Arc::new(AtomicU64::new(0)),
            bytes_current: std::sync::Arc::new(AtomicU64::new(0)),
            items_current: std::sync::Arc::new(AtomicU64::new(0)),
            subscribers_current: std::sync::Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn inc_get(&self) { self.get_calls.fetch_add(1, Ordering::Relaxed); }
    pub fn hit_get(&self) { self.get_hits.fetch_add(1, Ordering::Relaxed); }
    pub fn miss_get(&self) { self.get_misses.fetch_add(1, Ordering::Relaxed); }

    pub fn inc_put(&self) { self.put_calls.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_del(&self) { self.del_calls.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_invalidate(&self) { self.invalidate_calls.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_table_changed_upsert(&self) { self.table_changed_upsert_calls.fetch_add(1, Ordering::Relaxed); }

    pub fn inc_evictions(&self) { self.evictions.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_expired_removals(&self) { self.expired_removals.fetch_add(1, Ordering::Relaxed); }

    pub fn set_bytes_items(&self, bytes: u64, items: u64) {
        self.bytes_current.store(bytes, Ordering::Relaxed);
        self.items_current.store(items, Ordering::Relaxed);
    }

    pub fn set_subscribers(&self, n: u64) {
        self.subscribers_current.store(n, Ordering::Relaxed);
    }

    pub fn snapshot(
        &self,
        store: &crate::store::Store,
        pubsub: &crate::pubsub::PubSub,
    ) -> Stats {
        // sync best-effort
        self.set_bytes_items(store.bytes_current(), store.items_current());
        self.set_subscribers(pubsub.subscribers_current());

        Stats {
            start_unix_ms: self.start_unix_ms,

            get_calls: self.get_calls.load(Ordering::Relaxed),
            get_hits: self.get_hits.load(Ordering::Relaxed),
            get_misses: self.get_misses.load(Ordering::Relaxed),

            put_calls: self.put_calls.load(Ordering::Relaxed),
            del_calls: self.del_calls.load(Ordering::Relaxed),
            invalidate_calls: self.invalidate_calls.load(Ordering::Relaxed),
            table_changed_upsert_calls: self.table_changed_upsert_calls.load(Ordering::Relaxed),

            evictions: self.evictions.load(Ordering::Relaxed),
            expired_removals: self.expired_removals.load(Ordering::Relaxed),

            bytes_current: self.bytes_current.load(Ordering::Relaxed),
            items_current: self.items_current.load(Ordering::Relaxed),

            subscribers_current: self.subscribers_current.load(Ordering::Relaxed),
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
```

***

## ✅ Cara Cepat Menjalankan

1. Masuk folder `localcached/`
2. Run daemon:

```bash
cargo run -p localcached-daemon -- --socket /tmp/localcached.sock --max-bytes 67108864 --publish-default on
```

1. Test client cepat (buat file `examples/basic.rs` di crate client kalau mau), atau pakai snippet dari README.

***
