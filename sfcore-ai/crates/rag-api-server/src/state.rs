use std::sync::Arc;
use axum::extract::FromRef;

use crate::config::Settings;
use crate::database::DbPool;
use crate::security::{CustomHeaderValidator, DocumentAuthorization, IpWhitelist};
use crate::services::{
    conversation::ConversationManager,
    EmbeddingService, LlmService, RagService, DocumentService, EventBus
};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub embedding_service: Arc<EmbeddingService>,
    pub rag_service: Arc<RagService>,
    pub llm_service: Arc<LlmService>,
    pub conversation_manager: Arc<ConversationManager>,
    pub settings: Settings,
    pub document_service: Arc<DocumentService>,
    pub document_auth: Arc<DocumentAuthorization>,
    pub ip_whitelist: Arc<IpWhitelist>,
    pub header_validator: Arc<CustomHeaderValidator>,
    pub event_bus: Arc<EventBus>,
    pub limiters: Arc<crate::utils::limiters::Limiters>, // NEW
}

impl FromRef<AppState> for Arc<ConversationManager> {
    fn from_ref(state: &AppState) -> Self {
        state.conversation_manager.clone()
    }
}

impl FromRef<AppState> for Arc<EventBus> {
    fn from_ref(state: &AppState) -> Self {
        state.event_bus.clone()
    }
}
