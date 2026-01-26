use serde::Serialize;
use tokio::sync::broadcast;
use tracing::warn;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
#[serde(rename_all = "snake_case")]
pub enum SystemEvent {
    ProcessingStarted { document_id: i32, filename: String },
    ProcessingProgress { document_id: i32, progress: f64, message: String, status_flag: String },
    ProcessingCompleted { document_id: i32, chunks_count: usize },
    ProcessingError { document_id: i32, error: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionEvent {
    pub session_id: i64,
    pub event: SystemEvent,
}

pub struct EventBus {
    tx: broadcast::Sender<SessionEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, session_id: i64, event: SystemEvent) {
        let session_event = SessionEvent { session_id, event };
        if let Err(e) = self.tx.send(session_event) {
            warn!("Failed to publish event (maybe no subscribers): {}", e);
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SessionEvent> {
        self.tx.subscribe()
    }
}
