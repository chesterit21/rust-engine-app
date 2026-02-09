//! LlamaCpp Engine - High-performance llama.cpp bindings (~20x faster than Candle)

use anyhow::{anyhow, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
use std::pin::pin;
use std::time::Instant;

// Flash attention types from llama.cpp (not re-exported from llama_cpp_2)
const FLASH_ATTN_DISABLED: i32 = 0;

/// Simple Chat Message struct for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Options for LlamaCpp engine - all sampling parameters
#[derive(Debug, Clone)]
pub struct LlamaCppOptions {
    pub threads: Option<i32>,
    pub threads_batch: Option<i32>,
    pub context_length: u32,
    pub batch_size: usize,
    pub ubatch_size: usize,
    pub seed: u32,
    pub use_mlock: bool,
    pub no_cache_prompt: bool,
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
            threads: Some(4),
            threads_batch: Some(4),
            context_length: 4096,
            batch_size: 2048,
            ubatch_size: 1024,
            seed: 1234,
            use_mlock: true,
            no_cache_prompt: false,
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

    /// Load model specifically for embedding generation
    pub fn load_gguf_embedding(&mut self, model_path: &str) -> Result<()> {
        let t0 = Instant::now();
        info!("loading GGUF embedding model: {}", model_path);
        
        let mut model_params = LlamaModelParams::default();
        if self.opts.use_mlock {
            model_params = model_params.with_use_mlock(true);
        }

        let model_params = pin!(model_params);
        let model = LlamaModel::load_from_file(&self.backend, model_path, &model_params)
            .with_context(|| format!("failed to load embedding model: {}", model_path))?;
        
        let load_ms = t0.elapsed().as_millis();
        info!("embedding model loaded in {} ms", load_ms);
        self.model = Some(model);
        Ok(())
    }

    /// Generate embeddings for input text
    pub fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let model = self
            .model
            .as_ref()
            .ok_or_else(|| anyhow!("model not loaded"))?;

        let t0 = Instant::now();
        
        // Create context with embeddings enabled
        let ctx_size = NonZeroU32::new(self.opts.context_length)
            .ok_or_else(|| anyhow!("context_length must be > 0"))?;
        
        let mut ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(ctx_size))
            .with_n_batch(self.opts.batch_size as u32)
            .with_embeddings(true)  // Enable embeddings mode
            .with_flash_attention_policy(FLASH_ATTN_DISABLED); // Disable flash attn for Windows compat

        if let Some(threads) = self.opts.threads {
            ctx_params = ctx_params.with_n_threads(threads);
        }

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .with_context(|| "failed to create embedding context")?;

        // Tokenize input
        let tokens = model
            .str_to_token(text, AddBos::Always)
            .with_context(|| "failed to tokenize text for embedding")?;

        let n_tokens = tokens.len();
        info!("Embedding: {} tokens", n_tokens);

        if n_tokens > self.opts.context_length as usize {
            return Err(anyhow!(
                "Text too long: {} tokens exceeds context {}",
                n_tokens,
                self.opts.context_length
            ));
        }

        // Create batch for all tokens
        let mut batch = LlamaBatch::new(n_tokens, 1);
        for (i, &token) in tokens.iter().enumerate() {
            batch.add(token, i as i32, &[0], i == n_tokens - 1)
                .map_err(|e| anyhow!("Failed to add token to batch: {}", e))?;
        }

        // Decode to get embeddings
        ctx.decode(&mut batch)
            .with_context(|| "Failed to decode for embeddings")?;

        // Get embeddings from the last token position
        let embeddings = ctx.embeddings_seq_ith(0)
            .with_context(|| "Failed to get embeddings")?;

        let embed_vec: Vec<f32> = embeddings.to_vec();
        
        let elapsed_ms = t0.elapsed().as_millis();
        info!("Embedding generated: {} dimensions in {} ms", embed_vec.len(), elapsed_ms);

        Ok(embed_vec)
    }

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
        info!("=== Generation Started ===");

        // ===== 1. CREATE CONTEXT =====
        let ctx_size = NonZeroU32::new(self.opts.context_length)
            .ok_or_else(|| anyhow!("context_length must be > 0"))?;
        
        let mut ctx_params = LlamaContextParams::default()
            .with_n_ctx(Some(ctx_size))
            .with_n_batch(self.opts.batch_size as u32)
            .with_n_ubatch(self.opts.ubatch_size as u32)
            .with_flash_attention_policy(FLASH_ATTN_DISABLED); // Disable flash attn for Windows compat

        if let Some(threads) = self.opts.threads {
            ctx_params = ctx_params.with_n_threads(threads);
        }

        if let Some(threads_batch) = self.opts.threads_batch {
            ctx_params = ctx_params.with_n_threads_batch(threads_batch);
        } else if let Some(threads) = self.opts.threads {
            ctx_params = ctx_params.with_n_threads_batch(threads);
        }

        let mut ctx = model
            .new_context(&self.backend, ctx_params)
            .with_context(|| "failed to create context")?;

        // ===== 2. TOKENIZE PROMPT =====
        let tokens_list = model
            .str_to_token(prompt, AddBos::Always)
            .with_context(|| "failed to tokenize prompt")?;
        
        let n_prompt = tokens_list.len();
        info!("Prompt tokens: {}", n_prompt);

        // Validate total length
        if n_prompt + max_tokens as usize > self.opts.context_length as usize {
            return Err(anyhow!(
                "Total tokens ({} prompt + {} max = {}) exceeds context {}",
                n_prompt,
                max_tokens,
                n_prompt + max_tokens as usize,
                self.opts.context_length
            ));
        }

        // ===== 3. CHUNKED PREFILL (CRITICAL FIX) =====
        let batch_size = self.opts.batch_size;
        let mut n_past = 0i32;
        
        info!("Prefill: {} tokens in chunks of {}", n_prompt, batch_size);

        let mut token_idx = 0;
        while token_idx < n_prompt {
            let chunk_size = std::cmp::min(batch_size, n_prompt - token_idx);
            let chunk = &tokens_list[token_idx..token_idx + chunk_size];
            
            debug!("Processing chunk {}-{} (size={})", token_idx, token_idx + chunk_size, chunk_size);
            
            // Create batch with proper size
            let mut batch = LlamaBatch::new(batch_size, 1);
            
            // ✅ CRITICAL FIX: Only enable logits for LAST token of ENTIRE prompt
            let is_last_chunk = (token_idx + chunk_size) == n_prompt;
            
            for (i, &token) in chunk.iter().enumerate() {
                let pos = n_past + i as i32;
                let is_last_token_in_chunk = i == (chunk_size - 1);
                
                // Only request logits for the very last token of the entire prompt
                let need_logits = is_last_chunk && is_last_token_in_chunk;
                
                if let Err(e) = batch.add(token, pos, &[0], need_logits) {
                    error!("Failed to add token {} to batch: {}", token, e);
                    return Err(anyhow!("Batch add failed: {}", e));
                }
            }
            
            // Decode this chunk with error handling + panic catch
            let decode_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                ctx.decode(&mut batch)
            }));
            
            match decode_result {
                Ok(Ok(())) => { /* success */ }
                Ok(Err(e)) => {
                    error!("Prefill decode failed at chunk {}-{}: {}", 
                           token_idx, token_idx + chunk_size, e);
                    return Err(anyhow!("Prefill decode failed at tokens {}-{}: {}", 
                                      token_idx, token_idx + chunk_size, e));
                }
                Err(panic_info) => {
                    error!("Prefill decode PANIC at chunk {}-{}: {:?}", 
                           token_idx, token_idx + chunk_size, panic_info);
                    return Err(anyhow!("Prefill decode panicked at tokens {}-{}", 
                                      token_idx, token_idx + chunk_size));
                }
            }
            
            n_past += chunk_size as i32;
            token_idx += chunk_size;
        }
        
        let prefill_ms = t_start.elapsed().as_millis();
        info!("Prefill completed: {} ms, n_past={}", prefill_ms, n_past);

        // ===== 4. GENERATION LOOP (CRITICAL FIX) =====
        let mut n_decode = 0;
        let mut output = String::new();
        let t_gen_start = Instant::now();
        let mut first_token_time: Option<u128> = None;
        let mut decoder = encoding_rs::UTF_8.new_decoder();

        // Create sampler with try-catch
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

        info!("Starting generation loop (max_tokens={})", max_tokens);

        while n_decode < max_tokens {
            // Check context limit
            if n_past >= self.opts.context_length as i32 {
                warn!("Context limit reached at {} tokens", n_past);
                break;
            }
            
            // ✅ CRITICAL FIX: Sample from index -1 (last token with logits)
            // The previous batch had logits=true for the last token
            let token = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                sampler.sample(&ctx, -1)
            })) {
                Ok(t) => t,
                Err(e) => {
                    error!("Sampling panic at n_decode={}, n_past={}: {:?}", n_decode, n_past, e);
                    return Err(anyhow!("Sampling failed at token {}", n_decode));
                }
            };
            
            sampler.accept(token);

            // Record first token time
            if first_token_time.is_none() {
                first_token_time = Some(t_start.elapsed().as_millis());
                debug!("First token generated at {} ms", first_token_time.unwrap());
            }

            // Check for end-of-generation
            if model.is_eog_token(token) {
                info!("EOG token detected at position {}", n_decode);
                break;
            }

            // Decode token to string with error handling
            let output_bytes = match model.token_to_bytes(token, Special::Tokenize) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to decode token {}: {}", token, e);
                    return Err(anyhow!("Token decode failed: {}", e));
                }
            };
            
            let mut token_str = String::with_capacity(32);
            let _ = decoder.decode_to_string(&output_bytes, &mut token_str, false);
            output.push_str(&token_str);

            // Call user callback
            if !callback(token_str) {
                info!("Generation stopped by callback at token {}", n_decode);
                break;
            }

            // ✅ CRITICAL: Add next token to context with logits=true
            let mut batch = LlamaBatch::new(1, 1);
            if let Err(e) = batch.add(token, n_past, &[0], true) {
                error!("Failed to add generated token to batch: {}", e);
                return Err(anyhow!("Batch add failed during generation: {}", e));
            }
            
            // Decode with error handling
            if let Err(e) = ctx.decode(&mut batch) {
                error!("Decode failed at generation step {}: {}", n_decode, e);
                return Err(anyhow!("Generation decode failed at token {}: {}", n_decode, e));
            }
            
            n_past += 1;
            n_decode += 1;
        }

        let total_ms = t_start.elapsed().as_millis();
        let gen_ms = t_gen_start.elapsed().as_millis();
        let tokens_per_sec = if gen_ms > 0 {
            (n_decode as f32) / (gen_ms as f32 / 1000.0)
        } else {
            0.0
        };

        info!("=== Generation Complete ===");
        info!("Tokens generated: {}", n_decode);
        info!("Speed: {:.2} tok/s", tokens_per_sec);
        info!("Total time: {} ms", total_ms);

        Ok(GenerationResult {
            output,
            tokens_generated: n_decode,
            prefill_ms,
            first_token_ms: first_token_time.unwrap_or(0),
            total_ms,
            tokens_per_sec,
        })
    }

    pub fn generate(&self, prompt: &str, max_tokens: i32) -> Result<GenerationResult> {
        self.generate_with_callback(prompt, max_tokens, |token| {
            print!("{}", token);
            let _ = std::io::Write::flush(&mut std::io::stdout());
            true
        })
    }
}

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