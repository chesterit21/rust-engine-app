Sip—kita bikin `Limiters` global di `AppState`, tapi yang *consume* tetap `Service/ConversationManager`, dan semua angka limiter masuk `settings.toml` + bisa dioverride via environment variable `APP__...`. [mcp_tool_github-mcp-direct_get_file_contents:1][mcp_tool_github-mcp-direct_get_file_contents:2]

## 1) Skema config (TOML + env var)

Karena loader kamu pakai `Environment::with_prefix("APP").separator("__").try_parsing(true)`, maka nested config bisa dioverride dengan format `APP__SECTION__FIELD=value` dan otomatis ke-parse jadi number/bool. [mcp_tool_github-mcp-direct_get_file_contents:2]

Tambahkan ini ke `config/settings.toml`:

```toml
[limits]
embedding_concurrency = 8
db_search_concurrency = 20
llm_generate_concurrency = 4
llm_stream_concurrency = 2

# opsional (biar request gak ngantri selamanya kalau downstream macet)
acquire_timeout_ms = 15000
```

Contoh override via env var (sesuai pola `APP__...`): [mcp_tool_github-mcp-direct_get_file_contents:2]

- `APP__LIMITS__DB_SEARCH_CONCURRENCY=20`
- `APP__LIMITS__EMBEDDING_CONCURRENCY=8`
- `APP__LIMITS__LLM_GENERATE_CONCURRENCY=4`
- `APP__LIMITS__LLM_STREAM_CONCURRENCY=2`
- `APP__LIMITS__ACQUIRE_TIMEOUT_MS=15000`

## 2) Struct config di Rust (`settings.rs`)

Di `src/config/settings.rs`, tambahin struct baru dan daftarkan ke `Settings` (karena `Settings` saat ini belum punya field limiter). [mcp_tool_github-mcp-direct_get_file_contents:2]

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub embedding_concurrency: usize,
    pub db_search_concurrency: usize,
    pub llm_generate_concurrency: usize,
    pub llm_stream_concurrency: usize,
    pub acquire_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub rag: RagConfig,
    pub prompts: PromptsConfig,
    pub limits: LimitsConfig, // NEW
}
```

## 3) Rancangan `Limiters` (yang ditaruh di AppState)

Karena `AppState` memang jadi tempat shared dependency dan di-build di `main.rs`, ini tempat paling tepat untuk menaruh `Arc<Limiters>`. [mcp_tool_github-mcp-direct_get_file_contents:0][mcp_tool_github-mcp-direct_get_file_contents:1]

Contoh file baru: `src/utils/limiters.rs` (atau `src/services/limiters.rs`, terserah style repo kamu):

```rust
use std::{sync::Arc, time::{Duration, Instant}};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use anyhow::Result;

#[derive(Clone)]
pub struct Limiters {
    pub embedding: Arc<Semaphore>,
    pub db_search: Arc<Semaphore>,
    pub llm_generate: Arc<Semaphore>,
    pub llm_stream: Arc<Semaphore>,
    pub acquire_timeout: Duration,
}

impl Limiters {
    pub fn new(cfg: &crate::config::LimitsConfig) -> Self {
        Self {
            embedding: Arc::new(Semaphore::new(cfg.embedding_concurrency.max(1))),
            db_search: Arc::new(Semaphore::new(cfg.db_search_concurrency.max(1))),
            llm_generate: Arc::new(Semaphore::new(cfg.llm_generate_concurrency.max(1))),
            llm_stream: Arc::new(Semaphore::new(cfg.llm_stream_concurrency.max(1))),
            acquire_timeout: Duration::from_millis(cfg.acquire_timeout_ms.max(1)),
        }
    }

    pub async fn acquire_timed(
        sem: Arc<Semaphore>,
        acquire_timeout: Duration,
    ) -> Result<(OwnedSemaphorePermit, Duration)> {
        let start = Instant::now();
        let permit = tokio::time::timeout(acquire_timeout, sem.acquire_owned()).await??;
        Ok((permit, start.elapsed()))
    }
}
```

### Wiring ke `AppState`

Di `src/state.rs` tambah field:

```rust
pub struct AppState {
    // ...
    pub limiters: Arc<crate::utils::limiters::Limiters>,
}
```

Lalu di `main.rs` saat build state (posisinya pas setelah `settings` kebaca), buat instance: [mcp_tool_github-mcp-direct_get_file_contents:0]

```rust
let limiters = Arc::new(utils::limiters::Limiters::new(&settings.limits));

