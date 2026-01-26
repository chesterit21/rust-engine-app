Kita bisa tambah “planner call” (LLM call-1) dengan aman: hasilnya **tidak di-stream ke user**, hanya dipakai untuk (a) menentukan jalur retrieval (metadata vs vector), (b) optional query rewrite untuk embedding/retrieval, dan (c) ngasih `stage/progress` yang lebih halus.

## Desain planner (production)

Saat ini retrieval decision kamu banyak mengandalkan `QueryAnalyzer` + similarity embedding di `ContextBuilder::decide_retrieval()` (mis. overview/summary → `DocumentMetadataQuery`, clarification → context-aware).
Planner call kita taruh **sebelum embedding** supaya bisa memutuskan “metadata-only” tanpa biaya embedding/vector search.

### Output planner (JSON kecil)

Contoh schema yang kita minta dari LLM (jangan dipublikasikan ke user):

- `intent`: `"metadata"` | `"vector"` | `"clarify"`  
- `force_retrieve`: bool (kalau user minta “cek ulang”, override skip karena similarity tinggi)
- `context_aware`: bool
- `query_rewrite`: string|null (dipakai hanya untuk embedding/retrieval)
- `reason`: string|null (buat log/debug internal)

## Patch inti: extend LLM API supaya planner murah

Sekarang `LlmService::generate_chat()` selalu pakai `max_tokens` dari config dan `temperature: 0.7`.
Best practice: planner dibuat **temperature 0.0** dan **max_tokens kecil** (mis. 128–256), jadi murah + stabil.

### 1) Update trait `LlmProvider` (di `conversation/manager.rs`)

Tambahin method baru:

```rust
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;

    // NEW (planner-friendly)
    async fn generate_with(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String>;

    async fn generate_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>>;

    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String>;
}
```

### 2) Implement `generate_with` di `LlmService`

Karena request struct kamu sudah punya `max_tokens` dan `temperature`, tinggal buat helper baru.

```rust
impl LlmService {
    async fn generate_chat_with(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String, ApiError> {
        let request = ChatCompletionRequest {
            messages,
            max_tokens,
            temperature,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to call LLM API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!("LLM API error: {} - {}", status, body)));
        }

        #[derive(serde::Deserialize)]
        struct ChatCompletionResponse {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to parse LLM response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ApiError::LlmError("No choices returned from LLM".to_string()))
    }
}

#[async_trait::async_trait]
impl LlmProvider for LlmService {
    async fn generate(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        self.generate_chat(messages.to_vec()).await.map_err(|e| anyhow::anyhow!(e))
    }

    async fn generate_with(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> anyhow::Result<String> {
        self.generate_chat_with(messages.to_vec(), max_tokens, temperature)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    // ...generate_stream & summarize_chunks tetap...
}
```

## Integrasi planner di ConversationManager

Ini konsep patch ke `handle_message()` (yang kemarin sudah punya `stage/progress`).  
Tambahkan phase baru: `plan` (mis. progress 8–12), lalu planner memutuskan jalur.

### 1) Tambah tipe planner decision

```rust
#[derive(Debug, serde::Deserialize)]
struct PlannerDecision {
    intent: String,                 // "metadata" | "vector" | "clarify"
    force_retrieve: Option<bool>,
    context_aware: Option<bool>,
    query_rewrite: Option<String>,
    reason: Option<String>,
}
```

### 2) Tambah helper `call_planner()`

Planner prompt sebaiknya “strict JSON” (no markdown), dan kita parse dengan fallback ke rule-based `QueryAnalyzer` yang sudah ada sekarang.

```rust
async fn call_planner(
    &self,
    message: &str,
    document_id: Option<i64>,
) -> PlannerDecision {
    let sys = r#"
You are a planning module for a RAG system.
Return ONLY valid JSON with fields:
intent: "metadata"|"vector"|"clarify"
force_retrieve: boolean
context_aware: boolean
query_rewrite: string|null
reason: string|null
No markdown. No extra keys.
"#;

    let user = format!(
        "message: {}\ndocument_id: {:?}",
        message, document_id
    );

    let plan_messages = vec![
        ChatMessage::system(sys.to_string()),
        ChatMessage::user(user),
    ];

    let raw = match self.llm_provider.generate_with(&plan_messages, 192, 0.0).await {
        Ok(s) => s,
        Err(_) => return PlannerDecision {
            intent: "vector".to_string(),
            force_retrieve: Some(false),
            context_aware: Some(true),
            query_rewrite: None,
            reason: Some("planner_llm_failed_fallback".to_string()),
        },
    };

    serde_json::from_str::<PlannerDecision>(&raw).unwrap_or(PlannerDecision {
        intent: "vector".to_string(),
        force_retrieve: Some(false),
        context_aware: Some(true),
        query_rewrite: None,
        reason: Some("planner_json_parse_failed_fallback".to_string()),
    })
}
```

