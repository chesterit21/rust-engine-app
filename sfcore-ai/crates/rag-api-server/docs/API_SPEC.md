# RAG API Server - API Specification & Client Integration Guide

Dokumen ini menjelaskan alur kerja dan spesifikasi teknis untuk mengintegrasikan Client App dengan RAG API Server yang menggunakan arsitektur **Reactive & Decoupled**.

---

## 1. Security Headers (Wajib)

Semua permintaan (kecuali `/health`) harus menyertakan header berikut:

| Header | Deskripsi | Contoh |
| --- | --- | --- |
| `X-App-ID` | ID Aplikasi Client | `DMS-CLIENT-APP-2026` |
| `X-API-Key` | API Key Rahasia | `your-secret-api-key-here` |
| `X-Request-Timestamp` | Unix Timestamp (Detik) | `1706170000` |
| `X-Request-Signature` | HMAC-SHA256 (Key, ID+Timestamp) | `a1b2c3d4...` |

---

## 2. Alur Kerja Utama (Client App Guide)

Agar pengalaman pengguna (UX) maksimal, Client App harus mengikuti urutan interaksi berikut:

### Langkah 1: Inisialisasi Sesi (`Consolidated Init`)

Saat aplikasi pertama kali dibuka (atau halaman chat dimuat), segera panggil:

- **Endpoint**: `POST /api/chat/init`
- **Output**: Dapatkan atau set `session_id` dan daftar dokumen yang tersedia.

### Langkah 2: Berlangganan Event (`Persistent Stream`)

Segera setelah mendapatkan `session_id`, jalankan fungsi **standalone listener** untuk mendengarkan semua event sistem sistem (SSE):

- **Endpoint**: `GET /api/chat/events?session_id=...`
- **Tujuan**: Menangkap update proses latar belakang secara asinkron.

### Langkah 3: Penanganan Attachment & Unggah File

Jika pengguna melakukan **Upload** atau **Drag-and-Drop** file:

1. **Kirim Segera**: Langsung kirim file ke `POST /api/upload`. Sertakan `session_id`.
2. **UX Feedback (Fake Response)**: Begitu file mulai di-upload, Client App **WAJIB** menampilkan pesan template manual (seolah-olah dari AI) di bubble chat:
    > "Mohon tunggu sebentar ya, dokumen kamu sedang diproses oleh system. Nanti kamu bisa menanyakan perihal isi dokumen tersebut dengan saya setelah proses upload selesai. Kamu bisa melihat progresnya pada progress bar berikut."
3. **Progres Real-time via SSE**: Client App memantau stream dari `/api/chat/events`. Gunakan data dari `system_event` untuk memperbarui progress bar secara visual.

---

## 3. Spesifikasi Event Progres (SSE)

Server akan mengirimkan event dengan format berikut pada `/api/chat/events`:

`event: system_event`
`data: {"type": "processing_progress", "payload": {"progress": 0.6, "message": "...", "status_flag": "embedding-inprogress"}}`

### Daftar `status_flag` untuk Progress Bar

| Flag | Progress | Deskripsi untuk UI |
| --- | --- | --- |
| `detecting` | 10% | Mendeteksi format file... |
| `parsing` | 20% | Mengekstrak teks dari dokumen... |
| `chunking` | 40% | Membagi kalimat menjadi potongan teks... |
| `embedding-inprogress` | 60% | **Proses AI: Mengubah teks menjadi vektor (Embedding)...** |
| `saving` | 80% | Menyimpan metadata ke database... |
| `indexing` | 90% | Membangun index pencarian... |
| `completed` | 100% | Pemrosesan selesai! Dokumen siap ditanyakan. |

---

## 4. Endpoints Reference

### A. Chat Initialization

- **Response (JSON)**:

```json
{
  "session_id": 456,
  "documents": [...],     // Dokumen yang sudah selesai di-index
  "processing_docs": [   // Dokumen yang SEDANG di-proses (Resilience)
    {
      "document_id": 789,
      "status": "embedding-inprogress",
      "progress": 0.6,
      "message": "AI is thinking...",
      "updated_at": "2024-01-15T10:00:00Z"
    }
  ]
}
```

**State Recovery**: Jika Client App melakukan refresh halaman atau koneksi SSE terputus, data `processing_docs` ini digunakan untuk membangun kembali Progress Bar secara otomatis di sisi UI.

### B. Chat Stream (LLM)

- **URL**: `POST /api/chat/stream`
- **Tujuan**: Stream jawaban dari AI.
- **SSE Events**: `message`, `done`, `error`.

### C. Upload Document (Async)

- **URL**: `POST /api/upload`
- **Fields**: `user_id`, `session_id`, `file`
- **Response**: `{"success": true, "message": "Accepted"}`

---

## 5. Contoh Implementasi SSE (Pseudo-code)

```javascript
const eventSource = new EventSource(`/api/chat/events?session_id=${sessionId}`);

eventSource.addEventListener('system_event', (e) => {
  const event = JSON.parse(e.data);
  if (event.type === 'processing_progress') {
    const { progress, status_flag, message } = event.payload;
    
    // Update Progress Bar UI
    myProgressBar.setValue(progress * 100);
    myStatusLabel.setText(message);
    
    // Highlight if embedding
    if (status_flag === 'embedding-inprogress') {
      myProgressBar.setColor('blue');
    }
  }
});
```
