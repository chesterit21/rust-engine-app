Perfect bro! Sekarang saya buatkan solusi lengkapnya. Ini akan saya bagi jadi beberapa bagian:

***

## 📚 PART 1: README.md - RAG EMBEDDING WORKER

**`README.md`**

```markdown
# 🚀 RAG Embedding Worker

Production-grade document embedding worker service written in Rust for RAG (Retrieval-Augmented Generation) pipelines.

## ✨ Features

- **🎯 On-Demand Embedding**: Spawn llama-server only when needed to minimize memory usage (< 6GB RAM)
- **👂 PostgreSQL LISTEN/NOTIFY**: Real-time document change detection with zero polling
- **📦 Bulk Indexing**: Initial processing of existing documents with batch optimization
- **🔄 Multi-Format Support**: PDF, DOCX, Markdown, HTML, Code files, and more
- **⚡ Memory Efficient**: Zero-cost abstractions with Rust's ownership model
- **📊 Metrics & Monitoring**: Built-in performance tracking and logging
- **🔐 Production Ready**: Error handling, retry logic, and graceful shutdown

## 🏗️ Architecture

```

┌─────────────────────────────────────────────────────────────────┐
│                        PostgreSQL Database                       │
│  ┌──────────────────┐  ┌──────────────────┐  ┌───────────────┐ │
│  │ TblDocumentFiles │  │ rag_document_     │  │ rag_ingestion_│ │
│  │                  │  │    chunks         │  │     log       │ │
│  └────────┬─────────┘  └──────────────────┘  └───────────────┘ │
│           │ TRIGGER: notify_document_change()                   │
│           │ CHANNEL: 'document_changes'                         │
└───────────┼─────────────────────────────────────────────────────┘
            │
            │ LISTEN (PostgreSQL NOTIFY)
            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RAG Embedding Worker (Rust)                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │  NotificationListener                                     │   │
│  │  ├─ LISTEN on 'document_changes'                         │   │
│  │  └─ Enqueue tasks to TaskQueue                           │   │
│  └────────────────────┬─────────────────────────────────────┘   │
│                       │                                          │
│  ┌────────────────────▼─────────────────────────────────────┐   │
│  │  DocumentProcessor                                        │   │
│  │  ├─ Load file from path                                  │   │
│  │  ├─ Parse document (PDF/DOCX/MD/HTML/Code)               │   │
│  │  ├─ Chunk text (Semantic/Fixed/Recursive)                │   │
│  │  └─ Generate embeddings                                  │   │
│  └────────────────────┬─────────────────────────────────────┘   │
│                       │                                          │
│  ┌────────────────────▼─────────────────────────────────────┐   │
│  │  LlamaServerManager (On-Demand)                          │   │
│  │  ├─ Check available memory (> 2GB required)              │   │
│  │  ├─ Spawn llama-server process                           │   │
│  │  ├─ Generate embeddings via HTTP                         │   │
│  │  └─ Stop process to free memory                          │   │
│  └────────────────────┬─────────────────────────────────────┘   │
│                       │                                          │
│  ┌────────────────────▼─────────────────────────────────────┐   │
│  │  Repository (Database Layer)                             │   │
│  │  ├─ Save chunks to rag_document_chunks                   │   │
│  │  ├─ Update ingestion log status                          │   │
│  │  └─ Handle transaction & error recovery                  │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

```

## 🔄 Processing Flow

```mermaid
graph TD
    A[INSERT/UPDATE TblDocumentFiles] --> B[PostgreSQL Trigger]
    B --> C[NOTIFY 'document_changes']
    C --> D[NotificationListener Receives]
    D --> E[Enqueue Task to TaskQueue]
    E --> F{Check Memory Available?}
    F -->|< 2GB| G[Skip - Log Warning]
    F -->|>= 2GB| H[Start llama-server]
    H --> I[Load Document from Path]
    I --> J[Parse Document Content]
    J --> K[Chunk Text Semantic/Fixed/Recursive]
    K --> L[Generate Embeddings Batch]
    L --> M[Stop llama-server]
    M --> N[Save Chunks to DB]
    N --> O[Update Ingestion Log: Completed]
    
    G --> P[Update Ingestion Log: Failed]
    
    style H fill:#90EE90
    style M fill:#FFB6C1
    style N fill:#87CEEB
```

## 📋 Prerequisites

### Development (Linux Mint / Ubuntu)

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# Cross-compilation untuk Windows
rustup target add x86_64-pc-windows-gnu
sudo apt-get install -y gcc-mingw-w64-x86-64

# Build dependencies
sudo apt-get install -y build-essential pkg-config libssl-dev

# PostgreSQL client (optional, untuk testing)
sudo apt-get install -y postgresql-client
```

### Production (Windows Server 2022)

1. **PostgreSQL** dengan extension `pgvector`
2. **llama-server binary** (dari llama.cpp)
3. **Embedding model** (contoh: `all-MiniLM-L6-v2-Q4_K_M.gguf`)
4. **NSSM** (Non-Sucking Service Manager) untuk run sebagai Windows service

