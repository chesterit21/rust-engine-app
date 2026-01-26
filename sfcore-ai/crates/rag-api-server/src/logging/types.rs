use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::collections::HashMap;
use serde_json::Value;

/// Activity type categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    RequestReceived,    // Log initial payload
    MessageSent,        // Log final completion
    RetrievalExecuted,
    RetrievalSkipped,
    TokenOverflow,
    SlidingWindowEnforced,
    LlmError,
    RetrievalError,
    SessionCreated,
    SessionExpired,
    CascadeDeletion,
    ProcessingStage,
}

impl ActivityType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::RequestReceived => "request_received",
            Self::MessageSent => "message_sent",
            Self::RetrievalExecuted => "retrieval_executed",
            Self::RetrievalSkipped => "retrieval_skipped",
            Self::TokenOverflow => "token_overflow",
            Self::SlidingWindowEnforced => "sliding_window_enforced",
            Self::LlmError => "llm_error",
            Self::RetrievalError => "retrieval_error",
            Self::SessionCreated => "session_created",
            Self::SessionExpired => "session_expired",
            Self::CascadeDeletion => "cascade_deletion",
            Self::ProcessingStage => "processing_stage",
        }
    }
}

/// Activity status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActivityStatus {
    Success,
    Error,
    Warning,
    Info, // For "RequestReceived"
}

impl ActivityStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Success => "success",
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

/// Complete activity log entry
#[derive(Debug, Clone)]
pub struct ActivityLog {
    // Session & User
    pub session_id: i64,
    pub user_id: i64,
    
    // Activity
    pub activity_type: ActivityType,
    pub activity_status: ActivityStatus,
    
    // Context
    pub document_id: Option<i64>,
    pub message_content: Option<String>,
    pub response_content: Option<String>,
    
    // Metrics
    pub token_count: Option<i32>,
    pub retrieval_skipped: Option<bool>,
    pub similarity_score: Option<f32>,
    
    // Performance
    pub processing_time_ms: Option<i32>,
    pub llm_call_duration_ms: Option<i32>,
    pub retrieval_duration_ms: Option<i32>,
    
    // Error
    pub error_message: Option<String>,
    pub error_type: Option<String>,
    
    // Metadata
    pub user_agent: Option<String>,
    pub ip_address: Option<IpAddr>,
    
    // Timestamp
    pub created_at: DateTime<Utc>,

    // Custom fields
    pub custom_fields: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl ActivityLog {
    /// Create builder for fluent API
    pub fn builder(session_id: i64, user_id: i64, activity_type: ActivityType) -> ActivityLogBuilder {
        ActivityLogBuilder::new(session_id, user_id, activity_type)
    }
}

/// Builder pattern for ActivityLog
pub struct ActivityLogBuilder {
    log: ActivityLog,
    custom_fields: Option<std::collections::HashMap<String, serde_json::Value>>, // ADD THIS
}

impl ActivityLogBuilder {
    pub fn new(session_id: i64, user_id: i64, activity_type: ActivityType) -> Self {
        Self {
            log: ActivityLog {
                session_id,
                user_id,
                activity_type,
                activity_status: ActivityStatus::Success,
                document_id: None,
                message_content: None,
                response_content: None,
                token_count: None,
                retrieval_skipped: None,
                similarity_score: None,
                processing_time_ms: None,
                llm_call_duration_ms: None,
                retrieval_duration_ms: None,
                error_message: None,
                error_type: None,
                user_agent: None,
                ip_address: None,
                created_at: Utc::now(),
                custom_fields: None,
            },
            custom_fields: None,  // ADD THIS LINE
        }
    }

    pub fn status(mut self, status: ActivityStatus) -> Self {
        self.log.activity_status = status;
        self
    }

    pub fn document_id(mut self, id: i64) -> Self {
        self.log.document_id = Some(id);
        self
    }

    pub fn message(mut self, content: impl Into<String>) -> Self {
        self.log.message_content = Some(content.into());
        self
    }

    pub fn response(mut self, content: impl Into<String>) -> Self {
        self.log.response_content = Some(content.into());
        self
    }

    pub fn token_count(mut self, count: i32) -> Self {
        self.log.token_count = Some(count);
        self
    }

    pub fn retrieval_skipped(mut self, skipped: bool) -> Self {
        self.log.retrieval_skipped = Some(skipped);
        self
    }

    pub fn similarity(mut self, score: f32) -> Self {
        self.log.similarity_score = Some(score);
        self
    }

    pub fn processing_time(mut self, ms: i32) -> Self {
        self.log.processing_time_ms = Some(ms);
        self
    }

    pub fn llm_duration(mut self, ms: i32) -> Self {
        self.log.llm_call_duration_ms = Some(ms);
        self
    }

    pub fn retrieval_duration(mut self, ms: i32) -> Self {
        self.log.retrieval_duration_ms = Some(ms);
        self
    }

    pub fn error(mut self, message: impl Into<String>, error_type: impl Into<String>) -> Self {
        self.log.error_message = Some(message.into());
        self.log.error_type = Some(error_type.into());
        self.log.activity_status = ActivityStatus::Error;
        self
    }

    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.log.user_agent = Some(agent.into());
        self
    }

    pub fn ip_address(mut self, ip: IpAddr) -> Self {
        self.log.ip_address = Some(ip);
        self
    }

    pub fn build(self) -> ActivityLog {
        self.log
    }

    /// Add custom key-value data (encode in message for now)
    pub fn custom(mut self, key: &str, value: impl Into<Value>) -> Self {
        if self.custom_fields.is_none() {
            self.custom_fields = Some(HashMap::new());
        }
        
        if let Some(ref mut fields) = self.custom_fields {
            fields.insert(key.to_string(), value.into());
        }
        
        self
    }

}
