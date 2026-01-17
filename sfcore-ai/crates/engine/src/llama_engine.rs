//! LlamaCpp Engine - High-performance llama.cpp bindings (~20x faster than Candle)

use anyhow::{anyhow, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaChatTemplate, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use log::info;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::pin::pin;
use std::time::Instant;

/// Simple Chat Message struct for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Options for LlamaCpp engine - all sampling parameters
#[derive(Debug, Clone)]
pub struct LlamaCppOptions {
    // --- System Parameters ---
    /// Jumlah thread untuk decoding (generasi token per token).
    /// Disarankan: 50-75% dari physical cores (misal 2-3 untuk 4 cores).
    /// Jangan set max cores agar OS tidak macet.
    pub threads: Option<i32>,

    /// Jumlah thread untuk prefill (prompt processing) dan batching.
    /// Ini sangat berpengaruh saat prompt awal panjang.
    /// Disarankan: Setara physical cores (misal 4 untuk 4 cores).
    pub threads_batch: Option<i32>,

    /// Panjang konteks maksimal (prompt + output).
    /// Hati-hati: Semakin besar, semakin boros RAM (KV Cache).
    /// Default: 2048 (cukup untuk chat pendek/sedang).
    pub context_length: u32,

    /// Logical batch size (maksimal token yang diproses sekaligus).
    /// Lebih besar = prefill lebih cepat tapi butuh RAM lebih.
    /// Default: 512-2048.
    pub batch_size: usize,

    /// Physical batch size (sub-batch yang dieksekusi per step).
    /// Pecahan dari batch_size untuk efisiensi L2 Cache CPU.
    /// Default: 512 (sweet spot untuk banyak CPU modern).
    pub ubatch_size: usize,

    /// Seed untuk Random Number Generator (RNG).
    /// Set nilai tetap untuk hasil yang deterministik (reproducible).
    pub seed: u32,

    /// Jika true, kunci model di RAM agar tidak kena swap ke disk.
    /// Sangat disarankan jika RAM cukup, mencegah stuttering.
    /// Default: false (aman untuk RAM pas-pasan).
    pub use_mlock: bool,

    // --- Sampling Parameters ---
    /// Mengontrol keacakan output (Creativity).
    /// - 0.0: Greedy decoding (selalu pilih yang paling mungkin / kaku).
    /// - 0.7: Balanced (kreatif tapi logis).
    /// - >1.0: Sangat acak / halusinasi.
    pub temperature: f32,

    /// Membatasi pilihan token hanya pada K token teratas.
    /// - 40: Nilai default umum.
    /// - 0: Disabled (pertimbangkan semua token di vocab).
    pub top_k: i32,

    /// Nucleus Sampling: Ambil token teratas dengan total probabilitas P.
    /// - 0.9: Filter ekor panjang probabilitas rendah.
    /// - 1.0: Disabled.
    pub top_p: f32,

    /// Minimum Probability: Buang token yang probabilitasnya < P * prob token terbaik.
    /// - 0.05: Filter token sampah/typo yang sangat tidak mungkin.
    pub min_p: f32,

    // --- Penalties (Anti-Repetition) ---
    /// Hukuman Multiplikatif untuk token yang sudah muncul.
    /// - 1.0: Disabled (tanpa hukuman).
    /// - 1.1 - 1.2: Cukup untuk mencegah looping ringan.
    pub repeat_penalty: f32,

    /// Jumlah token terakhir yang dicek untuk penalti (Context window lookback).
    /// - 64: Cek 64 token terakhir.
    /// - 0: Cek seluruh konteks (lambat).
    pub repeat_last_n: i32,

    /// Hukuman Aditif berdasarkan seberapa sering token muncul (Frequency).
    /// Efek: Mencegah kata yang SAMA diulang-ulang berlebihan.
    pub frequency_penalty: f32,

    /// Hukuman Aditif jika token SUDAH pernah muncul (Presence).
    /// Efek: Memaksa model membicarakan topik/hal BARU (bukan sekadar kata beda).
    pub presence_penalty: f32,
}

impl Default for LlamaCppOptions {
    fn default() -> Self {
        Self {
            // System defaults
            threads: Some(4),
            threads_batch: Some(4),
            context_length: 4096, // 4K context
            batch_size: 2048,
            ubatch_size: 1024,

            seed: 1234,
            use_mlock: true,

            // Sampling defaults
            temperature: 0.5, // Balanced
            top_k: 40,        // Common default
            top_p: 0.9,       // Nucleus sampling
            min_p: 0.05,      // Filter very unlikely tokens

            // Repetition defaults (light penalty)
            repeat_penalty: 1.0, // Off by default
            repeat_last_n: 64,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
        }
    }
}

/// High-performance LLM engine using llama.cpp
pub struct LlamaCppEngine {
    backend: LlamaBackend,
    model: Option<LlamaModel>,
    opts: LlamaCppOptions,
}

impl LlamaCppEngine {
    /// Create a new LlamaCpp engine
    pub fn new(opts: LlamaCppOptions) -> Result<Self> {
        let backend =
            LlamaBackend::init().map_err(|e| anyhow!("failed to init llama backend: {e}"))?;
        info!("LlamaCpp backend initialized");
        Ok(Self {
            backend,
            model: None,
            opts,
        })
    }

    /// Load a GGUF model file
    pub fn load_gguf(&mut self, model_path: &str) -> Result<()> {
        let t0 = Instant::now();
        info!("loading GGUF model: {}", model_path);

        let mut model_params = LlamaModelParams::default();
        if self.opts.use_mlock {
            model_params = model_params.with_use_mlock(true);
        }
        let model_params = pin!(model_params);

        let model = LlamaModel::load_from_file(&self.backend, model_path, &model_params)
            .with_context(|| format!("failed to load model: {}", model_path))?;

        let load_ms = t0.elapsed().as_millis();
        info!("model loaded in {} ms", load_ms);

        self.model = Some(model);
        Ok(())
    }

    /// Apply chat template to a list of messages.
    /// Returns the formatted prompt string.
    pub fn apply_chat_template(&self, messages: &[ChatMessage]) -> Result<String> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("model not loaded"))?;

        // Convert to LlamaChatMessage
        let chat_messages: Vec<LlamaChatMessage> = messages
            .iter()
            .map(|m| LlamaChatMessage::new(m.role.clone(), m.content.clone()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow!("failed to create chat message: {:?}", e))?;

        // Get template (None = usage default from model)
        let template = model
            .chat_template(None)
            .map_err(|e| anyhow!("failed to get chat template: {:?}", e))?;

        // Apply
        let prompt = model
            .apply_chat_template(&template, &chat_messages, true)
            .map_err(|e| anyhow!("failed to apply chat template: {:?}", e))?;

        Ok(prompt)
    }

    /// Generate text with streaming callback
    /// Callback receives token string, returns true to continue, false to abort
    pub fn generate_with_callback<F>(
        &self,
        prompt: &str,
        max_tokens: i32,
        mut callback: F,
    ) -> Result<GenerationResult>
    where
        F: FnMut(String) -> bool,
    {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("model not loaded"))?;

        let t_start = Instant::now();

        // Create context
        let ctx_size = NonZeroU32::new(self.opts.context_length).unwrap();
        let mut ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(ctx_size))
            .with_n_batch(self.opts.batch_size as u32)
            .with_n_ubatch(self.opts.ubatch_size as u32);

        if let Some(threads) = self.opts.threads {
            ctx_params = ctx_params.with_n_threads(threads);
        }
        if let Some(threads_batch) = self.opts.threads_batch {
            ctx_params = ctx_params.with_n_threads_batch(threads_batch);
        } else if let Some(threads) = self.opts.threads {
            // Fallback to threads if threads_batch not set
            ctx_params = ctx_params.with_n_threads_batch(threads);
        }

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .with_context(|| "failed to create context")?;

        // Tokenize prompt
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .with_context(|| "failed to tokenize prompt")?;

        info!("prompt tokens: {}", tokens_list.len());

        // Create batch (optimized size)
        let mut batch = LlamaBatch::new(self.opts.batch_size, 1);

        let last_index = (tokens_list.len() - 1) as i32;
        for (i, token) in (0_i32..).zip(tokens_list.iter()) {
            let is_last = i == last_index;
            batch.add(*token, i, &[0], is_last)?;
        }

        // Initial decode (prefill)
        ctx.decode(&mut batch)
            .with_context(|| "prefill decode failed")?;

        let prefill_ms = t_start.elapsed().as_millis();

        // Generation loop
        let mut n_cur = batch.n_tokens();
        let n_len = tokens_list.len() as i32 + max_tokens;
        let mut n_decode = 0;
        let mut output = String::new();

        let t_gen_start = Instant::now();
        let mut first_token_time: Option<u128> = None;

        // UTF-8 decoder
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        // Sampler chain dengan semua parameters
        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::penalties(
                self.opts.repeat_last_n,
                self.opts.repeat_penalty,
                self.opts.frequency_penalty,
                self.opts.presence_penalty,
            ),
            LlamaSampler::top_k(self.opts.top_k),
            LlamaSampler::top_p(self.opts.top_p, 1),
            LlamaSampler::min_p(self.opts.min_p, 1),
            LlamaSampler::temp(self.opts.temperature),
            LlamaSampler::dist(self.opts.seed),
        ]);

        while n_cur < n_len {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            // Record first token time
            if first_token_time.is_none() {
                first_token_time = Some(t_start.elapsed().as_millis());
            }

            // Check end of generation
            if model.is_eog_token(token) {
                break;
            }

            // Decode token to string
            let output_bytes = model.token_to_bytes(token, Special::Tokenize)?;
            let mut token_str = String::with_capacity(32);
            let _ = decoder.decode_to_string(&output_bytes, &mut token_str, false);

            output.push_str(&token_str);

            // Invok callback
            let continue_gen = callback(token_str);
            if !continue_gen {
                break;
            }

            // Prepare next batch
            batch.clear();
            batch.add(token, n_cur, &[0], true)?;

            n_cur += 1;
            ctx.decode(&mut batch).with_context(|| "decode failed")?;
            n_decode += 1;
        }

        let total_ms = t_start.elapsed().as_millis();
        let gen_ms = t_gen_start.elapsed().as_millis();
        let tokens_per_sec = if gen_ms > 0 {
            (n_decode as f32) / (gen_ms as f32 / 1000.0)
        } else {
            0.0
        };

        Ok(GenerationResult {
            output,
            tokens_generated: n_decode,
            prefill_ms,
            first_token_ms: first_token_time.unwrap_or(0),
            total_ms,
            tokens_per_sec,
        })
    }

    /// Generate text with default stdout printing (CLI compatibility)
    pub fn generate(&self, prompt: &str, max_tokens: i32) -> Result<GenerationResult> {
        self.generate_with_callback(prompt, max_tokens, |token| {
            print!("{}", token);
            let _ = std::io::Write::flush(&mut std::io::stdout());
            true // continue
        })
        // Note: println!() is done by caller in main.rs or separate
    }
}

/// Result of text generation
#[derive(Debug)]
pub struct GenerationResult {
    pub output: String,
    pub tokens_generated: i32,
    pub prefill_ms: u128,
    pub first_token_ms: u128,
    pub total_ms: u128,
    pub tokens_per_sec: f32,
}

impl std::fmt::Display for GenerationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[metrics] ftl: {} ms, tokens: {}, time: {} ms, speed: {:.2} tok/s",
            self.first_token_ms, self.tokens_generated, self.total_ms, self.tokens_per_sec
        )
    }
}
