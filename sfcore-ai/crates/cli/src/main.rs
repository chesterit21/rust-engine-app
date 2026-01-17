use anyhow::Result;
use clap::Parser;
use log::info;
use sfcore_ai_engine::{metrics, LlamaCppEngine, LlamaCppOptions};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Parser, Debug)]
#[command(
    name = "sfcore-ai-cli",
    version,
    about = "SFCore AI - LLM Inference CLI"
)]
struct Args {
    /// Path to GGUF model file
    #[arg(long)]
    model: String,

    /// Prompt text
    #[arg(long, default_value = "Hello")]
    prompt: String,

    /// Maximum tokens to generate
    #[arg(long, default_value_t = 4096)]
    max_tokens: usize,

    // === System Options ===
    /// Number of CPU threads for decoding
    #[arg(long, default_value_t = 4)]
    threads: i32,

    /// Number of CPU threads for batch processing (prefill)
    #[arg(long, default_value_t = 4)]
    threads_batch: i32,

    /// Context length (max tokens in context)
    #[arg(long, default_value_t = 4096)]
    context_length: u32,

    /// Logical batch size
    #[arg(long, default_value_t = 2048)]
    batch_size: usize,

    /// Physical batch size (ubatch)
    #[arg(long, default_value_t = 1024)]
    ubatch_size: usize,

    /// Random Seed (Benih Acak).
    ///
    /// Menentukan titik awal generator angka acak.
    ///
    /// **Gunakan untuk**: Reprodusibilitas (hasil yang konsisten).
    ///
    /// **Efek Nilai**:
    /// - 1234 (Default): Hasil akan selalu sama jika prompt & parameter lain sama.
    /// - Ganti nilai untuk variasi output pada prompt yang sama.
    #[arg(long, default_value_t = 1234)]
    seed: u32,

    /// MEMORY: Lock Model in RAM (prevent swap)
    #[arg(long, default_value_t = true)]
    mlock: bool,

    // === Sampling Options ===
    /// Temperature (Suhu / Kreativitas).
    ///
    /// Mengontrol seberapa "liar" model dalam memilih kata.
    ///
    /// **Efek Nilai**:
    /// - 0.0: Greedy (Pasti pilih kata probabilitas tertinggi). Hasil kaku, repetitif, tapi logis.
    /// - 0.5 (Default): Seimbang antara kreatif dan koheren.
    /// - 1.0+: Kreatif tapi berisiko halusinasi/ngaco.
    ///
    /// **Max Value**: Biasanya 2.0 (sangat absurd).
    #[arg(long, default_value_t = 0.5)]
    temperature: f32,

    /// Top-K Sampling.
    ///
    /// Membatasi pilihan hanya pada K token dengan probabilitas tertinggi.
    ///
    /// **Gunakan untuk**: Mencegah model memilih kata yang sangat aneh/jarang.
    ///
    /// **Efek Nilai**:
    /// - 40 (Default): Standar industri. Pilihan cukup beragam tapi aman.
    /// - 1: Sama dengan Greedy decoding.
    /// - 0: Disabled (semua vocabulary dipertimbangkan, lambat & berisiko).
    #[arg(long, default_value_t = 40)]
    top_k: i32,

    /// Top-P (Nucleus) Sampling.
    ///
    /// Memilih token dari urutan teratas cumulative probability P.
    /// Lebih dinamis daripada Top-K karena jumlah opsi menyesuaikan kepastian model.
    ///
    /// **Efek Nilai**:
    /// - 0.9 (Default): Membuang 10% opsi terbawah (ekor panjang) yang tidak mungkin.
    /// - 1.0: Disabled (terima semua kemungkinan).
    /// - < 0.5: Sangat fokus dan kaku.
    #[arg(long, default_value_t = 0.8)]
    top_p: f32,

