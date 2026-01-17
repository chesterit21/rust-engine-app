pub mod conn_kv;
pub mod conn_sub;

use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Semaphore;
use std::sync::Arc;
use crate::config::{Config, RuntimeConfig};
use crate::store::{KvStore, Evictor};
use crate::pubsub::PubSubBus;
use crate::metrics::Metrics;
use std::fs;
use std::path::Path;

pub struct Context {
    pub kv: Arc<KvStore>,
    pub pubsub: Arc<PubSubBus>,
    pub evictor: Arc<Evictor>,
    pub metrics: Arc<Metrics>,
    pub cfg: Config,
    pub runtime_cfg: Arc<RuntimeConfig>,
    pub op_semaphore: Arc<Semaphore>,  // Backpressure: limits concurrent operations
}

pub async fn run(cfg: Config) -> anyhow::Result<()> {
    tracing::info!("Starting localcached server at {}", cfg.socket_path);
    tracing::info!("Max concurrent ops: {}", cfg.max_concurrent_ops);

    // Remove old socket
    if Path::new(&cfg.socket_path).exists() {
        fs::remove_file(&cfg.socket_path)?;
    }
    
    // Write PID file
    let pid = std::process::id();
    fs::write(&cfg.pid_path, pid.to_string())
        .map_err(|e| anyhow::anyhow!("Failed to write PID file to {}: {}", cfg.pid_path, e))?;

    let listener = UnixListener::bind(&cfg.socket_path)?;
    
    // Set permission? (Optional, default is usually srwxr-xr-x or similar depending on umask)
    // use std::os::unix::fs::PermissionsExt;
    // fs::set_permissions(&cfg.socket_path, fs::Permissions::from_mode(0o777))?;

    let metrics = Arc::new(Metrics::new());
    let kv = Arc::new(KvStore::default());
    let pubsub = Arc::new(PubSubBus::new(cfg.clone(), metrics.clone()));
    let runtime_cfg = Arc::new(RuntimeConfig::new(cfg.pressure_hot));
    let evictor = Arc::new(Evictor::new(kv.clone(), metrics.clone(), cfg.clone(), runtime_cfg.clone()));
    let op_semaphore = Arc::new(Semaphore::new(cfg.max_concurrent_ops));

    // Spawn evictor loop
    let ev_clone = evictor.clone();
    tokio::spawn(async move {
        ev_clone.run().await;
    });

    let ctx = Arc::new(Context {
        kv, pubsub, evictor, metrics, cfg: cfg.clone(), runtime_cfg, op_semaphore,
    });

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let c = ctx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, c).await {
                         tracing::error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(mut stream: UnixStream, ctx: Arc<Context>) -> anyhow::Result<()> {

    
    // Peek first byte to determine mode? 
    // Wait, protocol v1 doesn't distinguish handshake. 
    // Client sends opcode. 0x01..0x05 are KV, 0x20 is Sub.
    // We can just read the first frame.
    
    // Namun untuk menyederhanakan, kita delegasikan ke general dispatch
    // atau kita pisah handling berdasarkan request pertama?
    // Protocol spec tidak mewajibkan handshake "Hi I am Sub".
    // Jadi loop read frame, dispatch opcode.
    
    // Tapi jika client mengirim SUBSCRIBE, connection berubah state menjadi "Subscription Mode"
    // di mana dia TIDAK BOLEH mengirim command lain selain SUBSCRIBE/UNSUBSCRIBE
    // dan server HANYA push event.
    // Mari kita handle di satu handling loop.
    
    conn_kv::handle_conn(&mut stream, &ctx).await
}
