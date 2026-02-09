use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, delete},
    Json, Router,
};
use crate::{service::TechStackService, domain::TechStack, AppState};
use crate::dto::*;
use std::sync::Arc;

pub async fn get_all(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TechStack>>, (StatusCode, String)> {
    TechStackService::get_all(&state.db)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTechStackDto>,
) -> Result<Json<TechStack>, (StatusCode, String)> {
    TechStackService::create(&state.db, payload.name, payload.stack_type, payload.language, payload.description)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTechStackDto>,
) -> Result<Json<TechStack>, (StatusCode, String)> {
    TechStackService::update(&state.db, id, payload.name, payload.stack_type, payload.language, payload.description)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn delete_item(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    TechStackService::delete(&state.db, &id)
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

pub async fn get_prompt(
    State(_): State<Arc<AppState>>,
) -> Result<String, (StatusCode, String)> {
    let prompt = r#"Kamu adalah AI expert dibidang bahasa pemrograman, memahami semua stack teknologi dengan baik dan membantu user untuk memecahkan permaslahan tentang teknologi pemrograman.

### RULES & TASK
*- Bantu user sesuai [KONTEKS_PERTANYAAN] di bawah ini.
*- Pastikan jawaban kamu sesuai format output yang di minta, karena format jawaban sudah di sesuaikan dengan kebutuhan user sebagai data informasi yang di butuhkan.
*- Format output harus dalam format JSON yang sesuai di minta.
*- Pastikan jawaban kamu sesuai dengan informasi terkini dan deep-dive segala kemungkinan nya.

### CONTOH FORMAT OUTPUT
[
 {
    "name" : "Asp.netcore MVC", "type" : "FULLSTACK", "language" : "C#", "description" : "jelaskan deskripsi tentang teknologi framework Asp.Netcore MVC dan C# nya"
 },
 {
    "name" : "Vanilla Js + Bootstrap 5", "type" : "FE", "language" : "TypeScript", "description" : "jelaskan deskripsi tentang teknologi framework Vanilla JS + Boostrap 5 dan TypeScript nya"
 },
 {
    "name" : "Vanilla Js + TailwindCSS", "type" : "FE", "language" : "TypeScript", "description" : "jelaskan deskripsi tentang teknologi framework Vanilla JS + TailwindCSS dan TypeScript nya"
 },
 {
    "name" : "Django", "type" : "BE", "language" : "Python", "description" : "jelaskan deskripsi tentang teknologi framework Django dan Python nya"
 }
]

### KONTEKS_PERTANYAAN

Saya sedang membuat daftar list stack teknologi, perlu tahu semua kombinasi stack teknologi nya, karena bisa jadi satu stack teknologi , misal seperti react tidak selalu di menggunakan TailwindCSS, bisa jadi menggunakan library lain nya. begitu pun untuk stack teknologi lain nya, karena setiap industri bisnis atau developer mempunyai keunikan masing-masing sesuai kebutuhan bisnis atau bisa jadi untuk proses pembelajaran mereka sendiri. Tolong capture semua segala kemungkinanan nya, jangan sampai ada yang terlewatkan, di mulai dari teknologi yang umum dahulu saat ini yang sering di gunakan dan moder, lalu capture semua teknologi programming yang lawas nya. Go head."#;
    Ok(prompt.to_string())
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(get_all).post(create))
        .route("/prompt", get(get_prompt))
        .route("/{id}", delete(delete_item).put(update))
}