let app_state = AppState {
    // ...
    limiters: limiters.clone(),
};
```

## 4) Cara consume + ukur wait/execution (Service/Manager)

Karena orchestrator utama ada di `ConversationManager::handle_message()` (embed → retrieve → LLM), idealnya limiter dipakai di `EmbeddingService/RagService/LlmService` supaya semua call (termasuk endpoint lain seperti upload/search) otomatis ke-limit. [mcp_tool_github-mcp-direct_get_file_contents:3][mcp_tool_github-mcp-direct_get_file_contents:0]

Pola yang aku rekomendasikan untuk **wait_queue vs execution** (contoh di `LlmService::generate_chat_with`): [mcp_tool_github-mcp-direct_get_file_contents:3]

```rust
let (permit, wait) = Limiters::acquire_timed(
    self.limiters.llm_generate.clone(),
    self.limiters.acquire_timeout,
).await?;
tracing::debug!(wait_ms = wait.as_millis() as u64, "llm_generate_wait");

// eksekusi call
let exec_start = std::time::Instant::now();
let resp = self.client.post(...).send().await?;
tracing::debug!(exec_ms = exec_start.elapsed().as_millis() as u64, "llm_generate_exec");

drop(permit);
```

Sip, kita eksekusi dengan layout yang konsisten: `LimitsConfig` di `Settings`, `Limiters` (berisi 4 semaphore + timeout) di `AppState`, lalu setiap service (Embedding/RAG/LLM) pegang `Arc<Limiters>` dan melakukan `acquire()` + logging wait/exec di titik call resource (HTTP/DB). [mcp_tool_github-mcp-direct_get_file_contents:1][mcp_tool_github-mcp-direct_get_file_contents:3]

## 1) Tambah modul `utils::limiters`

Saat ini `utils/mod.rs` belum mengekspor limiter. [mcp_tool_github-mcp-direct_get_file_contents:0]

1) Buat file baru: `src/utils/limiters.rs`:

```rust
use anyhow::Result;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone)]
pub struct Limiters {
    pub embedding: Arc<Semaphore>,
    pub db_search: Arc<Semaphore>,
    pub llm_generate: Arc<Semaphore>,
    pub llm_stream: Arc<Semaphore>,
    pub acquire_timeout: Duration,
}

impl Limiters {
    pub fn new(cfg: &crate::config::LimitsConfig) -> Self {
        Self {
            embedding: Arc::new(Semaphore::new(cfg.embedding_concurrency.max(1))),
            db_search: Arc::new(Semaphore::new(cfg.db_search_concurrency.max(1))),
            llm_generate: Arc::new(Semaphore::new(cfg.llm_generate_concurrency.max(1))),
            llm_stream: Arc::new(Semaphore::new(cfg.llm_stream_concurrency.max(1))),
            acquire_timeout: Duration::from_millis(cfg.acquire_timeout_ms.max(1)),
        }
    }

    pub async fn acquire_timed(
        sem: Arc<Semaphore>,
        acquire_timeout: Duration,
    ) -> Result<(OwnedSemaphorePermit, Duration)> {
        let start = Instant::now();
        let permit = tokio::time::timeout(acquire_timeout, sem.acquire_owned()).await??;
        Ok((permit, start.elapsed()))
    }
}
```

1) Update `src/utils/mod.rs`:

```rust
pub mod error;
pub mod response;
pub mod similarity;
pub mod token_estimator;
pub mod limiters; // NEW

pub use similarity::cosine_similarity;
```

## 2) Tambah `LimitsConfig` ke Settings (+ TOML/env)

`Settings` kamu sekarang belum punya section `limits`. [mcp_tool_github-mcp-direct_get_file_contents:2]

Di `src/config/settings.rs` tambahkan:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub embedding_concurrency: usize,
    pub db_search_concurrency: usize,
    pub llm_generate_concurrency: usize,
    pub llm_stream_concurrency: usize,
    pub acquire_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub rag: RagConfig,
    pub prompts: PromptsConfig,
    pub limits: LimitsConfig, // NEW
}
```

