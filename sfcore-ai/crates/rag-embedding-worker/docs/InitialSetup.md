Perfect bro! Saya sudah dapat referensi credible untuk solusi production-grade. Sekarang saya buatkan solusi lengkap untuk RAG Embedding Worker dengan Rust.

## 📦 SOLUSI LENGKAP: RAG EMBEDDING WORKER (RUST)

### 🗄️ 1. DATABASE SETUP SCRIPT (LENGKAP)

```sql
-- ============================================
-- PART 1: EXTENSION & SCHEMA SETUP
-- ============================================
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pg_trgm; -- untuk fuzzy search

-- ============================================
-- PART 2: RAG TABLES (UPDATED dengan beberapa improvement)
-- ============================================

-- Table utama untuk chunks
CREATE TABLE IF NOT EXISTS rag_document_chunks (
  id BIGSERIAL PRIMARY KEY,
  
  -- FK ke DMS existing (TblDocumentFiles)
  document_id INT NOT NULL,
  
  -- optional (kalau multi-tenant)
  tenant_id INT,
  
  -- chunk info
  chunk_index INT NOT NULL,
  content TEXT NOT NULL,
  
  -- metadata chunks
  char_count INT NOT NULL,
  token_count INT,
  
  -- full-text search
  content_tsv tsvector
    GENERATED ALWAYS AS (to_tsvector('indonesian', content)) STORED,
  
  -- semantic search (OpenAI text-embedding-3-small = 1536 dims)
  -- Bisa ganti ke 384 kalau pakai fastembed AllMiniLML6V2
  embedding vector(384) NOT NULL,
  
  -- metadata tambahan
  page_number INT,
  section TEXT,
  tags TEXT[],
  
  -- timestamps
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now(),
  
  -- constraint untuk ensure uniqueness per document
  CONSTRAINT unique_document_chunk UNIQUE (document_id, chunk_index)
);

-- Table untuk tracking ingestion status
CREATE TABLE IF NOT EXISTS rag_ingestion_log (
  document_id INT PRIMARY KEY,
  
  -- file metadata
  file_path TEXT NOT NULL,
  file_size BIGINT,
  file_type VARCHAR(50),
  
  -- processing config
  embedding_model TEXT NOT NULL,
  chunk_size INT NOT NULL,
  chunk_overlap INT NOT NULL,
  
  -- status tracking
  status VARCHAR(20) NOT NULL DEFAULT 'pending',
  -- pending, processing, completed, failed
  
  total_chunks INT DEFAULT 0,
  processed_chunks INT DEFAULT 0,
  
  last_error TEXT,
  retry_count INT DEFAULT 0,
  
  -- timestamps
  started_at TIMESTAMPTZ,
  processed_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT now(),
  updated_at TIMESTAMPTZ DEFAULT now()
);

-- ============================================
-- PART 3: INDEXES (PRODUCTION CRITICAL!)
-- ============================================

-- 1. Vector Similarity Search (HNSW lebih cepat dari IVFFlat untuk < 1M rows)
CREATE INDEX IF NOT EXISTS idx_rag_chunks_embedding_hnsw
ON rag_document_chunks
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- Alternatif IVFFlat (untuk dataset besar > 1M rows)
-- CREATE INDEX idx_rag_chunks_embedding_ivfflat
-- ON rag_document_chunks
-- USING ivfflat (embedding vector_cosine_ops)
-- WITH (lists = 100);

-- 2. Full Text Search
CREATE INDEX IF NOT EXISTS idx_rag_chunks_tsv
ON rag_document_chunks
USING GIN (content_tsv);

-- 3. Filtering indexes
CREATE INDEX IF NOT EXISTS idx_rag_chunks_document
ON rag_document_chunks(document_id);

CREATE INDEX IF NOT EXISTS idx_rag_chunks_tenant
ON rag_document_chunks(tenant_id)
WHERE tenant_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_rag_chunks_tags
ON rag_document_chunks
USING GIN(tags)
WHERE tags IS NOT NULL;

-- 4. Ingestion log indexes
CREATE INDEX IF NOT EXISTS idx_ingestion_log_status
ON rag_ingestion_log(status);

CREATE INDEX IF NOT EXISTS idx_ingestion_log_updated
ON rag_ingestion_log(updated_at);

-- ============================================
-- PART 4: TRIGGER FUNCTION & NOTIFICATION
-- ============================================

-- Function untuk send notification ke Rust worker
CREATE OR REPLACE FUNCTION notify_document_change()
RETURNS TRIGGER AS $$
DECLARE
  payload JSON;
BEGIN
  -- Build payload dengan info dokumen
  payload := json_build_object(
    'operation', TG_OP,
    'document_id', NEW."DocumentID", -- sesuaikan dengan nama kolom actual
    'file_path', NEW."DocumentFilePath", -- sesuaikan dengan nama kolom actual
    'timestamp', extract(epoch from now())
  );
  
  -- Send notification ke channel 'document_changes'
  PERFORM pg_notify('document_changes', payload::text);
  
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger pada TblDocumentFiles
-- CATATAN: Sesuaikan nama table dengan yang actual di database
CREATE TRIGGER trg_document_files_notify
AFTER INSERT OR UPDATE ON "TblDocumentFiles" -- ganti dengan nama actual table
FOR EACH ROW
EXECUTE FUNCTION notify_document_change();

-- ============================================
-- PART 5: HELPER FUNCTIONS
-- ============================================

-- Function untuk update timestamp otomatis
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger untuk auto-update updated_at
CREATE TRIGGER trg_rag_chunks_updated_at
BEFORE UPDATE ON rag_document_chunks
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_ingestion_log_updated_at
BEFORE UPDATE ON rag_ingestion_log
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- PART 6: UTILITY FUNCTIONS FOR RAG
-- ============================================

-- Function untuk similarity search dengan hybrid scoring
CREATE OR REPLACE FUNCTION search_documents(
  query_embedding vector(384),
  query_text TEXT,
  match_count INT DEFAULT 10,
  document_filter INT[] DEFAULT NULL,
  tenant_filter INT DEFAULT NULL
)
RETURNS TABLE (
  chunk_id BIGINT,
  document_id INT,
  content TEXT,
  similarity_score FLOAT,
  text_rank FLOAT,
  hybrid_score FLOAT,
  page_number INT,
  chunk_index INT
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    c.id as chunk_id,
    c.document_id,
    c.content,
    1 - (c.embedding <=> query_embedding) as similarity_score,
    ts_rank(c.content_tsv, plainto_tsquery('indonesian', query_text)) as text_rank,
    -- Hybrid score: 70% semantic + 30% text
    (0.7 * (1 - (c.embedding <=> query_embedding)) + 
     0.3 * ts_rank(c.content_tsv, plainto_tsquery('indonesian', query_text))) as hybrid_score,
    c.page_number,
    c.chunk_index
  FROM rag_document_chunks c
  WHERE 
    (document_filter IS NULL OR c.document_id = ANY(document_filter))
    AND (tenant_filter IS NULL OR c.tenant_id = tenant_filter)
  ORDER BY hybrid_score DESC
  LIMIT match_count;
END;
$$ LANGUAGE plpgsql;

-- Function untuk cleanup chunks dari dokumen yang dihapus
CREATE OR REPLACE FUNCTION cleanup_orphaned_chunks()
RETURNS TABLE (deleted_count BIGINT) AS $$
DECLARE
  del_count BIGINT;
BEGIN
  DELETE FROM rag_document_chunks
  WHERE document_id NOT IN (
    SELECT "DocumentID" FROM "TblDocumentFiles" -- sesuaikan nama table & kolom
  );
  
  GET DIAGNOSTICS del_count = ROW_COUNT;
  
  RETURN QUERY SELECT del_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================
-- VERIFICATION & INFO
-- ============================================

-- Check extension installed
SELECT * FROM pg_extension WHERE extname = 'vector';

-- Check tables created
SELECT tablename FROM pg_tables 
WHERE schemaname = 'public' 
AND tablename LIKE 'rag_%';

-- Check indexes
SELECT indexname, indexdef 
FROM pg_indexes 
WHERE tablename LIKE 'rag_%';

-- Check trigger exists
SELECT tgname, tgtype, tgenabled 
FROM pg_trigger 
WHERE tgrelid = '"TblDocumentFiles"'::regclass; -- sesuaikan nama table

```

