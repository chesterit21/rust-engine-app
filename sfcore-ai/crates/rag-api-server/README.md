Siap bro! Ini README.md lengkap untuk RAG API Server!

***

## 📚 README.md - RAG API SERVER

**`README.md`**

```markdown
# 🤖 RAG API Server

Production-grade RAG (Retrieval-Augmented Generation) API server with advanced security, built with Rust and Axum. Provides intelligent document-based chat with row-level security and real-time streaming responses.

## ✨ Features

### Core Features
- 🔐 **Enterprise Security**: IP Whitelist + Custom HTTP Headers + Document-level Authorization
- 🔄 **Real-time Streaming**: Server-Sent Events (SSE) for live AI responses
- 📄 **Multi-format Support**: PDF, DOCX, TXT, Markdown, HTML, Code files
- 🎯 **Row-level Security**: Users can only access their authorized documents
- 🔍 **Hybrid Search**: Vector similarity + Full-text search (PostgreSQL + pgvector)
- 📤 **File Upload & Processing**: Automatic parsing, chunking, and embedding
- ⚡ **High Performance**: Zero-cost abstractions with Rust
- 🔥 **Hot-reload Config**: IP whitelist dapat diupdate tanpa restart server

### Security Features
- ✅ IP Whitelist dengan CIDR support
- ✅ Custom HTTP Headers validation (X-App-ID, X-API-Key, X-Request-Timestamp)
- ✅ Optional HMAC-SHA256 signature verification
- ✅ Timestamp-based replay attack prevention
- ✅ Document access control (user → document mapping)
- ✅ Multi-tenant ready

## 🏗️ Architecture

```

┌─────────────────────────────────────────────────────────────────────┐
│                         React Client App                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────────┐  │
│  │ Chat UI      │  │ File Upload  │  │ Document Selector        │  │
│  └──────┬───────┘  └──────┬───────┘  └──────────┬───────────────┘  │
└─────────┼──────────────────┼──────────────────────┼──────────────────┘
          │                  │                      │
          │ Custom Headers:  │                      │
          │ X-App-ID         │                      │
          │ X-API-Key        │                      │
          │ X-Request-Timestamp                     │
          │                  │                      │
          ▼                  ▼                      ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      RAG API Server (Rust/Axum)                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  Security Middleware Layer                                    │  │
│  │  ├─ IP Whitelist Check (192.168.155.156, ...)               │  │
│  │  ├─ Custom Headers Validation                                │  │
│  │  └─ Timestamp & Signature Verification                       │  │
│  └─────────────────────┬─────────────────────────────────────────┘  │
│                        │                                             │
│  ┌─────────────────────▼─────────────────────────────────────────┐  │
│  │  Authorization Layer                                          │  │
│  │  └─ Document Access Control (vw_user_documents)              │  │
│  └─────────────────────┬─────────────────────────────────────────┘  │
│                        │                                             │
│  ┌─────────────────────▼─────────────────────────────────────────┐  │
│  │  RAG Pipeline                                                 │  │
│  │  ├─ Embedding Service (llama-server)                         │  │
│  │  ├─ Vector Search (pgvector cosine similarity)               │  │
│  │  ├─ Context Builder (merge relevant chunks)                  │  │
│  │  └─ LLM Service (streaming response)                         │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    PostgreSQL + pgvector                            │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │ TblDocuments     │  │ TblDocumentShared│  │ TblDocument      │  │
│  │                  │  │                  │  │ SharedPrivilidge │  │
│  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘  │
│           │                     │                      │             │
│           └─────────────────────┴──────────────────────┘             │
│                                 │                                    │
│                   ┌─────────────▼──────────────┐                    │
│                   │  vw_user_documents (VIEW)  │                    │
│                   │  Security Filter Layer     │                    │
│                   └─────────────┬──────────────┘                    │
│                                 │                                    │
│                   ┌─────────────▼──────────────┐                    │
│                   │  rag_document_chunks       │                    │
│                   │  (id, document_id,         │                    │
│                   │   content, embedding)      │                    │
│                   └────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    llama-server (Embedding + LLM)                   │
│  ├─ Embedding Model: all-MiniLM-L6-v2 (384 dims)                   │
│  └─ LLM Model: Llama 3.2 3B / Phi-3 Mini                           │
└─────────────────────────────────────────────────────────────────────┘