Di `config/settings.toml` tambahkan:

```toml
[limits]
embedding_concurrency = 8
db_search_concurrency = 20
llm_generate_concurrency = 4
llm_stream_concurrency = 2
acquire_timeout_ms = 15000
```

Env var override (sesuai loader kamu `APP__...`): `APP__LIMITS__DB_SEARCH_CONCURRENCY=20`, dst. [mcp_tool_github-mcp-direct_get_file_contents:2]

## 3) Wiring ke `AppState` dan `main.rs`

`AppState` saat ini berisi service-service shared, jadi cocok untuk menambah `limiters: Arc<Limiters>`. [mcp_tool_github-mcp-direct_get_file_contents:1]

Di `src/state.rs`:

```rust
pub struct AppState {
    // ...
    pub limiters: Arc<crate::utils::limiters::Limiters>,
}
```

Di `main.rs`, setelah `let settings = Settings::load()?;` bikin:

```rust
let limiters = Arc::new(utils::limiters::Limiters::new(&settings.limits));
```

Lalu masukkan ke `AppState { ... limiters: limiters.clone(), }`. [mcp_tool_github-mcp-direct_get_file_contents:0]

## 4) Inject ke 3 service + titik acquire

Struktur `services/mod.rs` menunjukkan 3 service utama ada di `EmbeddingService`, `RagService`, `LlmService`. [mcp_tool_github-mcp-direct_get_file_contents:2]

### EmbeddingService

Sekarang `EmbeddingService` punya cache dan melakukan network call di `embed_internal()`. [mcp_tool_github-mcp-direct_get_file_contents:1]

Perubahan yang aku sarankan:

- Tambah field `limiters: Arc<Limiters>`.
- `embed_internal`: lakukan `acquire` hanya ketika cache MISS (jadi cache HIT tidak kena limiter).
- Log `embedding_wait_ms` dan `embedding_exec_ms`.

Sketsa patch inti:

```rust
use crate::utils::limiters::Limiters;

#[derive(Clone)]
pub struct EmbeddingService {
    // ...
    limiters: Arc<Limiters>,
}

pub fn new(llm_base_url: String, config: EmbeddingConfig, limiters: Arc<Limiters>) -> Self {
    Self { /*...*/ limiters }
}

async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
    // cache check (no limiter)
    { /* ... */ }

    let (_permit, wait) = Limiters::acquire_timed(
        self.limiters.embedding.clone(),
        self.limiters.acquire_timeout,
    ).await?;
    tracing::debug!(wait_ms = wait.as_millis() as u64, "embedding_wait");

    let exec_start = std::time::Instant::now();
    // http call...
    tracing::debug!(exec_ms = exec_start.elapsed().as_millis() as u64, "embedding_exec");

    Ok(embedding.clone())
}
```

### RagService (DB search + get_first_chunk)

Di `RagService::retrieve_with_embedding()`, DB hit utamanya ada di `repository.search_user_documents()` / `hybrid_search_user_documents()` dan kadang `get_first_chunk()`. [mcp_tool_github-mcp-direct_get_file_contents:3]

Pasang limiter `db_search`:

- Acquire sebelum call search, ukur `db_search_wait_ms` + `db_search_exec_ms`.
- Untuk `get_first_chunk`, juga pakai limiter yang sama (biar gak ada “jalan samping” nembus limit).

### LlmService (generate vs stream)

`LlmService` sekarang punya 2 jalur: `chat_stream()` dan `generate_chat_with()` (non-stream). [mcp_tool_github-mcp-direct_get_file_contents:3]

Pasang limiter terpisah:

- `llm_stream` dipakai di `chat_stream()`
- `llm_generate` dipakai di `generate_chat_with()` (ini juga otomatis nge-limit planner call di `ConversationManager` karena dia pakai `generate_with`). [mcp_tool_github-mcp-direct_get_file_contents:3]