### 3) Ubah alur di `handle_message()`

- Emit `stage(plan, 10)`
- `planner = call_planner(...)`
- Kalau `intent == "metadata"` → langsung pakai `execute_metadata_query()` (tanpa embedding)
- Kalau `intent != "metadata"` → lanjut `embed` dan `decide_retrieval()` seperti biasa, tapi:
  - `retrieval_query = planner.query_rewrite.unwrap_or(message)`
  - jika hasil `decide_retrieval()` adalah `Skip` tapi `planner.force_retrieve == true` → override jadi `Retrieve` (pakai `RetrievalReason::LowSimilarity(0.0)` yang sudah ada)

## Pertanyaan cepat sebelum gue finalize patch lengkap

Planner ini kamu mau fokus di:

1) “metadata vs vector vs clarify” saja (simple & stabil), atau  

Menyederhanakan planner decision hanya untuk jalur “metadata vs vector vs clarify” agar lebih stabil.
Kita implement “planner call” yang simple: LLM call-1 hanya mengklasifikasikan intent jadi `metadata | vector | clarify`, lalu server pakai hasil itu untuk memilih jalur retrieval tanpa membocorkan output planner ke client.

## 1) Tambah API “cheap planner” di LLM

Karena `LlmService::generate_chat()` sekarang selalu pakai `max_tokens` dari config dan `temperature: 0.7`, planner bakal boros dan kurang stabil kalau kita pakai method itu langsung.
Solusinya: extend trait `LlmProvider` dengan `generate_with(max_tokens, temperature)` supaya planner bisa pakai `temperature: 0.0` dan `max_tokens` kecil (mis. 128–192).

### Patch A — update trait `LlmProvider` (di `src/services/conversation/manager.rs`)

Tambahkan method baru ini ke trait (di bagian `/// Trait for LLM service`).

```rust
#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[ChatMessage]) -> Result<String>;

    // NEW: untuk planner (murah & stabil)
    async fn generate_with(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String>;

    async fn generate_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>>;

    async fn summarize_chunks(&self, chunks: &[RetrievalChunk], query: &str) -> Result<String>;
}
```

### Patch B — implement `generate_with` (di `src/services/llm_service.rs`)

Tambahkan helper baru dan implement method trait-nya.

```rust
impl LlmService {
    async fn generate_chat_with(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String, ApiError> {
        let request = ChatCompletionRequest {
            messages,
            max_tokens,
            temperature,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to call LLM API: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!("LLM API error: {} - {}", status, body)));
        }

        #[derive(serde::Deserialize)]
        struct ChatCompletionResponse {
            choices: Vec<Choice>,
        }
        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(serde::Deserialize)]
        struct Message {
            content: String,
        }

        let chat_response: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to parse LLM response: {}", e)))?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ApiError::LlmError("No choices returned from LLM".to_string()))
    }
}

#[async_trait::async_trait]
impl LlmProvider for LlmService {
    async fn generate(&self, messages: &[ChatMessage]) -> anyhow::Result<String> {
        self.generate_chat(messages.to_vec())
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    async fn generate_with(
        &self,
        messages: &[ChatMessage],
        max_tokens: usize,
        temperature: f32,
    ) -> anyhow::Result<String> {
        self.generate_chat_with(messages.to_vec(), max_tokens, temperature)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    // generate_stream & summarize_chunks tetap seperti sekarang
}
```

## 2) Tambah planner decision (simple)

