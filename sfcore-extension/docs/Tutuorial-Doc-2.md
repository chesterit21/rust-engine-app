# Dokumentasi Debugging: Mengapa F5 Membuka Jendela Baru?

Dokumentasi ini menjelaskan "sihir" di balik tombol **F5** di VS Code saat mengembangkan ekstensi, menggunakan analogi yang familiar bagi developer .NET.

## Masalah: "Kenapa muncul VS Code baru?"

Saat Anda menekan **F5** di Solution Extension ini, tiba-tiba muncul jendela VS Code baru dengan judul **[Extension Development Host]**.

**Jawabannya:**
VS Code yang Anda gunakan untuk coding adalah **Editor**, sedangkan VS Code baru yang muncul adalah **Environment Debugging**.

### Analogi untuk Developer .NET

| Konsep VS Code Extension | Analogi Visual Studio / .NET | Penjelasan |
| :--- | :--- | :--- |
| **VS Code (Source Code)** | **Visual Studio IDE** | Tempat Anda menulis kode, compile, dan pasang breakpoint. |
| **F5 (Start Debugging)** | **F5 (Start Debugging)** | Perintah untuk menjalankan aplikasi dengan Debugger terlampir. |
| **Extension Development Host** | **IIS Express / Browser / Console Window** | Aplikasi berjalan di "Sandbox" terisolasi agar tidak merusak IDE utama. |
| **Reload Window** | **Hot Reload / Rebuild & Run** | Memuat ulang kode terbaru tanpa restart IDE utama. |

Bayangkan jika Anda membuat **Visual Studio Plugin**. Untuk mengetesnya, Visual Studio harus menjalankan instance Visual Studio *lain* (disebut "Experimental Instance") yang memuat plugin Anda. Jika tidak, crash di plugin Anda akan menutup IDE tempat Anda coding!

## Bedah Konfigurasi: Kunci Rahasia `.vscode/launch.json`