***
Berikut “versi lengkap” (copy–paste ready) untuk: `LimitsConfig` + `Limiters` global di `AppState`, limiter dipakai di Service layer (Embedding/RAG/LLM), plus observability `wait_queue_ms` vs `exec_ms` dan SSE buffering fix (tanpa dependency tambahan). [mcp_tool_github-mcp-direct_get_file_contents:0][mcp_tool_github-mcp-direct_get_file_contents:1][mcp_tool_github-mcp-direct_get_file_contents:2]

## 0) Files yang berubah/baru

- NEW: `src/utils/limiters.rs`
- UPDATE: `src/utils/mod.rs`
- UPDATE: `src/config/settings.rs` (tambah `[limits]`)
- UPDATE: `config/settings.toml` (tambah `[limits]`) [mcp_tool_github-mcp-direct_get_file_contents:0]
- UPDATE: `src/state.rs` (tambah `limiters`)
- UPDATE: `src/services/embedding_service.rs` (limiter + wait/exec)
- UPDATE: `src/services/rag_service.rs` (limiter DB search + intro chunk fetch) [mcp_tool_github-mcp-direct_get_file_contents:1]
- UPDATE: `src/services/llm_service.rs` (limiter generate vs stream + SSE buffer) [mcp_tool_github-mcp-direct_get_file_contents:2]
- UPDATE: `src/main.rs` (wiring constructor baru)

***

## 1) Update `config/settings.toml`

Tambahin block ini (di bawah `[llm]` atau di mana saja): [mcp_tool_github-mcp-direct_get_file_contents:0]

```toml
[limits]
embedding_concurrency = 8
db_search_concurrency = 20
llm_generate_concurrency = 4
llm_stream_concurrency = 2
acquire_timeout_ms = 15000
```

Env override (sesuai loader `APP__...`):  

- `APP__LIMITS__EMBEDDING_CONCURRENCY=8`  
- `APP__LIMITS__DB_SEARCH_CONCURRENCY=20`  
- `APP__LIMITS__LLM_GENERATE_CONCURRENCY=4`  
- `APP__LIMITS__LLM_STREAM_CONCURRENCY=2`  
- `APP__LIMITS__ACQUIRE_TIMEOUT_MS=15000`

***

## 2) NEW `src/utils/limiters.rs`

Buat file baru:

```rust
use anyhow::Result;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(Clone)]
pub struct Limiters {
    pub embedding: Arc<Semaphore>,
    pub db_search: Arc<Semaphore>,
    pub llm_generate: Arc<Semaphore>,
    pub llm_stream: Arc<Semaphore>,
    pub acquire_timeout: Duration,
}

impl Limiters {
    pub fn new(cfg: &crate::config::LimitsConfig) -> Self {
        Self {
            embedding: Arc::new(Semaphore::new(cfg.embedding_concurrency.max(1))),
            db_search: Arc::new(Semaphore::new(cfg.db_search_concurrency.max(1))),
            llm_generate: Arc::new(Semaphore::new(cfg.llm_generate_concurrency.max(1))),
            llm_stream: Arc::new(Semaphore::new(cfg.llm_stream_concurrency.max(1))),
            acquire_timeout: Duration::from_millis(cfg.acquire_timeout_ms.max(1)),
        }
    }

    pub async fn acquire_timed(
        sem: Arc<Semaphore>,
        acquire_timeout: Duration,
        op: &'static str,
    ) -> Result<(OwnedSemaphorePermit, Duration)> {
        let start = Instant::now();

        let permit = tokio::time::timeout(acquire_timeout, sem.acquire_owned())
            .await
            .map_err(|_| anyhow::anyhow!("Limiter acquire timeout for op={}", op))??;

        Ok((permit, start.elapsed()))
    }
}
```

***

## 3) UPDATE `src/utils/mod.rs`

Tambahkan modul baru:

```rust
pub mod error;
pub mod response;
pub mod similarity;
pub mod token_estimator;
pub mod limiters; // NEW

pub use similarity::cosine_similarity;
```

***

## 4) UPDATE `src/config/settings.rs`

Tambahkan `LimitsConfig` dan inject ke `Settings`:

```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitsConfig {
    pub embedding_concurrency: usize,
    pub db_search_concurrency: usize,
    pub llm_generate_concurrency: usize,
    pub llm_stream_concurrency: usize,
    pub acquire_timeout_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub database: DatabaseConfig,
    pub embedding: EmbeddingConfig,
    pub llm: LlmConfig,
    pub rag: RagConfig,
    pub prompts: PromptsConfig,
    pub limits: LimitsConfig, // NEW
}
```

