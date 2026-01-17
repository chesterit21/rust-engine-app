# 🚀 SFCore AI & Tools Repository

Repository ini berisi sekumpulan tools performa tinggi yang dibangun dengan **Rust**, dirancang khusus untuk efisiensi maksimal pada hardware terbatas (Low-End Hardware).

Terdapat dua aplikasi utama dalam project ini:

---

## 1. 🤖 SFCore-AI (Inference Engine)

**SFCore-AI** adalah mesin inferensi untuk model AI (LLM) berbasis format **GGUF**. Aplikasi ini dibangun di atas binding `llama.cpp` dengan optimasi khusus di level Rust.

### 🎯 Tujuan & Filosofi

Dibuat untuk membuktikan bahwa **AI tidak harus mahal**. Kami menargetkan performa inferensi yang *usable* (bisa dipakai chat real-time) bahkan pada komputer "kentang" yang sering dianggap usang.

### 🖥️ Spesifikasi Testing (The "Potato" Rig)

Semua benchmark dan tuning dilakukan pada mesin dengan spesifikasi berikut:

- **CPU**: Intel Core i3-6100 (Gen 6 Skylake, 2 Core / 4 Thread).
- **RAM**: 16 GB DDR4.
- **OS**: Linux.

### 🔥 Keunggulan Utama

1. **Extreme Optimization**: Menggunakan flag CPU instructions (`-march=native`) untuk memeras setiap `FLOPS` dari CPU tua.
2. **Native Threading**: Manajemen thread yang presisi untuk menghindari *Core Oversubscription* (Context Switching berlebih).
3. **Fast Startup**: Waktu loading model yang sangat cepat (~400ms untuk FTL) berkat memory mapping yang efisien.
4. **IPC Server**: Berjalan sebagai daemon dengan Unix Domain Socket, siap diintegrasikan dengan aplikasi lain (NodeJS/Go/Python).

### 📚 Dokumentasi

- [**Walkthrough & Benchmark**](docs/sfcore-ai/readme.md) - Detail perjalanan tuning performa mencapai >8 tok/s di i3 Gen 6.

---

## 2. ⚡ SFCore-Cache (`localcached`)

**SFCore-Cache** (binary: `localcached-server`) adalah sistem in-memory key-value store lokal, mirip seperti **Redis**, namun didesain khusus untuk komunikasi antar proses (IPC) dalam satu mesin.

### 🎯 Kenapa SFCore-Cache?

Redis sangat powerful, tapi menggunakan TCP/IP (bahkan via localhost) memiliki overhead latency dan syscall. Untuk arsitektur Microservices atau Modular Monolith yang berjalan dalam satu server fisik/VPS, **Unix Domain Socket (UDS)** jauh lebih cepat (sekitar 30-50% lebih kencang dibanding TCP Loopback).

### 🌟 Keunggulan SFCore-Cache

1. **Ultra Low Latency**: Menggunakan UDS dan protokol binary minimalis.
2. **High Stability**: Menggunakan allocator **mimalloc** (Microsoft) untuk mencegah fragmentasi memori di load tinggi.
3. **Backpressure Control**: Built-in mechanism (Semaphore) untuk menahan request saat beban puncak, mencegah server crash (OOM).
4. **Smart Eviction**: Algoritma **Sampled LRU** yang cerdas membuang data dingin tanpa overhead sorting yang berat.

### ⚠️ Kompatibilitas Sistem

Saat ini **HANYA support Linux (dan MacOS/BSD)** karena ketergantungan pada standar Unix Domain Socket (`AF_UNIX`). **Tidak support Windows**.

### 📚 Dokumentasi Lengkap

Panduan lengkap mulai dari penggunaan dasar, CLI, hingga integrasi framework:

- [**User Manual (General)**](docs/sfcore-cache/UserManual.md) - Cara kerja, protokol, dan konsep dasar.
- [**User Manual (CLI)**](docs/sfcore-cache/UserManual-Cli.md) - Panduan manajemen server (Start/Stop/Monitor) via Terminal.
- [**Framework Integration (DI)**](docs/sfcore-cache/UserManual-DI.md) - **Recommended!** Panduan best-practice Dependency Injection untuk .NET, Laravel, NestJS, Spring Boot, dll.

---

## 📜 License

Project ini didistribusikan di bawah lisensi [MIT](./LICENSE).

## 👏 Credits & Citations

Kami mengucapkan terima kasih kepada library Open Source luar biasa yang menjadi fondasi alat ini:

- **[Ratatui](https://ratatui.rs/)**: Untuk engine TUI (Terminal User Interface) yang modern, performant, dan kaya fitur yang digunakan pada `localcached-cli`.
- **[llama.cpp](https://github.com/ggerganov/llama.cpp)**: Core engine untuk inferensi GGUF yang efisien.
- **[mimalloc](https://github.com/microsoft/mimalloc)**: Allocator super cepat dari Microsoft.

---

Copyright © 2026 SFCore AI Team. Built with ❤️ and 🦀 Rust.