## 🚀 Quick Start

### 1. Clone & Configure

```bash
git clone <repository-url>
cd rag-embedding-worker

# Copy environment template
cp .env.example .env

# Edit configuration
nano .env
```

### 2. Database Setup

```bash
# Connect to PostgreSQL
psql -U postgres -d your_database

# Run SQL script
\i database_setup.sql
```

### 3. Build (Linux Development)

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Cross-compile untuk Windows
cargo build --release --target x86_64-pc-windows-gnu
```

### 4. Run Locally

```bash
# Set environment variables
export DATABASE_URL="postgres://user:password@localhost:5432/dbname"
export DOCUMENT_ROOT_PATH="/path/to/documents"
export LLAMA_SERVER_BINARY="/path/to/llama-server"
export LLAMA_SERVER_MODEL="/path/to/model.gguf"

# Run worker
cargo run --release
```

## 📦 Configuration

### Environment Variables

```env
# Database
DATABASE_URL=postgres://user:password@localhost:5432/dbname

# Llama Server
LLAMA_SERVER_BINARY=/path/to/llama-server
LLAMA_SERVER_MODEL=/path/to/models/all-MiniLM-L6-v2-Q4_K_M.gguf

# Worker
DOCUMENT_ROOT_PATH=/path/to/documents

# Logging
RUST_LOG=info,rag_embedding_worker=debug
LOG_FORMAT=pretty  # or "json" for production
```

### Configuration File (`config/settings.toml`)

```toml
[database]
pool_max_size = 10
listen_channel = "document_changes"

[embedding]
model = "AllMiniLML6V2"
dimension = 384
batch_size = 32

[chunking]
size = 512
overlap = 50
strategy = "semantic"  # semantic, fixed, recursive

[worker]
bulk_batch_size = 10
processing_timeout_seconds = 300

[llama_server]
host = "127.0.0.1"
port = 8080
startup_timeout_seconds = 60
embedding_only = true
ctx_size = 2048
threads = 4
```

## 🪟 Windows Server 2022 Deployment

### Step 1: Prepare Binary

**Dari Linux (Cross-Compile):**

```bash
# Install MinGW
sudo apt-get install -y gcc-mingw-w64-x86-64

# Add Windows target
rustup target add x86_64-pc-windows-gnu

# Configure Cargo
mkdir -p ~/.cargo
cat >> ~/.cargo/config << EOF
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-gcc-ar"
EOF

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Binary ada di: target/x86_64-pc-windows-gnu/release/rag-embedding-worker.exe
```

### Step 2: Transfer ke Windows Server

```bash
# Upload via SCP/FTP ke Windows Server
# Struktur folder di Windows:
C:\Program Files\RAG-Embedding-Worker\
├── rag-embedding-worker.exe
├── config\
│   └── settings.toml
├── logs\
└── models\
    └── all-MiniLM-L6-v2-Q4_K_M.gguf
```

### Step 3: Install Dependencies di Windows Server

**Install llama-server:**

```powershell
# Download llama.cpp pre-built binary untuk Windows
# https://github.com/ggml-org/llama.cpp/releases

# Extract ke folder
C:\Program Files\llama.cpp\
└── llama-server.exe

# Download model embedding (contoh: all-MiniLM-L6-v2)
# Simpan di: C:\Program Files\RAG-Embedding-Worker\models\
```

**Install PostgreSQL Client Library (jika perlu):**

```powershell
# Install Visual C++ Redistributable
# Download dari: https://aka.ms/vs/17/release/vc_redist.x64.exe
```

### Step 4: Setup Environment

**Buat file `.env` di `C:\Program Files\RAG-Embedding-Worker\.env`:**

```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DOCUMENT_ROOT_PATH=D:\Documents
LLAMA_SERVER_BINARY=C:\Program Files\llama.cpp\llama-server.exe
LLAMA_SERVER_MODEL=C:\Program Files\RAG-Embedding-Worker\models\all-MiniLM-L6-v2-Q4_K_M.gguf
RUST_LOG=info,rag_embedding_worker=debug
LOG_FORMAT=json
```

### Step 5: Install sebagai Windows Service menggunakan NSSM

**Download NSSM:**

```powershell
# Download dari: https://nssm.cc/download
# Extract ke: C:\Program Files\NSSM\
```

**Install Service:**

```powershell
# Buka PowerShell sebagai Administrator

# Set path
$env:Path += ";C:\Program Files\NSSM"

# Install service
nssm install "RAG-Embedding-Worker" "C:\Program Files\RAG-Embedding-Worker\rag-embedding-worker.exe"

# Set working directory
nssm set "RAG-Embedding-Worker" AppDirectory "C:\Program Files\RAG-Embedding-Worker"

# Set environment variables (optional, bisa pakai .env file)
nssm set "RAG-Embedding-Worker" AppEnvironmentExtra "RUST_LOG=info"

