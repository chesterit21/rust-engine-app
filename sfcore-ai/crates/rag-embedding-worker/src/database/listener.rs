use super::DocumentNotification;
use anyhow::Result;
use futures::StreamExt;
use serde_json;
use tokio::sync::mpsc;
use tokio_postgres::AsyncMessage;
use tracing::{debug, error, info, warn};

use crate::config::DatabaseConfig;
use std::time::Duration;
use tokio::time::sleep;

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
            info!("🔄 Starting notification listener service...");
            
            loop {
                info!("🔄 Connecting listener to DB...");
                
                // Connect
                let connect_result = tokio_postgres::connect(&config_url, tokio_postgres::NoTls).await;
                
                match connect_result {
                    Ok((client, mut connection)) => {
                        info!("✅ Listener connected");
                        
                        let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
                        
                        // Execute LISTEN
                        if let Err(e) = client.execute(&format!("LISTEN {}", channel_name), &[]).await {
                             error!("Failed to execute LISTEN: {}. Retrying in 5s...", e);
                             sleep(Duration::from_secs(5)).await;
                             continue;
                        }
                         
                        info!("✅ Started listening on channel: {}", channel_name);

                        // Polling loop
                        let mut stream = futures::stream::poll_fn(move |cx| {
                            connection.poll_message(cx)
                        });

                        loop {
                            tokio::select! {
                                 // Handle messages
                                 msg_opt = stream.next() => {
                                     match msg_opt {
                                         Some(Ok(AsyncMessage::Notification(notif))) => {
                                             debug!("Received notification: {:?}", notif.payload());
                                             match serde_json::from_str::<DocumentNotification>(notif.payload()) {
                                                 Ok(doc_notif) => {
                                                     if let Err(e) = tx.send(doc_notif) {
                                                         error!("Failed to send notification to channel: {}", e);
                                                         // Channel closed, service should stop
                                                         return; 
                                                     }
                                                 }
                                                 Err(e) => error!("Failed to parse notification payload: {}", e),
                                             }
                                         }
                                         Some(Ok(AsyncMessage::Notice(notice))) => debug!("Received notice: {:?}", notice),
                                         Some(Ok(_)) => {}, 
                                         Some(Err(e)) => {
                                             error!("Connection error: {}", e);
                                             break; // Break inner loop to reconnect
                                         }
                                         None => {
                                             warn!("Connection stream ended");
                                             break; // Break inner loop to reconnect
                                         }
                                     }
                                 }
                                 // Handle shutdown signal if needed (currently unused but good practice)
                                 _ = shutdown_rx.recv() => {
                                     info!("Listener shutting down");
                                     return;
                                 }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Listener failed to connect: {}. Retrying in 5s...", e);
                    }
                }
                
                // Wait before reconnecting
                sleep(Duration::from_secs(5)).await;
            }
        });
        
        Ok(rx)
    }
}