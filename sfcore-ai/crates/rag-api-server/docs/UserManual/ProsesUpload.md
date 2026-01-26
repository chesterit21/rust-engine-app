# Alur Proses Upload Dokumen (RAG System)

Dokumen ini menjelaskan secara teknis dan logis bagaimana sistem backend menangani permintaan upload file dari Client App.

## 1. Menerima Request

* **Endpoint**: `POST /api/documents`
* **Format**: `Multipart/form-data`
* **Parameter**:
  * `user_id`: ID pengguna (integer).
  * `session_id`: ID sesi chat (untuk notifikasi progress real-time).
  * `file`: Binary file yang diupload.

## 2. Fase Sinkron (Synchronous) - "Respon Cepat"

Fase ini terjadi seketika saat request diterima. Tujuannya agar Client tidak menunggu lama dan mendapatkan ID Dokumen secepatnya.

1. **Validasi Input**:
    * Sistem memastikan parameter `user_id`, `session_id`, dan `file` tersedia.

2. **Deteksi & Validasi Ketat (Security Hardening)**:
    * **Size Limit**: Maksimal 50MB. Lebih dari itu ditolak.
    * **Magic Number Check**: Sistem membaca *byte header* file menggunakan library `infer` untuk memastikan tipe file asli (bukan cuma dari ekstensi).
    * **MIME Whitelist**: Hanya menerima tipe aman: `application/pdf`, `application/vnd.openxml...` (Office), `text/plain`, `image/png`, dll.
    * **Anti-Executable**: File `.exe`, `.bat`, `.sh`, `.elf` akan ditolak mentah-mentah meskipun di-rename jadi `.pdf`.

3. **Cek Kategori (Otomatis)**:
    * Sistem mengecek ke tabel database `TblCategories`.
    * **Logic**: Apakah User ini sudah punya kategori dengan nama `"Document-Upload-AI"`?
    * **Jika Belum**: Sistem membuat kategori baru (`INSERT`) dengan `ParentId = NULL` dan `Owner = user_id`.
    * **Jika Sudah**: Sistem mengambil `Id` dari kategori yang sudah ada tersebut.

4. **Penyimpanan Fisik (Persistent Storage)**:
    * Sistem membaca konfigurasi `document_path` dari `settings.toml` (contoh: `C:\DMS\uploads\`).
    * **Folder User**: Sistem memformat path folder: `[document_path]\[UserID]-Document-AI\`.
    * **Auto-Create**: Jika folder tersebut belum ada, sistem membuatnya otomatis.
    * **Secure Naming (UUID)**: File disimpan dengan format UUID `[UUIDv4].[Ext]` (contoh: `550e8400-e29b....pdf`). Nama asli file HANYA disimpan di database, tidak di file system. Ini mencegah directory traversal, file collision, dan menebak nama file lain.
    * **Write File**: File disimpan ke folder tersebut. Path lengkapnya (absolute path) dicatat untuk disimpan ke database.

5. **Database Record (Pre-processing)**:
    * **Insert tabel `TblDocuments`**:
        * `CategoryId`: ID Kategori dari langkah 3.
        * `DocumentTitle`: Nama file asli (misal: `Laporan.pdf`).
        * `DocumentDesc`: "Uploaded via API: [filename]".
    * **Insert tabel `TblDocumentFiles`**:
        * `FilePath`: Path fisik lengkap (UUID) dari langkah 4.
        * `FileName`: Nama file asli (untuk display/download user).
        * `IsMainDocumentFile`: `true`.

6. **Return Response**:
    * Sistem mengembalikan response JSON ke Client:

        ```json
        {
            "documentId": 123,
            "documentName": "Laporan.pdf"
        }
        ```

    * Koneksi HTTP dengan Client selesai di sini. Client bisa langsung menggunakan ID tersebut.

## 3. Fase Asinkron (Background) - "AI Processing"

Fase ini berjalan di latar belakang (background thread) setelah response dikirim. Client memantau fase ini melalui **Progress Bar** (via SSE/EventBus).

1. **Notifikasi Start**:
    * Server mengirim event SSE `ProcessingStarted` ke Client.

2. **Parsing Dokumen**:
    * Sistem membaca konten teks dari file yang tersimpan.
    * Menggunakan parser sesuai tipe file (misal: `pdf` extract text).

3. **Chunking (Pemecahan Teks)**:
    * Teks dipecah menjadi bagian-bagian kecil (chunks) sesuai konfigurasi (misal: 512 token).
    * Tujuannya agar teks muat dalam konteks model AI.

4. **Embedding (Proses AI)**:
    * Setiap chunk dikirim ke Model Embedding.
    * Model mengubah teks menjadi vektor angka (vector representation).
    * *Update Progress*: Sepanjang proses ini, event `ProcessingProgress` dikirim terus-menerus (10%... 40%... 60%).

5. **Indexing Database**:
    * Hasil vektor disimpan ke tabel `rag_document_chunks` (menggunakan pgvector).
    * Ini memungkinkan dokumen bisa dicari secara semantik (Semantic Search) nantinya.

6. **Auto-Summary**:
    * Sistem mengirim potongan awal dokumen ke LLM untuk membuat ringkasan otomatis.
    * Ringkasan disimpan di metadata dokumen.

7. **Selesai**:
    * Server mengirim event SSE `ProcessingCompleted` (100%).
    * Client menutup progress bar.

## 4. Error Handling (Mekanisme Kegagalan)

Apa yang terjadi jika proses gagal di tengah jalan (misal: file corrupt atau koneksi AI putus)?

1. **Log Error**: Error dicatat di server logs.
2. **Update Database Status**:
    * Kolom status di tabel tracking di-update menjadi `failed`.
    * **Penting**: Kueri progress bar di Client memfilter status `failed`.
    * **Efek**: Dokumen yang error akan otomatis **hilang/ditutup** dari tampilan progress bar Client, mencegah progress bar "stuck" selamanya.
3. **Notifikasi Error**:
    * Event SSE `ProcessingError` dikirim ke Client (opsional untuk menampilkan pesan error toast).