***

## 5) UPDATE `src/state.rs`

Tambahkan field limiter:

```rust
use crate::utils::limiters::Limiters;

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

    pub limiters: Arc<Limiters>, // NEW
}
```

***

## 6) UPDATE `src/services/embedding_service.rs`

Perubahan inti:

- Tambah `limiters: Arc<Limiters>`
- Constructor tambah arg `limiters`
- Acquire limiter hanya saat cache MISS
- Emit `embedding_wait_ms` dan `embedding_exec_ms`

Patch gaya “replace bagian struct+new+embed_internal”:

```rust
use crate::utils::limiters::Limiters;
use std::time::Instant;

#[derive(Clone)]
pub struct EmbeddingService {
    client: Client,
    base_url: String,
    dimension: usize,
    model_name: String,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
    limiters: Arc<Limiters>, // NEW
}

impl EmbeddingService {
    pub fn new(llm_base_url: String, config: EmbeddingConfig, limiters: Arc<Limiters>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .unwrap_or_else(|_| Client::new()),
            base_url: llm_base_url,
            dimension: config.dimension,
            model_name: config.model,
            cache: Arc::new(RwLock::new(HashMap::new())),
            limiters, // NEW
        }
    }

    async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
        // 1) Cache check (no limiter)
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                debug!("Cache HIT for embedding ({:.20}...) - skipping API call", text);
                return Ok(embedding.clone());
            }
        }

        // 2) Limiter acquire (only on cache MISS)
        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.embedding.clone(),
            self.limiters.acquire_timeout,
            "embedding",
        )
        .await?;

        debug!(wait_ms = wait.as_millis() as u64, op = "embedding", "wait_queue");

        let exec_start = Instant::now();

        debug!("Generating embedding for {} chars using model {}", text.len(), self.model_name);

        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model_name.clone(),
        };

        let url = format!("{}/v1/embeddings", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to embedding server")?;

        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "embedding", "exec");

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Embedding API error ({}): {}", status, body);
        }

        let response_body: EmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse embedding response (expected OpenAI format)")?;

        if response_body.data.is_empty() {
            anyhow::bail!("Empty data array returned from embedding server");
        }

        let embedding = &response_body.data[0].embedding;

        if embedding.is_empty() {
            anyhow::bail!("Generated embedding vector is empty");
        }

        if embedding.len() != self.dimension {
            anyhow::bail!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.dimension,
                embedding.len()
            );
        }

        // 3) Store in Cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding.clone())
    }
}
```

***

## 7) UPDATE `src/services/rag_service.rs`

Tambahkan limiter DB untuk:

- hybrid/vector search (main cost) [mcp_tool_github-mcp-direct_get_file_contents:1]
- `get_first_chunk()` (jalan samping) [mcp_tool_github-mcp-direct_get_file_contents:1]

Patch inti:

