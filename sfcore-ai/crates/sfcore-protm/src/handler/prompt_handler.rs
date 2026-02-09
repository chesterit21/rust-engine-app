use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use crate::{domain::TechStack, AppState};
use crate::dto::*;
use std::sync::Arc;
use std::fs;

pub async fn get_prompt(
    Path(key): Path<String>,
) -> Result<String, (StatusCode, String)> {
    // Read prompts.toml file on every request to allow hot-reloading
    let content = fs::read_to_string("crates/sfcore-protm/prompts.toml")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read prompts.toml: {}", e)))?;

    // Parse TOML
    let value = content.parse::<toml::Table>()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse prompts.toml: {}", e)))?;

    if let Some(section) = value.get(&key) {
        // Check if section is a table
        if let Some(table) = section.as_table() {
                if let Some(prompt) = table.get("generation_prompt") {
                    return Ok(prompt.as_str().unwrap_or("").to_string());
                }
        }
    }

    Err((StatusCode::NOT_FOUND, format!("Prompt key '{}' not found", key)))
}

pub async fn generate_architecture_prompt(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GeneratePromptRequest>,
) -> Result<String, (StatusCode, String)> {
    // 1. Fetch Tech Stack
    // Note: We access the pool directly. TechStack struct is in domain/mod.rs
    let stack = sqlx::query_as::<sqlx::Sqlite, TechStack>("SELECT id, name, type, language, description FROM tech_stacks WHERE id = ?")
        .bind(&payload.stack_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "Tech stack not found".to_string()))?;

    // 2. Read Prompt Template
    let content = fs::read_to_string("crates/sfcore-protm/prompts.toml")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to read prompts.toml: {}", e)))?;
    
    let toml_val = content.parse::<toml::Table>()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse prompts.toml: {}", e)))?;
    
    // Specific key for architecture patterns
    let template = toml_val.get("architecture_patterns")
        .and_then(|t| t.get("generation_prompt"))
        .and_then(|v| v.as_str())
        .ok_or((StatusCode::NOT_FOUND, "Prompt template 'architecture_patterns' not found".to_string()))?;

    // 3. Replace Placeholders
    // stack_type is mapped to "type" field in DB but "stack_type" in struct due to rename
    let filled = template
        .replace("{stack_id}", &stack.id)
        .replace("{stack_name}", &stack.name)
        .replace("{type}", &stack.stack_type)
        .replace("{description}", &stack.description.unwrap_or_default()) 
        .replace("{version}", &payload.version);

    Ok(filled)
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/generate", post(generate_architecture_prompt))
        .route("/{key}", get(get_prompt))
}
