use crate::config::Config;
use crate::metrics::Metrics;
use dashmap::DashMap;
use localcached_proto::PushEvent;
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct PubSubBus {
    channels: DashMap<String, broadcast::Sender<PushEvent>>,
    cfg: Config,
    metrics: Arc<Metrics>,
}

impl PubSubBus {
    pub fn new(cfg: Config, metrics: Arc<Metrics>) -> Self {
        Self {
            channels: DashMap::new(),
            cfg,
            metrics,
        }
    }

    pub fn publish(&self, topic: &str, event: PushEvent) {
        if let Some(sender) = self.channels.get(topic) {
            // Jika tidak ada receiver, broadcast return error, kita ignore saja (semantics: at-most-once)
            // Tapi kita hitung metrics
            let _ = sender.send(event);
            self.metrics.inc_published();
        }
        // Lazy cleanup empty channels? Nanti dulu, biar simple v1.
    }

    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<PushEvent> {
        let entry = self.channels.entry(topic.to_string()).or_insert_with(|| {
            let (tx, _rx) = broadcast::channel(self.cfg.pubsub_capacity);
            tx
        });
        entry.subscribe()
    }

    pub fn topic_count(&self) -> u64 {
        self.channels.len() as u64
    }
}