```rust
use crate::utils::limiters::Limiters;
use std::time::Instant;

#[derive(Clone)]
pub struct RagService {
    pub repository: Arc<Repository>,
    pub embedding_service: Arc<EmbeddingService>,
    pub llm_service: Arc<LlmService>,
    pub config: RagConfig,
    pub limiters: Arc<Limiters>, // NEW
}

impl RagService {
    pub fn new(
        repository: Arc<Repository>,
        embedding_service: Arc<EmbeddingService>,
        llm_service: Arc<LlmService>,
        config: RagConfig,
        limiters: Arc<Limiters>, // NEW
    ) -> Self {
        Self {
            repository,
            embedding_service,
            llm_service,
            config,
            limiters,
        }
    }

    pub async fn retrieve_with_embedding(
        &self,
        user_id: i32,
        query_text: &str,
        query_embedding: Vec<f32>,
        document_ids: Option<Vec<i32>>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        info!("Retrieving context with embedding for user {}", user_id);

        let vector = Vector::from(query_embedding);

        // Acquire DB-search limiter (covers hybrid/vector search)
        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.db_search.clone(),
            self.limiters.acquire_timeout,
            "db_search",
        )
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

        debug!(wait_ms = wait.as_millis() as u64, op = "db_search", "wait_queue");

        let exec_start = Instant::now();

        let mut chunks = if self.config.rerank_enabled {
            let search_future = self.repository.hybrid_search_user_documents(
                user_id,
                vector.clone(),
                query_text.to_string(),
                self.config.retrieval_top_k as i32,
                document_ids.clone(),
            );

            match tokio::time::timeout(std::time::Duration::from_secs(15), search_future).await {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    warn!("Hybrid search failed: {}", e);
                    return Err(ApiError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    warn!("Hybrid search timeout after 15s");
                    return Err(ApiError::DatabaseError("Search timeout".to_string()));
                }
            }
        } else {
            let search_future = self.repository.search_user_documents(
                user_id,
                vector.clone(),
                self.config.retrieval_top_k as i32,
                document_ids.clone(),
            );

            match tokio::time::timeout(std::time::Duration::from_secs(15), search_future).await {
                Ok(Ok(c)) => c,
                Ok(Err(e)) => {
                    warn!("Vector search failed: {}", e);
                    return Err(ApiError::DatabaseError(e.to_string()));
                }
                Err(_) => {
                    warn!("Vector search timeout after 15s");
                    return Err(ApiError::DatabaseError("Search timeout".to_string()));
                }
            }
        };

        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "db_search", "exec");

        // Intro chunk injection juga lewat limiter DB (biar gak bypass concurrency guard)
        if let Some(ids) = &document_ids {
            if ids.len() == 1 {
                let doc_id = ids[0];
                let has_intro = chunks.iter().any(|c| c.chunk_index == 0 && c.document_id == doc_id);

                if !has_intro {
                    let (_permit, wait) = Limiters::acquire_timed(
                        self.limiters.db_search.clone(),
                        self.limiters.acquire_timeout,
                        "db_search_intro_chunk",
                    )
                    .await
                    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

                    debug!(wait_ms = wait.as_millis() as u64, op = "db_search_intro_chunk", "wait_queue");

                    let exec_start = Instant::now();
                    match self.repository.get_first_chunk(doc_id).await {
                        Ok(Some(intro_chunk)) => {
                            debug!("Injecting intro chunk (index 0) for doc {}", doc_id);
                            chunks.insert(0, intro_chunk);
                        }
                        Ok(None) => debug!("No intro chunk found for doc {}", doc_id),
                        Err(e) => warn!("Failed to fetch intro chunk: {}", e),
                    }
                    debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "db_search_intro_chunk", "exec");
                }
            }
        }

        debug!("Retrieved {} chunks for user {}", chunks.len(), user_id);
        Ok(chunks)
    }
}
```

***

## 8) UPDATE `src/services/llm_service.rs` (Limiter + SSE buffer)

Di LLM ada 2 limiter: `llm_generate` untuk non-stream, `llm_stream` untuk stream. [mcp_tool_github-mcp-direct_get_file_contents:2]  
Untuk stream, permit harus “hidup” sampai stream selesai; jadi permit kita simpan di state `unfold`. [mcp_tool_github-mcp-direct_get_file_contents:2]

**Replace struct + new() + chat_stream() + generate_chat_with()** seperti ini:

