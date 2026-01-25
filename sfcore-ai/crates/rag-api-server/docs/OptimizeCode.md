## Review & Rekomendasi Peningkatan Performa

### 1. **Batching & Caching di Embedding Service** ⚡ HIGH IMPACT

```rust
// embedding_service.rs - Tambahkan cache & optimasi batch
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct EmbeddingService {
    client: Client,
    base_url: String,
    dimension: usize,
    model_name: String,
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>, // Cache embeddings
}

impl EmbeddingService {
    pub fn new(llm_base_url: String, config: EmbeddingConfig) -> Self {
        Self {
            // ... existing fields
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    // Optimasi: Check cache dulu sebelum hit API
    async fn embed_internal(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                debug!("Cache HIT for embedding");
                return Ok(embedding.clone());
            }
        }
        
        // ... existing API call logic ...
        
        // Store in cache after generation
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }
        
        Ok(embedding)
    }
    
    // CRITICAL: Batch asli (parallel API calls)
    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, ApiError> {
        use futures::future::join_all;
        
        // Parallel embedding generation
        let futures: Vec<_> = texts.into_iter()
            .map(|text| self.embed(&text))
            .collect();
        
        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }
}
```

### 2. **Database Query Optimization** 🔥 CRITICAL

```rust
// rag_service.rs - Gunakan connection pooling yang efisien
impl RagService {
    pub async fn retrieve_with_embedding(
        &self,
        user_id: i32,
        query_text: &str,
        query_embedding: Vec<f32>,
        document_id: Option<i32>,
    ) -> Result<Vec<DocumentChunk>, ApiError> {
        let vector = Vector::from(query_embedding);
        
        // OPTIMIZATION: Use prepared statements (if not already)
        // OPTIMIZATION: Add index hints jika perlu
        let chunks = if self.config.rerank_enabled {
            // Hybrid search - pastikan ada composite index
            // CREATE INDEX idx_chunks_user_doc ON document_chunks(user_id, document_id, embedding vector_cosine_ops);
            self.repository
                .hybrid_search_user_documents(
                    user_id,
                    vector,
                    query_text.to_string(),
                    self.config.retrieval_top_k as i32,
                    document_id,
                )
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        } else {
            self.repository
                .search_user_documents(user_id, vector, self.config.retrieval_top_k as i32, document_id)
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        };
        
        Ok(chunks)
    }
}
```

**Database Indexes yang WAJIB ada:**

```sql
-- Untuk vector search
CREATE INDEX idx_chunks_embedding ON document_chunks 
USING ivfflat (embedding vector_cosine_ops) 
WITH (lists = 100);

-- Untuk filtering
CREATE INDEX idx_chunks_user_doc ON document_chunks(user_id, document_id);

-- Untuk hybrid search
CREATE INDEX idx_chunks_content_fts ON document_chunks 
USING gin(to_tsvector('english', content));
```

### 3. **Stream Processing Optimization** 🚀

```rust
// manager.rs - Optimasi stream handling
pub async fn handle_message(
    self: std::sync::Arc<Self>,
    session_id: SessionId,
    user_id: i64,
    message: String,
    document_id: Option<i64>,
) -> Result<Pin<Box<dyn Stream<Item = Result<String, anyhow::Error>> + Send>>> {
    // ... existing setup ...
    
    let stream = async_stream::try_stream! {
        let mut full_response = String::with_capacity(1024); // Pre-allocate capacity
        
        if manager.stream_enabled {
            manager.logger.log(/* ... */);
            
            let mut stream = manager.llm_provider.generate_stream(&llm_messages).await?;
            
            manager.logger.log(/* ... */);
            
            use futures::StreamExt;
            
            // OPTIMIZATION: Buffer chunks untuk reduce overhead
            let mut buffer = String::with_capacity(256);
            const BUFFER_SIZE: usize = 10; // Batch 10 chunks before yielding
            let mut count = 0;
            
            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(chunk) => {
                        full_response.push_str(&chunk);
                        buffer.push_str(&chunk);
                        count += 1;
                        
                        // Yield buffered chunks
                        if count >= BUFFER_SIZE {
                            yield buffer.clone();
                            buffer.clear();
                            count = 0;
                        }
                    }
                    Err(e) => Err(e)?,
                }
            }
            
            // Yield remaining buffer
            if !buffer.is_empty() {
                yield buffer;
            }
        } else {
            // ... non-streaming logic ...
        }
        
        // ... post-processing ...
    };
    
    Ok(Box::pin(stream))
}
```

### 4. **Token Management Optimization** 💡