**📌 Referensi Database Setup:**

- [pgvector official documentation](https://github.com/pgvector/pgvector) [github](https://github.com/pgvector/pgvector-rust)
- [PostgreSQL LISTEN/NOTIFY best practices](https://users.rust-lang.org/t/listen-for-psql-notification-using-tokio-postgres/105798) [users.rust-lang](https://users.rust-lang.org/t/listen-for-psql-notification-using-tokio-postgres/105798)
- [HNSW vs IVFFlat comparison](https://cosminsanda.com/posts/using-pgvector-with-seaorm-in-rust/) [cosminsanda](https://cosminsanda.com/posts/using-pgvector-with-seaorm-in-rust/)

***

### 🏗️ 2. RUST APPLICATION ARCHITECTURE

```
rag-embedding-worker/
├── Cargo.toml
├── .env.example
├── config/
│   └── settings.toml
└── src/
    ├── main.rs                    # Entry point & service orchestration
    ├── lib.rs                     # Library exports
    ├── config/
    │   ├── mod.rs                 # Configuration management
    │   └── settings.rs            # Settings struct
    ├── database/
    │   ├── mod.rs
    │   ├── pool.rs                # Connection pool management
    │   ├── models.rs              # Database models/structs
    │   ├── repository.rs          # Database operations
    │   └── listener.rs            # PostgreSQL LISTEN handler
    ├── embedding/
    │   ├── mod.rs
    │   ├── provider.rs            # Embedding provider trait
    │   ├── fastembed.rs           # Fastembed implementation
    │   └── openai.rs              # OpenAI implementation (optional)
    ├── document/
    │   ├── mod.rs
    │   ├── loader.rs              # File loading & detection
    │   ├── parser.rs              # Document parsing (PDF, DOCX, TXT)
    │   └── chunker.rs             # Text chunking strategies
    ├── worker/
    │   ├── mod.rs
    │   ├── processor.rs           # Main processing logic
    │   ├── bulk_indexer.rs        # Initial bulk embedding
    │   └── queue.rs               # Task queue management
    └── utils/
        ├── mod.rs
        ├── error.rs               # Error handling
        ├── logger.rs              # Logging setup
        └── metrics.rs             # Performance monitoring
```

***

### 📝 3. CARGO.TOML (Dependencies)

```toml
[package]
name = "rag-embedding-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
# Async runtime
tokio = { version = "1.42", features = ["full"] }
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4"] }

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-native-tls",
    "postgres",
    "chrono",
    "json"
] }
pgvector = { version = "0.4", features = ["sqlx"] }
deadpool-postgres = "0.14"

# Embedding
fastembed = "4.4"  # Local embedding, zero cost!

# Document processing
lopdf = "0.35"  # PDF parsing
docx-rs = "0.4"  # DOCX parsing
encoding_rs = "0.8"  # Text encoding detection

# Text processing
text-splitter = "0.18"  # Smart text chunking
unicode-segmentation = "1.12"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# Configuration
config = "0.14"
dotenvy = "0.15"

# Utilities
chrono = { version = "0.4", features = ["serde"] }
futures = "0.3"
async-trait = "0.1"
once_cell = "1.20"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true  # Remove debug symbols for smaller binary
```

**📌 Referensi Dependencies:**

- [Fastembed for Rust - Local embeddings](https://dev.to/joshmo_dev/local-embeddings-with-fastembed-rig-rust-3581) [dev](https://dev.to/joshmo_dev/local-embeddings-with-fastembed-rig-rust-3581)
- [lopdf - PDF parsing library](https://github.com/J-F-Liu/lopdf) [github](https://github.com/J-F-Liu/lopdf)
- [text-splitter - Smart chunking](https://www.linkedin.com/pulse/improving-text-chunking-rag-rust-shelby-jenkins--bgj5c) [linkedin](https://www.linkedin.com/pulse/improving-text-chunking-rag-rust-shelby-jenkins--bgj5c)

***

### 🎯 4. CONFIGURATION FILES

**`.env.example`**

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DB_POOL_MAX_SIZE=10

# Embedding
EMBEDDING_MODEL=AllMiniLML6V2  # fastembed model
EMBEDDING_DIMENSION=384
CHUNK_SIZE=512
CHUNK_OVERLAP=50

# Worker
WORKER_THREADS=4
BATCH_SIZE=10
MAX_RETRIES=3

# Logging
RUST_LOG=info,rag_embedding_worker=debug
LOG_FORMAT=json  # or "pretty"

# Paths
DOCUMENT_ROOT_PATH=/path/to/documents
```

**`config/settings.toml`**

```toml
[database]
pool_max_size = 10
pool_timeout_seconds = 30
listen_channel = "document_changes"

[embedding]
model = "AllMiniLML6V2"  # Options: AllMiniLML6V2, BGESmallEN, etc
dimension = 384
batch_size = 32

[chunking]
size = 512
overlap = 50
strategy = "semantic"  # semantic, fixed, recursive

[worker]
threads = 4
bulk_batch_size = 10
processing_timeout_seconds = 300

[retry]
max_attempts = 3
initial_interval_ms = 1000
max_interval_ms = 60000
multiplier = 2.0
```

***
