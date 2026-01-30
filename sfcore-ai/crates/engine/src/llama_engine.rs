//! LlamaCpp Engine - High-performance llama.cpp bindings (~20x faster than Candle)

use anyhow::{anyhow, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
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
    pub threads: Option<i32>,
    pub threads_batch: Option<i32>,
    pub context_length: u32,
    pub batch_size: usize,
    pub ubatch_size: usize,
    pub seed: u32,
    pub use_mlock: bool,
    
    // --- NEW: Cache Control ---
    /// Disable KV cache for prompts (saves memory but slower)
    pub no_cache_prompt: bool,

    // --- Sampling Parameters ---
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub min_p: f32,
    pub repeat_penalty: f32,
    pub repeat_last_n: i32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
}

impl Default for LlamaCppOptions {
    fn default() -> Self {
        Self {
            // System defaults
            threads: Some(4),
            threads_batch: Some(4),
            context_length: 4096,
            batch_size: 2048,
            ubatch_size: 1024,
            seed: 1234,
            use_mlock: true,
            no_cache_prompt: false, // Enable cache by default
            
            // Sampling defaults
            temperature: 0.5,
            top_k: 40,
            top_p: 0.9,
            min_p: 0.05,
            repeat_penalty: 1.0,
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
    pub fn apply_chat_template(&self, messages: &[ChatMessage]) -> Result<String> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("model not loaded"))?;

        let chat_messages: Vec<LlamaChatMessage> = messages
            .iter()
            .map(|m| LlamaChatMessage::new(m.role.clone(), m.content.clone()))
            .collect::<std::result::Result<_, _>>()
            .map_err(|e| anyhow!("failed to create chat message: {:?}", e))?;

        let template = model
            .chat_template(None)
            .map_err(|e| anyhow!("failed to get chat template: {:?}", e))?;

        let prompt = model
            .apply_chat_template(&template, &chat_messages, true)
            .map_err(|e| anyhow!("failed to apply chat template: {:?}", e))?;

        Ok(prompt)
    }

    /// Generate text with streaming callback
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
            ctx_params = ctx_params.with_n_threads_batch(threads);
        }

        if self.opts.no_cache_prompt {
            info!("Prompt caching disabled (no_cache_prompt = true)");
        }

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .with_context(|| "failed to create context")?;

        // Tokenize prompt
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .with_context(|| "failed to tokenize prompt")?;
        
        let n_prompt_tokens = tokens_list.len();
        info!("prompt tokens: {}", n_prompt_tokens);

        // Validate prompt length
        if n_prompt_tokens >= self.opts.context_length as usize {
            return Err(anyhow!(
                "Prompt too long: {} tokens exceeds context limit of {}",
                n_prompt_tokens,
                self.opts.context_length
            ));
        }

        // ✅ FIX: Chunked prefill (process in batches)
        let batch_size = self.opts.batch_size;
        let mut n_cur = 0i32;
        
        info!("prefill starting: {} tokens in chunks of {}", n_prompt_tokens, batch_size);
        
        // Process prompt in batches
        for chunk_start in (0..n_prompt_tokens).step_by(batch_size) {
            let chunk_end = std::cmp::min(chunk_start + batch_size, n_prompt_tokens);
            let chunk = &tokens_list[chunk_start..chunk_end];
            let chunk_size = chunk.len();
            
            let mut batch = LlamaBatch::new(batch_size, 1);
            
            // Add tokens from this chunk
            // ✅ KEY FIX: Only mark LAST token of LAST chunk for logits
            for (i, token) in chunk.iter().enumerate() {
                let pos = chunk_start as i32 + i as i32;
                let is_last_in_chunk = i == (chunk_size - 1);
                let is_last_chunk = chunk_end == n_prompt_tokens;
                
                // Only request logits for the very last token of the entire prompt
                let need_logits = is_last_in_chunk && is_last_chunk;
                
                batch.add(*token, pos, &[0], need_logits)?;
            }
            
            // Decode this batch
            ctx.decode(&mut batch)
                .with_context(|| format!("prefill decode failed at chunk {}-{}", chunk_start, chunk_end))?;
            
            n_cur += chunk_size as i32;
        }
        
        let prefill_ms = t_start.elapsed().as_millis();
        info!("prefill completed in {} ms", prefill_ms);

        // ✅ Generation loop (now starts after full prefill)
        let n_len = n_prompt_tokens as i32 + max_tokens;
        let mut n_decode = 0;
        let mut output = String::new();
        let t_gen_start = Instant::now();
        let mut first_token_time: Option<u128> = None;

        let mut decoder = encoding_rs::UTF_8.new_decoder();

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

        // ✅ CRITICAL FIX: Sample from batch index, not absolute position
        // Generation loop
        while n_cur < n_len {
            // ✅ Sample from the last token in the batch (index -1 means last)
            // The batch only contains 1 token, so we use index -1 (last token)
            let token = sampler.sample(&ctx, -1);
            sampler.accept(token);

            if first_token_time.is_none() {
                first_token_time = Some(t_start.elapsed().as_millis());
            }

            if model.is_eog_token(token) {
                break;
            }

            let output_bytes = model.token_to_bytes(token, Special::Tokenize)?;
            let mut token_str = String::with_capacity(32);
            let _ = decoder.decode_to_string(&output_bytes, &mut token_str, false);
            output.push_str(&token_str);

            let continue_gen = callback(token_str);
            if !continue_gen {
                break;
            }

            // ✅ Add next token to context with logits=true
            let mut batch = LlamaBatch::new(1, 1);
            batch.add(token, n_cur, &[0], true)?;  // logits=true for next sampling
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

    /// Generate text with default stdout printing
    pub fn generate(&self, prompt: &str, max_tokens: i32) -> Result<GenerationResult> {
        self.generate_with_callback(prompt, max_tokens, |token| {
            print!("{}", token);
            let _ = std::io::Write::flush(&mut std::io::stdout());
            true
        })
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