Kita buat planner output paling kecil agar parsing stabil: `{"intent":"metadata"|"vector"|"clarify"}`.
Untuk mapping ke retrieval: `metadata -> RetrievalReason::DocumentMetadataQuery`, `clarify -> RetrievalReason::ClarificationWithContext`, `vector -> pakai logic existing ContextBuilder::decide_retrieval()`.

Tambahkan di `manager.rs` (dekat `ConversationManager impl`):

```rust
#[derive(Debug, serde::Deserialize)]
struct PlannerOut {
    intent: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlannerIntent {
    Metadata,
    Vector,
    Clarify,
}

impl PlannerIntent {
    fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "metadata" => Self::Metadata,
            "clarify" => Self::Clarify,
            _ => Self::Vector,
        }
    }
}
```

## 3) Integrasi planner ke `handle_message()`

Di `ConversationManager::handle_message()`, saat ini flow kamu: enforce sliding window → embed → decide retrieval via `ContextBuilder` → retrieval loop → LLM.
Kita sisipkan planner **sebelum embedding**, lalu embedding hanya dilakukan kalau intent bukan `metadata`.

Patch konsep (potongan yang bisa kamu tempel setelah `enforce_sliding_window(...)`):

```rust
// === Planner call-1 (tidak di-stream ke user) ===
let planner_messages = vec![
    ChatMessage::system(
        "You are a planning module for a RAG system.\n\
Return ONLY valid JSON exactly like: {\"intent\":\"metadata\"} or {\"intent\":\"vector\"} or {\"intent\":\"clarify\"}.\n\
No markdown. No extra keys."
            .to_string(),
    ),
    ChatMessage::user(format!(
        "message: {}\ndocument_id: {:?}",
        message, document_id
    )),
];

let planner_raw = self
    .llm_provider
    .generate_with(&planner_messages, 160, 0.0)
    .await
    .unwrap_or_else(|_| "{\"intent\":\"vector\"}".to_string());

let planner_out = serde_json::from_str::<PlannerOut>(&planner_raw)
    .unwrap_or(PlannerOut { intent: "vector".to_string() });

let planner_intent = PlannerIntent::from_str(&planner_out.intent);

// === Embedding hanya kalau perlu ===
self.logger.log(
    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
        .message("KIRIM KE MODEL EMBEDDING")
        .build(),
);

let mut query_embedding: Option<Vec<f32>> = None;

if planner_intent != PlannerIntent::Metadata {
    query_embedding = Some(
        self.embedding_provider
            .embed(&message)
            .await
            .context("Failed to generate embedding")?,
    );
}

self.logger.log(
    ActivityLog::builder(session_id, user_id, ActivityType::ProcessingStage)
        .message("SELESAI MODEL EMBEDDING")
        .build(),
);
```

Lalu di bagian loop kamu yang sekarang bikin `decision = context_builder.decide_retrieval(...)`, ganti jadi:

```rust
let decision = match planner_intent {
    PlannerIntent::Metadata => RetrievalDecision::Retrieve {
        reason: RetrievalReason::DocumentMetadataQuery,
        context_aware: false,
    },
    PlannerIntent::Clarify => RetrievalDecision::Retrieve {
        reason: RetrievalReason::ClarificationWithContext,
        context_aware: true,
    },
    PlannerIntent::Vector => manager.context_builder.decide_retrieval(
        &final_state,
        &message,
        document_id,
        query_embedding.as_ref(), // Option<&Vec<f32>>
    )?,
};
```

Dan pas manggil `execute_retrieval_with_metrics(...)`, untuk parameter `current_embedding`, pakai fallback empty slice kalau metadata:

```rust
let emb: &[f32] = query_embedding.as_deref().unwrap_or(&[]);
let (system_context, metrics) = manager
    .execute_retrieval_with_metrics(
        &mut final_state,
        &decision,
        &message,
        document_id,
        emb,
        &tried_chunk_ids,
    )
    .await?;
```

Terakhir, pas update state di bawah, set `last_query_embedding` hanya kalau ada embedding:

```rust
if let Some(emb) = query_embedding {
    final_state.last_query_embedding = Some(emb);
}
```

## Opsional: stage/progress lebih halus

Kalau kamu sudah pakai SSE event `stage` yang kemarin, kamu tinggal emit phase baru `plan` sebelum `embed` (mis. progress 8–12) dan jangan pernah kirim `planner_raw` ke client.  