```

## 📋 Prerequisites

### Development Environment (Linux)

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Cross-compilation untuk Windows
rustup target add x86_64-pc-windows-gnu
sudo apt-get install -y gcc-mingw-w64-x86-64

# Build dependencies
sudo apt-get install -y build-essential pkg-config libssl-dev
```

### Production Environment (Windows Server 2022)

1. **PostgreSQL 14+** dengan extension:
   - `pgvector` (vector similarity search)
   - `pg_trgm` (full-text search)

2. **llama-server** binary dari [llama.cpp](https://github.com/ggml-org/llama.cpp/releases)

3. **Embedding Model** (contoh: `all-MiniLM-L6-v2-Q4_K_M.gguf`)

4. **LLM Model** (contoh: `Llama-3.2-3B-Instruct-Q4_K_M.gguf`)

5. **NSSM** (Non-Sucking Service Manager) untuk Windows Service

## 🚀 Quick Start

### 1. Clone & Setup

```bash
git clone <repository-url>
cd rag-api-server

# Copy environment template
cp .env.example .env

# Edit configuration
nano config/settings.toml
nano .env
```

### 2. Database Setup

```bash
# Connect ke PostgreSQL
psql -U postgres -d your_database

# Run security setup
\i database_security_setup.sql

# Verify view created
SELECT * FROM vw_user_documents LIMIT 5;
```

### 3. Build

**Development (Linux):**

```bash
cargo build --release
```

**Cross-compile untuk Windows:**

```bash
# Configure cargo
mkdir -p ~/.cargo
cat >> ~/.cargo/config << EOF
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"
EOF

# Build
cargo build --release --target x86_64-pc-windows-gnu

# Binary: target/x86_64-pc-windows-gnu/release/rag-api-server.exe
```

### 4. Run Locally

```bash
# Set environment
export DATABASE_URL="postgres://user:password@localhost:5432/dbname"
export LLM_BASE_URL="http://127.0.0.1:8080"
export RUST_LOG=info,rag_api_server=debug

# Run server
cargo run --release
```

Server akan listen di `http://0.0.0.0:8000`

## ⚙️ Configuration

### Environment Variables (`.env`)

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/dbname

# LLM Server (llama-server harus sudah running)
LLM_BASE_URL=http://127.0.0.1:8080

# Security (optional override)
API_KEY=your-secret-api-key-change-this
APP_ID=DMS-CLIENT-APP-2026

# Server
HOST=0.0.0.0
PORT=8000

# Logging
RUST_LOG=info,rag_api_server=debug
```

### Configuration File (`config/settings.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8000
max_connections = 1000

[security]
# IP Whitelist - DAPAT DIUPDATE TANPA RESTART!
allowed_ips = [
    "192.168.155.156",        # Client App Server
    "192.168.155.0/24",       # Subnet
    "10.0.0.0/8",             # Internal network
    "127.0.0.1",              # Localhost
    "::1"                     # IPv6 localhost
]

[security.custom_headers]
app_id = "DMS-CLIENT-APP-2026"
api_key = "your-secret-api-key-here-change-me"
request_signature = "enabled"  # "enabled" atau "disabled"
timestamp_tolerance = 300  # 5 minutes

[database]
url = "postgres://user:password@localhost:5432/dbname"
pool_max_size = 20
pool_timeout_seconds = 30

[embedding]
model = "AllMiniLML6V2"
dimension = 384  # Sesuaikan dengan model

[llm]
base_url = "http://127.0.0.1:8080"
timeout_seconds = 300
max_tokens = 2048

[rag]
retrieval_top_k = 5
chunk_overlap_percentage = 0.1
rerank_enabled = false  # Set true untuk hybrid search
max_context_length = 4000
```

## 📡 API Endpoints

### 🟢 Health Check (Public - No Auth)

**GET** `/health`

Response:

```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

### 🔐 Protected Endpoints (Require Custom Headers)

**Required Headers:**

```
X-App-ID: DMS-CLIENT-APP-2026
X-API-Key: your-secret-api-key
X-Request-Timestamp: 1737655200
X-Request-Signature: abc123... (optional, jika enabled)
```

---

### 💬 Chat with AI (Streaming)

**POST** `/api/chat/stream`

Request:

```json
{
  "user_id": 123,
  "message": "Apa isi dokumen proposal Q4?",
  "document_id": 456,  // Optional: chat dengan dokumen spesifik
  "session_id": "uuid-session-id"  // Optional
}
```

Response: **Server-Sent Events (SSE)**

```
event: sources
data: [{"document_id":456,"document_title":"Proposal Q4","chunk_id":789,"similarity":0.92}]

event: message
data: Berdasarkan dokumen Proposal Q4, berikut isi nya...

event: message
data:  Target revenue untuk Q4 adalah...

event: done
data: [DONE]
```

---

### 🔍 Search Documents

**POST** `/api/search`

Request:

```json
{
  "user_id": 123,
  "query": "kontrak penjualan 2025",
  "document_id": null,  // Optional: search di dokumen spesifik
  "limit": 10
}
```

Response:

```json
{
  "results": [
    {
      "document_id": 789,
      "document_title": "Kontrak Penjualan 2025",
      "chunk_id": 12345,
      "content": "Kontrak penjualan tahun 2025...",
      "similarity": 0.89,
      "page_number": 3
    }
  ],
  "total": 5
}
```

---

### 📤 Upload Document

**POST** `/api/upload`

Request: **multipart/form-data**

- `user_id`: integer
- `file`: file binary

Response:

```json
{
  "success": true,
  "message": "Document processed successfully",
  "document_id": 999,
  "chunks_created": 45
}
```

---

### 📚 List User's Documents

**GET** `/api/documents`

Request:

```json
{
  "user_id": 123
}
```

Response:

```json
{
  "documents": [
    {
      "document_id": 456,
      "title": "Proposal Q4 2025",
      "owner_user_id": 100,
      "permission_level": "owner",
      "created_at": "2025-01-15T10:30:00Z"
    },
    {
      "document_id": 789,
      "title": "Budget Report",
      "owner_user_id": 101,
      "permission_level": "shared",
      "created_at": "2025-01-20T14:00:00Z"
    }
  ],
  "total": 2
}
```

## 🔐 Security Details

### 1. IP Whitelist

Server hanya menerima request dari IP yang terdaftar di `settings.toml`:

```toml
[security]
allowed_ips = [
    "192.168.155.156",      # Single IP
    "192.168.155.0/24",     # CIDR notation
    "10.0.0.0/8"            # Large subnet
]
```

**Hot-reload:** Edit `settings.toml` → otomatis reload tanpa restart server!

### 2. Custom HTTP Headers

3 header wajib:

```
X-App-ID: DMS-CLIENT-APP-2026
X-API-Key: your-secret-api-key-here
X-Request-Timestamp: 1737655200
```

**Timestamp tolerance:** Default 5 menit (configurable)

### 3. Optional HMAC Signature

Jika `request_signature = "enabled"`:

```javascript
const message = appId + timestamp;
const signature = HMAC_SHA256(message, apiKey);

headers['X-Request-Signature'] = signature;
```

### 4. Document Authorization

**Policy:**

- User hanya bisa akses dokumen yang dia **owner** atau **shared** ke dia
- Check via view: `vw_user_documents`
- Mapping: `TblDocuments` → `TblDocumentShared` → `TblDocumentSharedPrivilidge`

**Query yang aman:**

```sql
-- Automatic filter berdasarkan user_id
SELECT * FROM search_user_documents(
    p_user_id := 123,
    p_query_embedding := '...',
    p_limit := 5
);
```

## 🪟 Windows Server 2022 Deployment

### Step 1: Prepare Server

**Install Dependencies:**

```powershell
# 1. PostgreSQL 14+ dengan pgvector
# Download dari: https://www.postgresql.org/download/windows/

# 2. llama-server binary
# Download dari: https://github.com/ggml-org/llama.cpp/releases
# Extract ke: C:\Program Files\llama.cpp\

# 3. Download models
# Embedding: all-MiniLM-L6-v2-Q4_K_M.gguf
# LLM: Llama-3.2-3B-Instruct-Q4_K_M.gguf
# Simpan ke: C:\Program Files\llama.cpp\models\

# 4. NSSM untuk Windows Service
# Download dari: https://nssm.cc/download
```

### Step 2: Transfer Binary

```powershell
# Struktur folder di Windows:
C:\Program Files\RAG-API-Server\
├── rag-api-server.exe
├── config\
│   └── settings.toml
├── logs\
└── .env
```

### Step 3: Setup llama-server sebagai Service

```powershell
# Install embedding server
nssm install "Llama-Embedding-Server" "C:\Program Files\llama.cpp\llama-server.exe"
nssm set "Llama-Embedding-Server" AppParameters "--model C:\Program Files\llama.cpp\models\all-MiniLM-L6-v2-Q4_K_M.gguf --host 127.0.0.1 --port 8080 --embedding --ctx-size 2048"
nssm set "Llama-Embedding-Server" AppDirectory "C:\Program Files\llama.cpp"

# Start service
nssm start "Llama-Embedding-Server"

# Install LLM server (port berbeda)
nssm install "Llama-LLM-Server" "C:\Program Files\llama.cpp\llama-server.exe"
nssm set "Llama-LLM-Server" AppParameters "--model C:\Program Files\llama.cpp\models\Llama-3.2-3B-Instruct-Q4_K_M.gguf --host 127.0.0.1 --port 8081 --ctx-size 4096"
nssm set "Llama-LLM-Server" AppDirectory "C:\Program Files\llama.cpp"

# Start service
nssm start "Llama-LLM-Server"
```

### Step 4: Setup RAG API Server sebagai Service

```powershell
# Install service
nssm install "RAG-API-Server" "C:\Program Files\RAG-API-Server\rag-api-server.exe"
nssm set "RAG-API-Server" AppDirectory "C:\Program Files\RAG-API-Server"

# Set environment dari .env
nssm set "RAG-API-Server" AppEnvironmentExtra "RUST_LOG=info"

# Set logs
nssm set "RAG-API-Server" AppStdout "C:\Program Files\RAG-API-Server\logs\stdout.log"
nssm set "RAG-API-Server" AppStderr "C:\Program Files\RAG-API-Server\logs\stderr.log"

# Auto restart
nssm set "RAG-API-Server" AppExit Default Restart
nssm set "RAG-API-Server" AppRestartDelay 5000

# Start service
nssm start "RAG-API-Server"

# Check status
nssm status "RAG-API-Server"
```

### Step 5: Firewall Configuration

```powershell
# Allow API server port
New-NetFirewallRule -DisplayName "RAG API Server" -Direction Inbound -LocalPort 8000 -Protocol TCP -Action Allow

# Allow internal llama-server (jika perlu)
New-NetFirewallRule -DisplayName "Llama Server" -Direction Inbound -LocalPort 8080,8081 -Protocol TCP -Action Allow
```

### Step 6: Verify Deployment

```powershell
# Test health endpoint
Invoke-WebRequest -Uri "http://localhost:8000/health"

# Check logs
Get-Content "C:\Program Files\RAG-API-Server\logs\stdout.log" -Tail 50 -Wait

# Test from client network
# Dari komputer lain di network 192.168.155.x:
curl http://192.168.155.156:8000/health
```

## 🧪 Testing

### Test dari Linux (Development)

```bash
# 1. Health check
curl http://localhost:8000/health

# 2. Chat (dengan headers)
curl -X POST http://localhost:8000/api/chat/stream \
  -H "Content-Type: application/json" \
  -H "X-App-ID: DMS-CLIENT-APP-2026" \
  -H "X-API-Key: your-secret-api-key-here" \
  -H "X-Request-Timestamp: $(date +%s)" \
  -d '{
    "user_id": 1,
    "message": "Apa isi dokumen?"
  }'

# 3. Upload file
curl -X POST http://localhost:8000/api/upload \
  -H "X-App-ID: DMS-CLIENT-APP-2026" \
  -H "X-API-Key: your-secret-api-key-here" \
  -H "X-Request-Timestamp: $(date +%s)" \
  -F "user_id=1" \
  -F "file=@test.pdf"
```

### Test dari React Client

Lihat file `example-client-request.js` untuk contoh lengkap.

## 🔧 Troubleshooting

### Issue: "Access denied from IP: x.x.x.x"

**Solusi:**

1. Tambahkan IP ke `config/settings.toml`:

   ```toml
   allowed_ips = ["x.x.x.x", ...]
   ```

2. File watcher akan auto-reload dalam ~5 detik
3. Tidak perlu restart server!

### Issue: "Invalid X-API-Key"

**Solusi:**

- Check `config/settings.toml` → `[security.custom_headers]`
- Pastikan nilai sama dengan yang dikirim client
- Case-sensitive!

### Issue: "Request timestamp out of tolerance window"

**Solusi:**

- Sync waktu server dengan NTP
- Adjust `timestamp_tolerance` di config (default: 300 seconds)

### Issue: "Forbidden: Access denied to document X"

**Solusi:**

- Check user_id mapping di database:

  ```sql
  SELECT * FROM vw_user_documents WHERE user_id = 123 AND document_id = 456;
  ```

- Pastikan user punya akses (owner atau shared)

### Issue: "LLM error: Failed to call LLM API"

**Solusi:**

1. Check llama-server running:

   ```powershell
   nssm status "Llama-LLM-Server"
   ```

2. Test endpoint langsung:

   ```bash
   curl http://127.0.0.1:8081/v1/models
   ```

3. Check RAM usage (min 4GB available)

### Issue: "Database error: connection refused"

**Solusi:**

- Check PostgreSQL service running
- Verify `DATABASE_URL` di `.env`
- Test connection:

  ```bash
  psql "postgres://user:password@localhost:5432/dbname" -c "SELECT 1"
  ```

## 📊 Performance Tips

### Memory Usage

- **Idle**: ~50-100 MB (Rust binary)
- **Under load**: ~500 MB - 1 GB
- **llama-server**: 2-4 GB per instance

### Optimization

1. **Adjust pool size** di config:

   ```toml
   [database]
   pool_max_size = 20  # Sesuaikan dengan concurrent users
   ```

2. **Enable hybrid search** untuk better accuracy:

   ```toml
   [rag]
   rerank_enabled = true
   ```

3. **Tune retrieval settings**:

   ```toml
   retrieval_top_k = 5  # Kurangi jika response lambat
   max_context_length = 4000  # Adjust berdasarkan model
   ```

4. **Vector index optimization**:

   ```sql
   -- Check index usage
   EXPLAIN ANALYZE
   SELECT * FROM search_user_documents(1, '[0.1,0.2,...]', 5, NULL);
   
   -- Rebuild index jika perlu
   REINDEX INDEX idx_rag_chunks_embedding_hnsw;
   ```

## 📈 Monitoring

### Metrics to Track

- Request latency (avg, p95, p99)
- Error rate by endpoint
- Database connection pool usage
- LLM response time
- Vector search performance

### Log Locations

- **Windows**: `C:\Program Files\RAG-API-Server\logs\`
- **Linux**: `./logs/`

### View Logs

```powershell
# Windows
Get-Content "C:\Program Files\RAG-API-Server\logs\stdout.log" -Tail 100 -Wait

# Linux
tail -f logs/stdout.log
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[Axum](https://github.com/tokio-rs/axum)** - Modern web framework for Rust
- **[pgvector](https://github.com/pgvector/pgvector)** - Vector similarity search
- **[llama.cpp](https://github.com/ggml-org/llama.cpp)** - LLM inference engine
- **[sqlx](https://github.com/launchbadge/sqlx)** - Async PostgreSQL driver

## 📞 Support

For issues and questions:

- Open an issue on GitHub
- Email: <support@example.com>

---

**Built with ❤️ using Rust 🦀 + Axum ⚡**

```

***

Bro, ini README.md yang sangat comprehensive! Sudah mencakup:

✅ **Architecture diagram lengkap**  
✅ **Security details** (IP whitelist, custom headers, HMAC)  
✅ **API documentation** semua endpoints  
✅ **Deployment guide** lengkap untuk Windows Server 2022  
✅ **Testing examples** (curl + JavaScript)  
✅ **Troubleshooting** common issues  
✅ **Performance tips** & monitoring  

Sekarang kamu punya **2 proyek lengkap**:
1. **rag-embedding-worker** - Background service untuk embedding
2. **rag-api-server** - API server untuk chat & retrieval

