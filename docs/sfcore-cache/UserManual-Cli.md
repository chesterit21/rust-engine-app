# User Manual - localcached-cli

**localcached-cli** adalah tool "Satu Pintu" untuk memanajemen server `localcached`, mulai dari menyalakan server, mematikan, hingga memonitor statistik secara real-time via TUI (Terminal User Interface).

## 🚀 Instalasi

Pastikan Anda berada di workspace `root-app/sfcore-ai`:

```bash
# Build CLI
cargo build --release -p localcached-cli

# (Opsional) Tambahkan ke PATH agar bisa dipanggil dari mana saja
export PATH=$PATH:$(pwd)/target/release
```

## 🛠️ Perintah Dasar

Aplikasi ini memiliki 3 mode utama: `monitor` (default), `start`, dan `stop`.

### 1. Menyalakan Server (Start)

Menjalankan server `localcached-server` sebagai background process (daemon).

```bash
# Start standar (mencari binary otomatis)
localcached-cli start

# Start dengan binary specific
localcached-cli start --bin ./target/release/localcached-server
```

> **Note:** Server akan menggunakan konfigurasi environment variable standar jika ada (misal `LOCALCACHED_SOCKET`).

### 2. Mematikan Server (Stop)

Mematikan server yang sedang berjalan (menggunakan PID file).

```bash
localcached-cli stop
```

### 3. Monitoring (TUI)

Masuk ke mode visual interaktif untuk melihat statistik, metrics, dan logs.

```bash
# Cara 1: Default
localcached-cli

# Cara 2: Eksplisit
localcached-cli monitor

# Cara 3: Custom Socket
localcached-cli --socket /tmp/mysocket.sock monitor
```

## 🖥️ Fitur TUI (Monitoring)

Saat Anda masuk ke mode TUI, Anda akan melihat dashboard dengan informasi:

- **Metrics**: Uptime, Total Keys, Memory Usage.
- **Charts**: Grafik Hit Rate (Realtime), Memory Pressure.
- **Log Activity**: Stream log aktivitas terbaru dari server.
- **Config Info**: Menampilkan path socket dan limit konfigurasi.

**Navigasi Keyboard:**

- `q` atau `Ctrl+c`: Keluar dari TUI.

---

## ⚙️ Environment Variables

CLI membaca variable berikut:

| Variable | Fungsi | Default |
| :--- | :--- | :--- |
| `LOCALCACHED_SOCKET` | Lokasi socket file target | `/run/localcached.sock` |
| `LOCALCACHED_BIN` | Lokasi binary server (untuk command `start`) | Auto-detect |
| `LOCALCACHED_PID_FILE` | Lokasi PID file (untuk `stop`) | Auto-derive dari socket path |

---

## 💡 Contoh Workflow Lengkap

```bash
# 1. Start Server
localcached-cli start

# 2. Cek Status (Masuk TUI)
localcached-cli
# (Tekan 'q' untuk keluar)

# 3. Stop Server jika sudah selesai
localcached-cli stop
```