# Set log files
nssm set "RAG-Embedding-Worker" AppStdout "C:\Program Files\RAG-Embedding-Worker\logs\stdout.log"
nssm set "RAG-Embedding-Worker" AppStderr "C:\Program Files\RAG-Embedding-Worker\logs\stderr.log"

# Auto restart on failure
nssm set "RAG-Embedding-Worker" AppExit Default Restart
nssm set "RAG-Embedding-Worker" AppRestartDelay 5000

# Start service
nssm start "RAG-Embedding-Worker"

# Check status
nssm status "RAG-Embedding-Worker"
```

**Service Management:**

```powershell
# Stop service
nssm stop "RAG-Embedding-Worker"

# Restart service
nssm restart "RAG-Embedding-Worker"

# Remove service
nssm remove "RAG-Embedding-Worker" confirm

# View logs
Get-Content "C:\Program Files\RAG-Embedding-Worker\logs\stdout.log" -Wait
```

### Step 6: Firewall Configuration (jika diperlukan)

```powershell
# Allow llama-server port (jika butuh akses dari luar)
New-NetFirewallRule -DisplayName "Llama Server" -Direction Inbound -LocalPort 8080 -Protocol TCP -Action Allow
```

## 📊 Monitoring & Troubleshooting

### Check Logs

```powershell
# Windows
Get-Content "C:\Program Files\RAG-Embedding-Worker\logs\app.log" -Tail 50 -Wait

# Linux
tail -f logs/app.log
```

### Common Issues

**1. "Not enough memory to start embedding server"**

- **Solusi**: Pastikan minimal 2GB RAM available. Tutup aplikasi lain atau upgrade RAM.

**2. "Llama-server failed to start"**

- **Cek**: Path ke binary dan model benar
- **Cek**: Model file tidak corrupt (re-download jika perlu)
- **Cek**: Port 8080 tidak digunakan aplikasi lain

**3. "File not found: /path/to/document"**

- **Cek**: `DOCUMENT_ROOT_PATH` benar
- **Cek**: Path di `TblDocumentFiles` relative atau absolute sesuai konfigurasi

**4. "Connection refused" (PostgreSQL)**

- **Cek**: PostgreSQL service running
- **Cek**: Connection string benar
- **Cek**: User memiliki permission ke database

## 🔧 Development

### Project Structure

```
rag-embedding-worker/
├── Cargo.toml
├── .env.example
├── config/
│   └── settings.toml
├── database_setup.sql
├── README.md
└── src/
    ├── main.rs
    ├── lib.rs
    ├── config/
    │   ├── mod.rs
    │   └── settings.rs
    ├── database/
    │   ├── mod.rs
    │   ├── pool.rs
    │   ├── models.rs
    │   ├── repository.rs
    │   └── listener.rs
    ├── embedding/
    │   ├── mod.rs
    │   ├── provider.rs
    │   └── llama_server.rs
    ├── document/
    │   ├── mod.rs
    │   ├── loader.rs
    │   ├── parser.rs
    │   └── chunker.rs
    ├── worker/
    │   ├── mod.rs
    │   ├── processor.rs
    │   ├── bulk_indexer.rs
    │   └── queue.rs
    └── utils/
        ├── mod.rs
        ├── error.rs
        ├── logger.rs
        └── metrics.rs
```

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Linting

```bash
cargo clippy -- -D warnings
```

## 📈 Performance Tuning

### Memory Usage

- **Idle**: ~10-50 MB (Rust binary only)
- **Processing**: ~2-3 GB (when llama-server active)
- **Peak**: ~4 GB (large documents + batch processing)

### Optimization Tips

1. **Adjust batch size**: Kurangi `bulk_batch_size` jika RAM terbatas
2. **Chunking strategy**: `semantic` lebih akurat tapi lebih lambat, `fixed` lebih cepat
3. **Model size**: Gunakan quantized models (Q4_K_M) untuk balance speed/quality
4. **Vector index**: HNSW untuk < 1M rows, IVFFlat untuk > 1M rows

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## 🙏 Credits

- **[llama.cpp](https://github.com/ggml-org/llama.cpp)** - LLM inference engine[web:30]
- **[pgvector](https://github.com/pgvector/pgvector)** - Vector similarity search for PostgreSQL[web:40]
- **[tokio-postgres](https://docs.rs/tokio-postgres)** - Async PostgreSQL client
- **[text-splitter](https://crates.io/crates/text-splitter)** - Smart text chunking
- **[lopdf](https://github.com/J-F-Liu/lopdf)** - PDF parsing library[web:11]

## 📞 Support

For issues and questions, please open an issue on GitHub or contact <support@example.com>.

---

**Built with ❤️ using Rust 🦀**

```

***

Bro, ini README.md yang sangat lengkap! Sekarang saya lanjutkan dengan **skenario baru (Full RAG Pipeline API)** nya. Mau saya lanjutkan sekarang dengan architecture + code untuk aplikasi RAG API dengan security layer? 🚀