File `.vscode/launch.json` adalah "Startup Project Properties" Anda.

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Run Extension",      // Nama profil debug (muncul di dropdown)
            "type": "extensionHost",      // <--- KUNCI UTAMA
            "request": "launch",
            "args": [
                "--extensionDevelopmentPath=${workspaceFolder}" // Load folder ini sebagai extension
            ],
            "outFiles": [
                "${workspaceFolder}/dist/**/*.js" // Maps source code (TS) ke compiled code (JS)
            ],
            "preLaunchTask": "${defaultBuildTask}" // Build dulu sebelum run
        }
    ]
}
```

### Penjelasan Baris Penting:

1.  **`"type": "extensionHost"`**:
    Ini memberitahu VS Code: *"Hei, jangan jalankan ini sebagai Node.js app biasa. Jalankan ini di dalam instance VS Code baru khusus untuk testing ekstensi."*
    
    Analogi: Mirip dengan memilih **"IIS Express"** vs **"Project"** di Visual Studio. `"type": "node"` adalah Console App, `"type": "extensionHost"` adalah IIS Express.

2.  **`"args": ["--extensionDevelopmentPath=${workspaceFolder}"]`**:
    Parameter ini memerintahkan VS Code tamu untuk memuat ekstensi dari folder kerja saat ini.

3.  **`"preLaunchTask": "${defaultBuildTask}"`**:
    Sebelum debug jalan, jalankan task build. Cek `.vscode/tasks.json` untuk lihat isinya (biasanya `npm run watch`).
    
    Analogi: **Build before Run** di Visual Studio.

## Alur Eksekusi (The Flow)

Saat Anda tekan **F5**:

1.  **Pre-Launch (Build)**:
    VS Code menjalankan `npm run watch` (dari `tasks.json`).
    *   Webpack mengkompilasi TypeScript (`.ts`) dan React (`.tsx`) menjadi JavaScript bundle (`dist/extension.js` & `dist/webview.js`).
    *   *Analogi: MSBuild mengkompilasi .cs menjadi .dll.*

2.  **Launch Extension Host**:
    VS Code meluncurkan proses baru (jendela baru). Jendela ini bersih, terisolasi, dan memuat ekstensi Anda dari `package.json`.

3.  **Activation**:
    Ekstensi belum jalan sampai ada "pemicu" (Activation Event).
    Di `package.json`:
    ```json
    "activationEvents": [
        "onStartupFinished" // Langsung jalan begitu jendela baru siap
    ]
    ```
    Begitu aktif, fungsi `activate()` di `src/extension/extension.ts` dipanggil.
    *   *Analogi: `Application_Start` di Global.asax atau `Main()` method.*

4.  **Debugging**:
    VS Code utama (Parent) menempelkan (attach) debugger ke VS Code tamu (Child). Breakpoint di TypeScript akan kena (hit) karena ada *Source Maps* yang memetakan JS balik ke TS.

## Deep Dive: Engine & Toolchain

Apa yang sebenarnya menggerakkan semua ini? Engine apa yang perlu Anda install?

### 1. The Engine: Electron & Node.js

VS Code dibangun di atas **Electron**. Electron menggabungkan **Chromium** (Rendering Engine browser) dan **Node.js** (JavaScript Runtime).

*   **Chromium**: Bertugas merender UI VS Code (HTML/CSS). Inilah mengapa kita bisa pakai React di dalam Webview! Webview pada dasarnya adalah tab Chrome di dalam VS Code.
    *   *Analogi: WPF Rendering Engine (DirectX).*
*   **Node.js**: Bertugas menjalankan logika backend (Akses File System, Network, Spawn Process). Ekstensi Anda (`extension.ts`) berjalan di proses Node.js ini.
    *   *Analogi: .NET CLR (Common Language Runtime).*

Jadi, ketika kita bicara "Extension Host", itu sebenarnya adalah sebuah **Proses Node.js** yang menjalankan kode JavaScript ekstensi Anda.

### 2. Tools Development yang Diperlukan (The "SDK")

Untuk mengembangkan ekstensi ini, Anda perlu menginstall beberapa "Engine" atau Tools di komputer Anda. Berikut analoginya dengan dunia .NET:

| Tool di Sini (JS/TS Ecosystem) | Analogi Tool .NET | Fungsi |
| :--- | :--- | :--- |
| **Node.js** (Runtime) | **.NET Runtime / SDK** | "Mesin" untuk menjalankan JavaScript di luar browser. Wajib install. |
| **NPM** (Node Package Manager) | **NuGet** | Untuk download library (package) dari internet. Perintah `npm install` mirip `Restore-NuGetPackages`. |
| **TypeScript** (Language) | **C#** | Bahasa pemrograman yang punya tipe data (Static Types). Tidak bisa jalan langsung, harus di-compile ke JS. |
| **Webpack** (Bundler) | **MSBuild / Roslyn** | "Compiler" canggih yang mengambil ratusan file `.ts`, `.tsx`, `.css`, gambar, lalu membungkusnya jadi satu file `.js` yang efisien. |
| **Yeoman / VSCE** | **Visual Studio Template Studio** | Generator untuk membuat kerangka proyek ekstensi (`yo code`) dan packaging file `.vsix` (`vsce package`) untuk di-publish ke Marketplace. |

### Ringkasan Infrastruktur

```mermaid
flowchart TD
    subgraph "Computer Anda (Dev Machine)"
        Node[Node.js Runtime] --> NPM[NPM (NuGet analog)]
    end

    subgraph "Development Process (Saat Coding)"
        TS[Source Code (.ts/.tsx)] --> Webpack[Webpack Engine]
        Webpack --> Bundle[Bundled JS (.js)]
    end

    subgraph "Runtime Process (Saat F5)"
        Electron[Electron Engine (VS Code)]
        ExtHost[Extension Host (Node Process)]
        
        Electron -- "Runs" --> ExtHost
        ExtHost -- "Loads" --> Bundle
    end
```

## Tips untuk Developer

*   **Jangan tutup jendela host setiap kali coding**: Biarkan jendela `Extension Development Host` terbuka. Setelah edit kode, cukup tekan **Ctrl+R** (Reload Window) di jendela host, atau tekan tombol **Restart** (ikon panah melingkar hijau) di floating debug bar VS Code utama. Ini jauh lebih cepat daripada Stop-Start.
*   **Debug Console**: `console.log` dari ekstensi Anda akan muncul di **Debug Console** jendela utama, BUKAN di jendela host.
*   **Webview Developer Tools**: Untuk debug UI React (CSS/HTML), tekan `Ctrl+Shift+I` **di jendela host**, lalu inspect elemen webview. Ingat, Webview itu seperti browser Chrome di dalam VS Code.

## Kesimpulan

Tombol F5 memicu orkestrasi: **Compile -> Launch Sandbox -> Attach Debugger**.
Ini membuat Anda aman bereksperimen. Kalau kode Anda bikin crash, yang mati cuma jendela tamu ("Extension Host"), soure code Anda di jendela utama tetap aman.

Sangat mirip dengan debugging ASP.NET Core di IIS Express!