    /// Min-P Sampling.
    ///
    /// Metode modern! Membuang token yang probabilitasnya < P * (probabilitas token terbaik).
    ///
    /// **Gunakan untuk**: Membersihkan opsi 'sampah' tanpa memotong opsi kreatif yang valid.
    ///
    /// **Efek Nilai**:
    /// - 0.05 (Default): Hapus token yang 20x lebih kecil peluangnya dari token terbaik.
    /// - 0.0: Disabled.
    #[arg(long, default_value_t = 0.04)]
    min_p: f32,

    // === Repetition Penalties ===
    /// Repeat penalty (1.0 = off, 1.1 = light)
    #[arg(long, default_value_t = 1.1)]
    repeat_penalty: f32,

    /// Tokens to check for repetition
    #[arg(long, default_value_t = 64)]
    repeat_last_n: i32,

    /// Frequency Penalty (Hukuman Frekuensi).
    ///
    /// Menghukum token berdasarkan BERAPA KALI token itu sudah muncul.
    ///
    /// **Gunakan untuk**: Mengurangi repetisi kata per kata (verbatim).
    ///
    /// **Efek Nilai**:
    /// - 0.0 (Default): Tidak ada hukuman.
    /// - 0.1 - 1.0: Mengurangi kecenderungan mengulang kata yang sama.
    /// - > 1.5: Model akan sangat menghindari kata yang sudah dipakai. Kalimat bisa jadi tidak gramatikal karena kehabisan kata sambung.
    ///
    /// **Max Value**: Secara teknis tidak terbatas, tapi > 2.0 biasanya merusak output.
    #[arg(long, default_value_t = 0.5)]
    frequency_penalty: f32,

    /// Presence Penalty (Hukuman Kehadiran).
    ///
    /// Menghukum token jika SUDAH PERNAH muncul (sekali saja cukup).
    ///
    /// **Gunakan untuk**: Memaksa model membicarakan TOPIK BARU (bukan sekadar kata beda).
    ///
    /// **Efek Nilai**:
    /// - 0.0 (Default): Tidak ada hukuman.
    /// - 0.1 - 1.0: Mendorong model untuk berpindah topik atau menggunakan sinonim.
    /// - > 1.5: Model mungkin halusinasi atau ganti bahasa karena takut pakai kata umum yang sudah keluar.
    ///
    /// **Max Value**: > 2.0 merusak output.
    #[arg(long, default_value_t = 0.2)]
    presence_penalty: f32,
}

fn main() -> Result<()> {
    // Set OpenBLAS threads to 1 to avoid oversubscription
    std::env::set_var("OPENBLAS_NUM_THREADS", "1");
    // Also MKL just in case
    std::env::set_var("MKL_NUM_THREADS", "1");

    env_logger::init();
    let args = Args::parse();

    info!("SFCore AI CLI - llama.cpp backend");

    let before = metrics::RuntimeMetrics::capture();

    let mut engine = LlamaCppEngine::new(LlamaCppOptions {
        threads: Some(args.threads),
        threads_batch: Some(args.threads_batch),
        context_length: args.context_length,
        batch_size: args.batch_size,
        ubatch_size: args.ubatch_size,
        seed: args.seed,
        use_mlock: args.mlock,
        temperature: args.temperature,
        top_k: args.top_k,
        top_p: args.top_p,
        min_p: args.min_p,
        repeat_penalty: args.repeat_penalty,
        repeat_last_n: args.repeat_last_n,
        frequency_penalty: args.frequency_penalty,
        presence_penalty: args.presence_penalty,
    })?;

    engine.load_gguf(&args.model)?;

    info!("Generate: prompt='{}'", args.prompt);

    let result = engine.generate(&args.prompt, args.max_tokens as i32)?;

    let after = metrics::RuntimeMetrics::capture();

    eprintln!("{}", result);
    eprintln!(
        "[memory] rss: {:.1} -> {:.1} MB",
        before.process_rss_mb, after.process_rss_mb
    );

    Ok(())
}