```rust
use crate::utils::limiters::Limiters;
use std::{sync::Arc, time::Instant};

#[derive(Clone)]
pub struct LlmService {
    client: Client,
    config: LlmConfig,
    context_extraction_system_prompt: String,
    limiters: Arc<Limiters>, // NEW
}

impl LlmService {
    pub fn new(
        config: LlmConfig,
        context_extraction_system_prompt: String,
        limiters: Arc<Limiters>, // NEW
    ) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(config.timeout_seconds))
                .build()
                .expect("Failed to create HTTP client"),
            config,
            context_extraction_system_prompt,
            limiters,
        }
    }

    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, ApiError>> + Send>>, ApiError> {
        debug!("Starting chat stream with {} messages", messages.len());

        let (permit, wait) = Limiters::acquire_timed(
            self.limiters.llm_stream.clone(),
            self.limiters.acquire_timeout,
            "llm_stream",
        )
        .await
        .map_err(|e| ApiError::LlmError(e.to_string()))?;

        debug!(wait_ms = wait.as_millis() as u64, op = "llm_stream", "wait_queue");

        let exec_start = Instant::now();

        let request = ChatCompletionRequest {
            messages,
            max_tokens: self.config.max_tokens,
            temperature: 0.7,
            stream: true,
        };

        let response = self
            .client
            .post(&format!("{}/v1/chat/completions", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| ApiError::LlmError(format!("Failed to call LLM API: {}", e)))?;

        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "llm_stream", "exec");

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!("LLM API error: {} - {}", status, body)));
        }

        let byte_stream = response.bytes_stream();

        // SSE buffering (handles split lines across frames)
        let parsed_stream = futures::stream::unfold(
            (byte_stream, String::new(), permit),
            |(mut stream, mut buf, permit)| async move {
                use futures::StreamExt;

                loop {
                    match stream.next().await {
                        Some(Ok(bytes)) => {
                            buf.push_str(&String::from_utf8_lossy(&bytes));

                            while let Some(nl) = buf.find('\n') {
                                let mut line = buf[..nl].to_string();
                                buf.drain(..=nl);

                                if line.ends_with('\r') {
                                    line.pop();
                                }

                                if !line.starts_with("data: ") {
                                    continue;
                                }

                                let json_str = line.trim_start_matches("data: ").trim();

                                if json_str == "[DONE]" {
                                    // permit dropped here automatically when state is dropped
                                    return None;
                                }

                                if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(json_str) {
                                    if let Some(content) = chunk
                                        .choices
                                        .first()
                                        .and_then(|c| c.delta.content.as_ref())
                                    {
                                        return Some((Ok(content.clone()), (stream, buf, permit)));
                                    }
                                }
                            }

                            // belum ketemu payload valid, baca frame berikutnya
                            continue;
                        }
                        Some(Err(e)) => {
                            return Some((
                                Err(ApiError::LlmError(format!("Stream error: {}", e))),
                                (stream, buf, permit),
                            ));
                        }
                        None => return None,
                    }
                }
            },
        );

        Ok(Box::pin(parsed_stream))
    }

    async fn generate_chat_with(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
        temperature: f32,
    ) -> Result<String, ApiError> {
        debug!(
            "Starting chat generation with {} messages (max_tokens={}, temp={})",
            messages.len(),
            max_tokens,
            temperature
        );

        let (_permit, wait) = Limiters::acquire_timed(
            self.limiters.llm_generate.clone(),
            self.limiters.acquire_timeout,
            "llm_generate",
        )
        .await
        .map_err(|e| ApiError::LlmError(e.to_string()))?;

        debug!(wait_ms = wait.as_millis() as u64, op = "llm_generate", "wait_queue");

        let exec_start = Instant::now();

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

        debug!(exec_ms = exec_start.elapsed().as_millis() as u64, op = "llm_generate", "exec");

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::LlmError(format!("LLM API error: {} - {}", status, body)));
        }

        #[derive(Deserialize)]
        struct ChatCompletionResponse {
            choices: Vec<Choice>,
        }
        #[derive(Deserialize)]
        struct Choice {
            message: Message,
        }
        #[derive(Deserialize)]
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
```

***

## 9) UPDATE `src/main.rs` (wiring constructor baru)

Yang perlu kamu ubah (intinya):

1) Setelah `let settings = Settings::load()?;` buat limiter:

```rust
let limiters = Arc::new(utils::limiters::Limiters::new(&settings.limits));
```

1) Update constructor services:

```rust
let embedding_service = Arc::new(EmbeddingService::new(
    settings.embedding.base_url.clone(),
    settings.embedding.clone(),
    limiters.clone(),
));

let llm_service = Arc::new(LlmService::new(
    settings.llm.clone(),
    settings.prompts.context_extraction_system_prompt.clone(),
    limiters.clone(),
));

let rag_service = Arc::new(RagService::new(
    repository.clone(),
    embedding_service.clone(),
    llm_service.clone(),
    settings.rag.clone(),
    limiters.clone(),
));
```

1) Masukkan `limiters` ke `AppState`:

```rust
let app_state = AppState {
    // ...
    limiters: limiters.clone(),
};
```

***
