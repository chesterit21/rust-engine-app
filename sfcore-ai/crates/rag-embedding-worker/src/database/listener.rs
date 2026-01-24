use super::DocumentNotification;
use anyhow::Result;
use futures::StreamExt;
use serde_json;
use tokio::sync::mpsc;
use tokio_postgres::AsyncMessage;
use tracing::{debug, error, info, warn};

use crate::config::DatabaseConfig;

pub struct NotificationListener {
    // pool: DbPool, // Not needed for LISTEN connection, we make a new one
    config: DatabaseConfig,
    channel: String,
}

impl NotificationListener {
    pub fn new(config: DatabaseConfig, channel: String) -> Self {
        Self { config, channel }
    }
    
    /// Start listening untuk PostgreSQL notifications
    /// Returns channel untuk receive notifications
    pub async fn start(
        &self,
    ) -> Result<mpsc::UnboundedReceiver<DocumentNotification>> {
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Parse config to tokio-postgres config or use url
        // Here we assume self.config.url is valid
        let config_url = self.config.url.clone();
        let channel_name = self.channel.clone();

        // Spawn task untuk handle connection management and stream
        tokio::spawn(async move {
            info!("🔄 Connecting listener to DB...");
            
            // Connect
            let connect_result = tokio_postgres::connect(&config_url, tokio_postgres::NoTls).await;
            
            match connect_result {
                Ok((client, mut connection)) => {
                    info!("✅ Listener connected");
                    
                    // Spawn connection polling
                    let (_close_tx, mut _close_rx) = mpsc::unbounded_channel::<()>();
                    
                    // We need to poll the connection for it to work.
                    // But we also need to get notifications. 
                    // In tokio-postgres 0.7, notifications come from `stream::poll_fn` calling `connection.poll_message()`.
                    // But simpler way: connection object *is* the worker. We can just run it.
                    // BUT for notifications we need `client.batch_execute("LISTEN ...")` AND `connection.poll_message()`.
                    
                    // Let's use the standard loop pattern for notifications
                    
                    let mut stream = futures::stream::poll_fn(move |cx| {
                        connection.poll_message(cx)
                    });
                    
                    // Execute LISTEN
                     if let Err(e) = client.execute(&format!("LISTEN {}", channel_name), &[]).await {
                         error!("Failed to execute LISTEN: {}", e);
                         return;
                     }
                     
                     info!("✅ Started listening on channel: {}", channel_name);

                    loop {
                        tokio::select! {
                             msg_opt = stream.next() => {
                                 match msg_opt {
                                     Some(Ok(AsyncMessage::Notification(notif))) => {
                                         debug!("Received notification: {:?}", notif.payload());
                                         // Parse JSON payload
                                         match serde_json::from_str::<DocumentNotification>(notif.payload()) {
                                             Ok(doc_notif) => {
                                                 if let Err(e) = tx.send(doc_notif) {
                                                     error!("Failed to send notification to channel: {}", e);
                                                     break;
                                                 }
                                             }
                                             Err(e) => {
                                                 error!("Failed to parse notification payload: {}", e);
                                             }
                                         }
                                     }
                                     Some(Ok(AsyncMessage::Notice(notice))) => {
                                         debug!("Received notice: {:?}", notice);
                                     }
                                     Some(Ok(_)) => {}, // Keep alive or other messages
                                     Some(Err(e)) => {
                                         error!("Connection error: {}", e);
                                         break;
                                     }
                                     None => {
                                         warn!("Connection stream ended");
                                         break;
                                     }
                                 }
                             }
                        }
                    }
                }
                Err(e) => {
                    error!("Listener failed to connect: {}", e);
                }
            }
            
            error!("Listener connection closed");
        });
        
        Ok(rx)
    }
}