use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, delete},
    Json, Router,
};
use crate::{service::ArchitecturePatternService, domain::ArchitecturePattern, AppState};
use crate::dto::*;
use std::sync::Arc;
use axum::extract::Query;
use axum::routing::post;
use tracing::info;

pub async fn get_pattern_groups(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ArchitecturePatternGroup>>, (StatusCode, String)> {
    let query = r#"
        SELECT 
            t.id as stack_id, 
            t.name as stack_name, 
            t.type as stack_type, 
            p.version,
            COUNT(p.id) as item_count
        FROM 
            tech_stacks t 
        JOIN 
            architecture_patterns p ON t.id = p.stack_id
        GROUP BY 
            t.id, t.name, t.type, p.version
    "#;

    sqlx::query_as::<_, ArchitecturePatternGroup>(query)
        .fetch_all(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_all(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FilterParams>,
) -> Result<Json<Vec<ArchitecturePattern>>, (StatusCode, String)> {
    let mut query = "SELECT id, parent_id, stack_id, name, version, type as pattern_type, layer_rules, order_index, naming_conventions, created_at FROM architecture_patterns".to_string();
    let mut conditions = Vec::new();

    if let Some(stack_id) = params.stack_id {
        conditions.push(format!("stack_id = '{}'", stack_id));
    }
    if let Some(version) = params.version {
        conditions.push(format!("version = '{}'", version));
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }

    sqlx::query_as::<_, ArchitecturePattern>(&query)
        .fetch_all(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateArchitecturePatternDto>,
) -> Result<Json<ArchitecturePattern>, (StatusCode, String)> {
    // Validation
    let valid_versions = ["LITE", "STANDAR", "PRODUCTION GRADE"];
    if !valid_versions.contains(&payload.version.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid version. Must be one of: {:?}", valid_versions)));
    }
    let valid_types = ["BE", "FE", "FULLSTACK"];
    if !valid_types.contains(&payload.pattern_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid type. Must be one of: {:?}", valid_types)));
    }

    ArchitecturePatternService::create(
        &state.db, 
        payload.id,
        payload.parent_id,
        payload.stack_id,
        payload.name,
        payload.version,
        payload.pattern_type,
        payload.layer_rules,
        payload.order_index,
        payload.naming_conventions
    )
    .await
    .map(Json)
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn bulk_create(
    State(state): State<Arc<AppState>>,
    Json(payloads): Json<Vec<CreateArchitecturePatternDto>>,
) -> Result<StatusCode, (StatusCode, String)> {
    for payload in payloads {
            info!("Processing bulk item: name='{}', stack_id='{}', parent_id='{:?}'", payload.name, payload.stack_id, payload.parent_id);
            // Validation
        let valid_versions = ["LITE", "STANDAR", "PRODUCTION GRADE"];
        if !valid_versions.contains(&payload.version.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid version for {}: Must be one of: {:?}", payload.name, valid_versions)));
        }
        let valid_types = ["BE", "FE", "FULLSTACK"];
        if !valid_types.contains(&payload.pattern_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid type for {}: Must be one of: {:?}", payload.name, valid_types)));
        }
        
        let name_for_error = payload.name.clone();
        // Create
        let _ = ArchitecturePatternService::create(
            &state.db, 
            payload.id,
            payload.parent_id,
            payload.stack_id,
            payload.name,
            payload.version,
            payload.pattern_type,
            payload.layer_rules,
            payload.order_index,
            payload.naming_conventions
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create {}: {}", name_for_error, e)))?;
    }
    
    Ok(StatusCode::CREATED)
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateArchitecturePatternDto>,
) -> Result<Json<ArchitecturePattern>, (StatusCode, String)> {
        // Validation
    let valid_versions = ["LITE", "STANDAR", "PRODUCTION GRADE"];
    if !valid_versions.contains(&payload.version.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid version. Must be one of: {:?}", valid_versions)));
    }
    let valid_types = ["BE", "FE", "FULLSTACK"];
    if !valid_types.contains(&payload.pattern_type.as_str()) {
            return Err((StatusCode::BAD_REQUEST, format!("Invalid type. Must be one of: {:?}", valid_types)));
    }
    
    ArchitecturePatternService::update(
        &state.db, 
        &id, 
        payload.parent_id.as_deref(),
        &payload.stack_id,
        &payload.name,
        &payload.version,
        &payload.pattern_type,
        payload.layer_rules.as_deref(),
        payload.order_index,
        payload.naming_conventions.as_deref()
    )
    .await
    .map(Json)
    .map_err(|e: sqlx::Error| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    ArchitecturePatternService::delete(&state.db, &id)
        .await
        .map(|count| {
            if count > 0 {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::NOT_FOUND
            }
        })
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/groups", get(get_pattern_groups)) // Specific path must come before wildcard
        .route("/bulk", post(bulk_create))
        .route("/{id}", delete(delete_item).put(update))
}
