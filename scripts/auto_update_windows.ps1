# Konfigurasi
$RepoOwner = "USERNAME_GITHUB_KAMU"  # Ganti dengan username GitHub
$RepoName = "NAMA_REPO_KAMU"         # Ganti dengan nama repository
$InstallDir = "C:\Program Files\RAG-System\api-server"
$ExeName = "rag-api-server.exe"
$ServiceName = "RagApiServer"        # Nama service (jika pakai NSSM/Windows Service)
$DownloadUrl = "https://github.com/$RepoOwner/$RepoName/releases/download/latest/$ExeName"

# Setup Folder
if (!(Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir
}

Write-Host "Checking for updates..." -ForegroundColor Cyan

# 1. Matikan Service / Process Lama
Write-Host "Stopping service..."
if (Get-Service $ServiceName -ErrorAction SilentlyContinue) {
    Stop-Service -Name $ServiceName -Force
} else {
    # Fallback kalau dijalankan manual (bukan service)
    Stop-Process -Name "rag-api-server" -ErrorAction SilentlyContinue
}

# Tunggu sebentar biar file release lock
Start-Sleep -Seconds 2

# 2. Download File Baru
Write-Host "Downloading new version from GitHub..."
try {
    # Backup dulu
    if (Test-Path "$InstallDir\$ExeName") {
        Move-Item "$InstallDir\$ExeName" "$InstallDir\$ExeName.bak" -Force
    }

    # Download (Butuh Token kalau Repo Private)
    # Kalau Private Repo, tambahkan header: -Headers @{Authorization = "token YOUR_PAT_TOKEN"}
    Invoke-RestMethod -Uri $DownloadUrl -OutFile "$InstallDir\$ExeName"
    
    Write-Host "Update success!" -ForegroundColor Green
}
catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
    # Restore backup
    if (Test-Path "$InstallDir\$ExeName.bak") {
        Move-Item "$InstallDir\$ExeName.bak" "$InstallDir\$ExeName" -Force
    }
    exit 1
}

# 3. Nyalakan Service Lagi
Write-Host "Starting service..."
if (Get-Service $ServiceName -ErrorAction SilentlyContinue) {
    Start-Service -Name $ServiceName
} else {
    # Start manual (background)
    Start-Process -FilePath "$InstallDir\$ExeName" -WorkingDirectory $InstallDir -WindowStyle Hidden
}

Write-Host "Server is up and running!" -ForegroundColor Green
