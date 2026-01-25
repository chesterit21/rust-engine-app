# Windows Server Deployment Guide (Specific Paths) - CPU OPTIMIZED

Panduan ini disesuaikan dengan struktur folder berikut:

- **Llama Server**: `C:\llama-server\llama-server.exe`
- **Model Chat**: `C:\models\qwen2.5.gguf`
- **Model Embedding**: `C:\models\Qwen3-Embedding-0.6B-Q8_0.gguf`
- **Aplikasi API**: `C:\Program Files\RAG-System\api-server\rag-api-server.exe`
- **Config**: `C:\Program Files\RAG-System\api-server\config\settings.toml`

---

## BAGIAN 1: Versi Testing (Manual Run)

Gunakan **3 Jendela PowerShell** terpisah (Run as Administrator disarankan untuk `--mlock`).

### Terminal 1: Menjalankan Chat Server (Port 8080)

Optimasi CPU:

- **--mlock**: Kunci RAM agar tidak masuk pagefile/swap (Wajib run as Admin).
- **--batch-size 2048**: Mempercepat pemrosesan prompt panjang.
- **--ubatch-size 512**: Ukuran physical batch untuk efisiensi CPU cache.
- **--cache-type-k f16**: Presisi Key cache (standard).

```powershell
PS C:\> cd C:\llama-server
PS C:\llama-server> .\llama-server.exe `
    -m "C:\models\qwen2.5.gguf" `
    --port 8080 `
    --host 0.0.0.0 `
    --ctx-size 32768 `
    --threads 8 `
    --mlock `
    --batch-size 2048 `
    --ubatch-size 512 `
    --cache-type-k f16
```

### Terminal 2: Menjalankan Embedding Server (Port 8081)

Optimasi Embedding:

- **--ubatch-size 4096**: Batch besar untuk embedding lebih efisien karena tidak ada generate token.

```powershell
PS C:\> cd C:\llama-server
PS C:\llama-server> .\llama-server.exe `
    -m "C:\models\Qwen3-Embedding-0.6B-Q8_0.gguf" `
    --port 8081 `
    --embedding `
    --host 0.0.0.0 `
    --ctx-size 8192 `
    --threads 4 `
    --mlock `
    --batch-size 4096 `
    --ubatch-size 4096
```

### Terminal 3: Menjalankan RAG API Server (Port 8000)

Aplikasi perlu dijalankan dari folder kerjanya agar bisa membaca folder `config/`.

```powershell
PS C:\> cd "C:\Program Files\RAG-System\api-server"
PS C:\Program Files\RAG-System\api-server> .\rag-api-server.exe
```

---

## BAGIAN 2: Versi Deployment Real (Windows Service dengan NSSM)

Pastikan `nssm.exe` sudah ada di sistem dan masuk PATH.
Buka PowerShell sebagai **Administrator**.

### 1. Install Service: Chat LLM

```powershell
PS C:\> nssm install SF-Chat-LLM "C:\llama-server\llama-server.exe"
PS C:\> nssm set SF-Chat-LLM AppDirectory "C:\llama-server"
PS C:\> nssm set SF-Chat-LLM AppParameters '-m "C:\models\qwen2.5.gguf" --port 8080 --host 0.0.0.0 --ctx-size 32768 --threads 8 --mlock --batch-size 2048 --ubatch-size 512 --cache-type-k f16'
PS C:\> nssm set SF-Chat-LLM Description "Llama Server for Chat Model (Port 8080) - CPU Optimized"
PS C:\> nssm start SF-Chat-LLM
```

### 2. Install Service: Embedding Model

```powershell
PS C:\> nssm install SF-Embedding-LLM "C:\llama-server\llama-server.exe"
PS C:\> nssm set SF-Embedding-LLM AppDirectory "C:\llama-server"
PS C:\> nssm set SF-Embedding-LLM AppParameters '-m "C:\models\Qwen3-Embedding-0.6B-Q8_0.gguf" --port 8081 --embedding --host 0.0.0.0 --ctx-size 8192 --threads 4 --mlock --batch-size 4096 --ubatch-size 4096'
PS C:\> nssm set SF-Embedding-LLM Description "Llama Server for Embedding Model (Port 8081) - CPU Optimized"
PS C:\> nssm start SF-Embedding-LLM
```

### 3. Install Service: RAG API Server

```powershell
PS C:\> nssm install SF-RAG-API "C:\Program Files\RAG-System\api-server\rag-api-server.exe"
PS C:\> nssm set SF-RAG-API AppDirectory "C:\Program Files\RAG-System\api-server"
PS C:\> nssm set SF-RAG-API Description "RAG API Middleware (Port 8000)"
PS C:\> nssm set SF-RAG-API AppStdout "C:\Program Files\RAG-System\api-server\logs\service.log"
PS C:\> nssm set SF-RAG-API AppStderr "C:\Program Files\RAG-System\api-server\logs\error.log"
PS C:\> nssm start SF-RAG-API
```

### Verifikasi Deployment

Cek status service:

```powershell
PS C:\> Get-Service SF-*
```