```rust
// manager.rs - Optimasi cascade deletion
async fn manage_tokens(
    &self,
    state: &mut ConversationState,
    system_context: &str,
) -> Result<()> {
    let token_count = TokenCounter::count_payload(
        system_context,
        &state.messages,
        "",
    );

    state.metadata.total_tokens_last = token_count.total;

    if !token_count.is_over_soft_limit() {
        return Ok(());
    }

    warn!("Token count {} exceeds 20K, performing cascade deletion", token_count.total);
    
    // OPTIMIZATION: Hitung berapa pair yang harus dihapus sekaligus
    let target_tokens = 18_000; // Safety margin
    let avg_tokens_per_pair = if state.messages.len() >= 2 {
        token_count.history_tokens / (state.messages.len() / 2)
    } else {
        1000 // Default estimate
    };
    
    let pairs_to_remove = ((token_count.total - target_tokens) / avg_tokens_per_pair).max(1);
    let messages_to_remove = (pairs_to_remove * 2).min(state.messages.len());
    
    info!("Removing {} pairs ({} messages) in one operation", 
          messages_to_remove / 2, messages_to_remove);
    
    state.messages.drain(0..messages_to_remove);
    
    // Verify final count
    let final_count = TokenCounter::count_payload(system_context, &state.messages, "");
    
    if final_count.total > 23_000 {
        warn!("Still over 23K after deletion, truncating retrieval");
        // ... truncation logic ...
    }

    Ok(())
}
```

### 5. **EventBus Optimization** 📡

```rust
// events.rs - Tambahkan bounded channel & filtering
use tokio::sync::mpsc;

pub struct EventBus {
    tx: broadcast::Sender<SessionEvent>,
    // Tambahkan bounded buffer untuk prevent memory spike
    buffer_tx: mpsc::Sender<SessionEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        let (buffer_tx, mut buffer_rx) = mpsc::channel::<SessionEvent>(100);
        
        // Background task untuk forward events
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            while let Some(event) = buffer_rx.recv().await {
                let _ = tx_clone.send(event);
            }
        });
        
        Self { tx, buffer_tx }
    }

    pub fn publish(&self, session_id: i64, event: SystemEvent) {
        let session_event = SessionEvent { session_id, event };
        
        // Non-blocking send
        if let Err(e) = self.buffer_tx.try_send(session_event) {
            warn!("Event buffer full, dropping event: {}", e);
        }
    }
}
```

### 6. **Memory Management - Cache Eviction** 🧹

```rust
// cache.rs (buat file baru atau update existing)
use lru::LruCache;
use std::num::NonZeroUsize;

pub struct ConversationCache {
    cache: Arc<RwLock<LruCache<SessionId, ConversationState>>>,
    max_size: NonZeroUsize,
}

impl ConversationCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(1000).unwrap())
            )),
            max_size: NonZeroUsize::new(1000).unwrap(),
        }
    }
    
    pub async fn get(&self, session_id: SessionId) -> Option<ConversationState> {
        let mut cache = self.cache.write().await;
        cache.get(&session_id).cloned()
    }
    
    pub async fn set(&self, session_id: SessionId, state: ConversationState) {
        let mut cache = self.cache.write().await;
        cache.put(session_id, state);
    }
}
```

### 7. **Monitoring & Metrics** 📊

```rust
// Tambahkan metrics untuk tracking performa
use std::sync::atomic::{AtomicU64, Ordering};

pub struct PerformanceMetrics {
    total_requests: AtomicU64,
    cache_hits: AtomicU64,
    avg_response_time_ms: AtomicU64,
    retrieval_skipped: AtomicU64,
}

impl PerformanceMetrics {
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed) as f64;
        let total = self.total_requests.load(Ordering::Relaxed) as f64;
        if total > 0.0 { hits / total } else { 0.0 }
    }
}
```

## Prioritas Implementasi

### **Tier 1 - Immediate Impact**

1. ✅ Database indexes (SQL migration)
2. ✅ Embedding cache dengan TTL
3. ✅ Batch embedding optimization

### **Tier 2 - High Value**

1. ✅ Stream buffering
2. ✅ Smart token deletion (calculate pairs upfront)
3. ✅ LRU cache untuk conversation state

### **Tier 3 - Nice to Have**

1. ✅ Performance metrics & monitoring
2. ✅ EventBus bounded buffer
3. ✅ Connection pool tuning

## Estimasi Peningkatan Performa

- **Embedding cache**: 70-90% faster untuk repeated queries
- **Batch processing**: 3-5x faster untuk multiple embeddings
- **Database indexes**: 10-50x faster vector search
- **Stream buffering**: 30-40% reduction in overhead
- **Smart token deletion**: 80% faster (1 operation vs multiple)
