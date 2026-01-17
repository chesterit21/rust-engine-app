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
