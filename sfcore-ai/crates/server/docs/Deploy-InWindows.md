## Cara Kerjanya di Windows

Kode pake `windows-service` crate yang nge-wrap Windows SCM (Service Control Manager) secara native. Ini yang paling bener buat `.exe` di Windows Server — sama seperti SQL Server, IIS, etc. jalanin service-nya.

**Flow-nya:**

```
sc start SFCoreAIService
        │
        ▼
Windows SCM panggil exe dengan --service flag
        │
        ▼
windows_service_glue::run_as_service()
        │
        ├── Register control handler (tangkap STOP signal dari SCM)
        ├── Report status → Running
        ├── Spawn server di thread terpisah
        └── Tunggu STOP signal → Report status → Stopped
```

---

## Step-by-step Deploy

**1. Build untuk Windows:**

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

**2. Copy ke server — folder structure:**

```
C:\SFCore\
├── sfcore-ai-server.exe       ← dari target\release\
├── server_config.toml         ← config lo (pastiin path model bener)
├── models\
│   └── QwenCoder-1.5b.gguf   ← model file
└── install_service.ps1        ← script helper
```

**3. Install & start pake PowerShell (run as Administrator):**

```powershell
cd C:\SFCore

# Install service
powershell -ExecutionPolicy Bypass -File install_service.ps1 -Action install

# Start
powershell -ExecutionPolicy Bypass -File install_service.ps1 -Action start

# Cek status
powershell -ExecutionPolicy Bypass -File install_service.ps1 -Action status
```

**4. Manage via `sc` command (built-in Windows):**

```cmd
sc start  SFCoreAIService
sc stop   SFCoreAIService
sc query  SFCoreAIService
sc delete SFCoreAIService
```

---

## Perhatian di `server_config.toml`

Di Windows, pastiin path model pakai backslash atau forward slash yang valid, dan transport **jangan** pakai `uds` (itu Linux-only):

```toml
model = "C:\\SFCore\\models\\QwenCoder-1.5b.gguf"
transport = "tcp"   # ← harus tcp atau http-sse di Windows
```
