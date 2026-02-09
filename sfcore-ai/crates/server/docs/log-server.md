.\sfcore-ai-server.exe : [2026-02-01T15:42:50Z INFO  sfcore_ai_server] SFCore AI Server starting (standalone mode)...
At line:1 char:1

+ .\sfcore-ai-server.exe 2>&1 | Out-File -FilePath "crash.log" -Encodin ...

+ ~~~~~~~~~~~~~~~~~~~~~~~~~~~
    + CategoryInfo          : NotSpecified: ([2026-02-01T15:...dalone mode)...:String) [], RemoteException
    + FullyQualifiedErrorId : NativeCommandError

[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Model: C:\Program Files\RustServer\Models\qwen2.5-coder-1.5b-instruct-q5_k_m.gguf
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Transport: http-sse
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] ≡ƒÄ▓ Random Seed Generated: 434044965
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] === Engine Configuration ===
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Context Length : 8096
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Batch / UBatch : 512 / 256
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Threads        : Some(8) / Some(8)
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] mlock          : true
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] no_cache_prompt: false
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Temp / TopK / TopP / MinP: 0.7 / 60 / 0.9 / 0.05
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] Model Mode     : LLM
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] ===========================
[2026-02-01T15:42:50Z INFO  tracing::span] init;
[2026-02-01T15:42:50Z INFO  sfcore_ai_engine::llama_engine] LlamaCpp backend initialized
[2026-02-01T15:42:50Z INFO  sfcore_ai_server] ≡ƒÆ¼ Loading LLM model...
[2026-02-01T15:42:50Z INFO  sfcore_ai_engine::llama_engine] loading GGUF model: C:\Program
Files\RustServer\Models\qwen2.5-coder-1.5b-instruct-q5_k_m.gguf
[2026-02-01T15:42:50Z INFO  llama_cpp_2::model] load_from_file;
llama_model_loader: direct I/O is enabled, disabling mmap
llama_model_loader: loaded meta data with 26 key-value pairs and 339 tensors from C:\Program
Files\RustServer\Models\qwen2.5-coder-1.5b-instruct-q5_k_m.gguf (version GGUF V3 (latest))
llama_model_loader: Dumping metadata keys/values. Note: KV overrides do not apply in this output.
llama_model_loader: - kv   0:                       general.architecture str              = qwen2
llama_model_loader: - kv   1:                               general.type str              = model
llama_model_loader: - kv   2:                               general.name str              = Qwen2.5 Coder 1.5B Instruct GGUF
llama_model_loader: - kv   3:                           general.finetune str              = Instruct-GGUF
llama_model_loader: - kv   4:                           general.basename str              = Qwen2.5-Coder
llama_model_loader: - kv   5:                         general.size_label str              = 1.5B
llama_model_loader: - kv   6:                          qwen2.block_count u32              = 28
llama_model_loader: - kv   7:                       qwen2.context_length u32              = 32768
llama_model_loader: - kv   8:                     qwen2.embedding_length u32              = 1536
llama_model_loader: - kv   9:                  qwen2.feed_forward_length u32              = 8960
llama_model_loader: - kv  10:                 qwen2.attention.head_count u32              = 12
llama_model_loader: - kv  11:              qwen2.attention.head_count_kv u32              = 2
llama_model_loader: - kv  12:                       qwen2.rope.freq_base f32              = 1000000.000000
llama_model_loader: - kv  13:     qwen2.attention.layer_norm_rms_epsilon f32              = 0.000001
llama_model_loader: - kv  14:                          general.file_type u32              = 17
llama_model_loader: - kv  15:                       tokenizer.ggml.model str              = gpt2
llama_model_loader: - kv  16:                         tokenizer.ggml.pre str              = qwen2
llama_model_loader: - kv  17:                      tokenizer.ggml.tokens arr[str,151936]  = ["!", "\"", "#", "$", "%", "&", "'", ...
llama_model_loader: - kv  18:                  tokenizer.ggml.token_type arr[i32,151936]  = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, ...
llama_model_loader: - kv  19:                      tokenizer.ggml.merges arr[str,151387]  = ["─á ─á", "─á─á ─á─á", "i n", "─á t",...
llama_model_loader: - kv  20:                tokenizer.ggml.eos_token_id u32              = 151645
llama_model_loader: - kv  21:            tokenizer.ggml.padding_token_id u32              = 151643
llama_model_loader: - kv  22:                tokenizer.ggml.bos_token_id u32              = 151643
llama_model_loader: - kv  23:               tokenizer.ggml.add_bos_token bool             = false
llama_model_loader: - kv  24:                    tokenizer.chat_template str              = {%- if tools %}\n    {{- '<|im_start|>...
llama_model_loader: - kv  25:               general.quantization_version u32              = 2
llama_model_loader: - type  f32:  141 tensors
llama_model_loader: - type q5_K:  169 tensors
llama_model_loader: - type q6_K:   29 tensors
print_info: file format = GGUF V3 (latest)
print_info: file type   = Q5_K - Medium
print_info: file size   = 1.19 GiB (5.76 BPW)
init_tokenizer: initializing tokenizer for type 2
load: 0 unused tokens
load: control token: 151661 '<|fim_suffix|>' is not marked as EOG
load: control token: 151649 '<|box_end|>' is not marked as EOG
load: control token: 151647 '<|object_ref_end|>' is not marked as EOG
load: control token: 151654 '<|vision_pad|>' is not marked as EOG
load: control token: 151659 '<|fim_prefix|>' is not marked as EOG
load: control token: 151648 '<|box_start|>' is not marked as EOG
load: control token: 151644 '<|im_start|>' is not marked as EOG
load: control token: 151646 '<|object_ref_start|>' is not marked as EOG
load: control token: 151650 '<|quad_start|>' is not marked as EOG
load: control token: 151651 '<|quad_end|>' is not marked as EOG
load: control token: 151652 '<|vision_start|>' is not marked as EOG
load: control token: 151653 '<|vision_end|>' is not marked as EOG
load: control token: 151655 '<|image_pad|>' is not marked as EOG
load: control token: 151656 '<|video_pad|>' is not marked as EOG
load: control token: 151660 '<|fim_middle|>' is not marked as EOG
load: printing all EOG tokens:
load:   - 151643 ('<|endoftext|>')
load:   - 151645 ('<|im_end|>')
load:   - 151662 ('<|fim_pad|>')
load:   - 151663 ('<|repo_name|>')
load:   - 151664 ('<|file_sep|>')
load: special tokens cache size = 22
load: token to piece cache size = 0.9310 MB
print_info: arch             = qwen2
print_info: vocab_only       = 0
print_info: no_alloc         = 0
print_info: n_ctx_train      = 32768
print_info: n_embd           = 1536
print_info: n_embd_inp       = 1536
print_info: n_layer          = 28
print_info: n_head           = 12
print_info: n_head_kv        = 2
print_info: n_rot            = 128
print_info: n_swa            = 0
print_info: is_swa_any       = 0
print_info: n_embd_head_k    = 128
print_info: n_embd_head_v    = 128
print_info: n_gqa            = 6
print_info: n_embd_k_gqa     = 256
print_info: n_embd_v_gqa     = 256
print_info: f_norm_eps       = 0.0e+00
print_info: f_norm_rms_eps   = 1.0e-06
print_info: f_clamp_kqv      = 0.0e+00
print_info: f_max_alibi_bias = 0.0e+00
print_info: f_logit_scale    = 0.0e+00
print_info: f_attn_scale     = 0.0e+00
print_info: n_ff             = 8960
print_info: n_expert         = 0
print_info: n_expert_used    = 0
print_info: n_expert_groups  = 0
print_info: n_group_used     = 0
print_info: causal attn      = 1
print_info: pooling type     = -1
print_info: rope type        = 2
print_info: rope scaling     = linear
print_info: freq_base_train  = 1000000.0
print_info: freq_scale_train = 1
print_info: n_ctx_orig_yarn  = 32768
print_info: rope_yarn_log_mul= 0.0000
print_info: rope_finetuned   = unknown
print_info: model type       = 1.5B
print_info: model params     = 1.78 B
print_info: general.name     = Qwen2.5 Coder 1.5B Instruct GGUF
print_info: vocab type       = BPE
print_info: n_vocab          = 151936
print_info: n_merges         = 151387
print_info: BOS token        = 151643 '<|endoftext|>'
print_info: EOS token        = 151645 '<|im_end|>'
print_info: EOT token        = 151645 '<|im_end|>'
print_info: PAD token        = 151643 '<|endoftext|>'
print_info: LF token         = 198 '─è'
print_info: FIM PRE token    = 151659 '<|fim_prefix|>'
print_info: FIM SUF token    = 151661 '<|fim_suffix|>'
print_info: FIM MID token    = 151660 '<|fim_middle|>'
print_info: FIM PAD token    = 151662 '<|fim_pad|>'
print_info: FIM REP token    = 151663 '<|repo_name|>'
print_info: FIM SEP token    = 151664 '<|file_sep|>'
print_info: EOG token        = 151643 '<|endoftext|>'
print_info: EOG token        = 151645 '<|im_end|>'
print_info: EOG token        = 151662 '<|fim_pad|>'
print_info: EOG token        = 151663 '<|repo_name|>'
print_info: EOG token        = 151664 '<|file_sep|>'
print_info: max token length = 256
load_tensors: loading model tensors, this can take a while... (mmap = false, direct_io = true)
load_tensors: layer   0 assigned to device CPU, is_swa = 0
load_tensors: layer   1 assigned to device CPU, is_swa = 0
load_tensors: layer   2 assigned to device CPU, is_swa = 0
load_tensors: layer   3 assigned to device CPU, is_swa = 0
load_tensors: layer   4 assigned to device CPU, is_swa = 0
load_tensors: layer   5 assigned to device CPU, is_swa = 0
load_tensors: layer   6 assigned to device CPU, is_swa = 0
load_tensors: layer   7 assigned to device CPU, is_swa = 0
load_tensors: layer   8 assigned to device CPU, is_swa = 0
load_tensors: layer   9 assigned to device CPU, is_swa = 0
load_tensors: layer  10 assigned to device CPU, is_swa = 0
load_tensors: layer  11 assigned to device CPU, is_swa = 0
load_tensors: layer  12 assigned to device CPU, is_swa = 0
load_tensors: layer  13 assigned to device CPU, is_swa = 0
load_tensors: layer  14 assigned to device CPU, is_swa = 0
load_tensors: layer  15 assigned to device CPU, is_swa = 0
load_tensors: layer  16 assigned to device CPU, is_swa = 0
load_tensors: layer  17 assigned to device CPU, is_swa = 0
load_tensors: layer  18 assigned to device CPU, is_swa = 0
load_tensors: layer  19 assigned to device CPU, is_swa = 0
load_tensors: layer  20 assigned to device CPU, is_swa = 0
load_tensors: layer  21 assigned to device CPU, is_swa = 0
load_tensors: layer  22 assigned to device CPU, is_swa = 0
load_tensors: layer  23 assigned to device CPU, is_swa = 0
load_tensors: layer  24 assigned to device CPU, is_swa = 0
load_tensors: layer  25 assigned to device CPU, is_swa = 0
load_tensors: layer  26 assigned to device CPU, is_swa = 0
load_tensors: layer  27 assigned to device CPU, is_swa = 0
load_tensors: layer  28 assigned to device CPU, is_swa = 0
create_tensor: loading tensor token_embd.weight
create_tensor: loading tensor output_norm.weight
create_tensor: loading tensor output.weight
create_tensor: loading tensor blk.0.attn_norm.weight
create_tensor: loading tensor blk.0.attn_q.weight
create_tensor: loading tensor blk.0.attn_k.weight
create_tensor: loading tensor blk.0.attn_v.weight
create_tensor: loading tensor blk.0.attn_output.weight
create_tensor: loading tensor blk.0.attn_q.bias
create_tensor: loading tensor blk.0.attn_k.bias
create_tensor: loading tensor blk.0.attn_v.bias
create_tensor: loading tensor blk.0.ffn_norm.weight
create_tensor: loading tensor blk.0.ffn_gate.weight
create_tensor: loading tensor blk.0.ffn_down.weight
create_tensor: loading tensor blk.0.ffn_up.weight
create_tensor: loading tensor blk.1.attn_norm.weight
create_tensor: loading tensor blk.1.attn_q.weight
create_tensor: loading tensor blk.1.attn_k.weight
create_tensor: loading tensor blk.1.attn_v.weight
create_tensor: loading tensor blk.1.attn_output.weight
create_tensor: loading tensor blk.1.attn_q.bias
create_tensor: loading tensor blk.1.attn_k.bias
create_tensor: loading tensor blk.1.attn_v.bias
create_tensor: loading tensor blk.1.ffn_norm.weight
create_tensor: loading tensor blk.1.ffn_gate.weight
create_tensor: loading tensor blk.1.ffn_down.weight
create_tensor: loading tensor blk.1.ffn_up.weight
create_tensor: loading tensor blk.2.attn_norm.weight
create_tensor: loading tensor blk.2.attn_q.weight
create_tensor: loading tensor blk.2.attn_k.weight
create_tensor: loading tensor blk.2.attn_v.weight
create_tensor: loading tensor blk.2.attn_output.weight
create_tensor: loading tensor blk.2.attn_q.bias
create_tensor: loading tensor blk.2.attn_k.bias
create_tensor: loading tensor blk.2.attn_v.bias
create_tensor: loading tensor blk.2.ffn_norm.weight
create_tensor: loading tensor blk.2.ffn_gate.weight
create_tensor: loading tensor blk.2.ffn_down.weight
create_tensor: loading tensor blk.2.ffn_up.weight
create_tensor: loading tensor blk.3.attn_norm.weight
create_tensor: loading tensor blk.3.attn_q.weight
create_tensor: loading tensor blk.3.attn_k.weight
create_tensor: loading tensor blk.3.attn_v.weight
create_tensor: loading tensor blk.3.attn_output.weight
create_tensor: loading tensor blk.3.attn_q.bias
create_tensor: loading tensor blk.3.attn_k.bias
create_tensor: loading tensor blk.3.attn_v.bias
create_tensor: loading tensor blk.3.ffn_norm.weight
create_tensor: loading tensor blk.3.ffn_gate.weight
create_tensor: loading tensor blk.3.ffn_down.weight
create_tensor: loading tensor blk.3.ffn_up.weight
create_tensor: loading tensor blk.4.attn_norm.weight
create_tensor: loading tensor blk.4.attn_q.weight
create_tensor: loading tensor blk.4.attn_k.weight
create_tensor: loading tensor blk.4.attn_v.weight
create_tensor: loading tensor blk.4.attn_output.weight
create_tensor: loading tensor blk.4.attn_q.bias
create_tensor: loading tensor blk.4.attn_k.bias
create_tensor: loading tensor blk.4.attn_v.bias
create_tensor: loading tensor blk.4.ffn_norm.weight
create_tensor: loading tensor blk.4.ffn_gate.weight
create_tensor: loading tensor blk.4.ffn_down.weight
create_tensor: loading tensor blk.4.ffn_up.weight
create_tensor: loading tensor blk.5.attn_norm.weight
create_tensor: loading tensor blk.5.attn_q.weight
create_tensor: loading tensor blk.5.attn_k.weight
create_tensor: loading tensor blk.5.attn_v.weight
create_tensor: loading tensor blk.5.attn_output.weight
create_tensor: loading tensor blk.5.attn_q.bias
create_tensor: loading tensor blk.5.attn_k.bias
create_tensor: loading tensor blk.5.attn_v.bias
create_tensor: loading tensor blk.5.ffn_norm.weight
create_tensor: loading tensor blk.5.ffn_gate.weight
create_tensor: loading tensor blk.5.ffn_down.weight
create_tensor: loading tensor blk.5.ffn_up.weight
create_tensor: loading tensor blk.6.attn_norm.weight
create_tensor: loading tensor blk.6.attn_q.weight
create_tensor: loading tensor blk.6.attn_k.weight
create_tensor: loading tensor blk.6.attn_v.weight
create_tensor: loading tensor blk.6.attn_output.weight
create_tensor: loading tensor blk.6.attn_q.bias
create_tensor: loading tensor blk.6.attn_k.bias
create_tensor: loading tensor blk.6.attn_v.bias
create_tensor: loading tensor blk.6.ffn_norm.weight
create_tensor: loading tensor blk.6.ffn_gate.weight
create_tensor: loading tensor blk.6.ffn_down.weight
create_tensor: loading tensor blk.6.ffn_up.weight
create_tensor: loading tensor blk.7.attn_norm.weight
create_tensor: loading tensor blk.7.attn_q.weight
create_tensor: loading tensor blk.7.attn_k.weight
create_tensor: loading tensor blk.7.attn_v.weight
create_tensor: loading tensor blk.7.attn_output.weight
create_tensor: loading tensor blk.7.attn_q.bias
create_tensor: loading tensor blk.7.attn_k.bias
create_tensor: loading tensor blk.7.attn_v.bias
create_tensor: loading tensor blk.7.ffn_norm.weight
create_tensor: loading tensor blk.7.ffn_gate.weight
create_tensor: loading tensor blk.7.ffn_down.weight
create_tensor: loading tensor blk.7.ffn_up.weight
create_tensor: loading tensor blk.8.attn_norm.weight
create_tensor: loading tensor blk.8.attn_q.weight
create_tensor: loading tensor blk.8.attn_k.weight
create_tensor: loading tensor blk.8.attn_v.weight
create_tensor: loading tensor blk.8.attn_output.weight
create_tensor: loading tensor blk.8.attn_q.bias
create_tensor: loading tensor blk.8.attn_k.bias
create_tensor: loading tensor blk.8.attn_v.bias
create_tensor: loading tensor blk.8.ffn_norm.weight
create_tensor: loading tensor blk.8.ffn_gate.weight
create_tensor: loading tensor blk.8.ffn_down.weight
create_tensor: loading tensor blk.8.ffn_up.weight
create_tensor: loading tensor blk.9.attn_norm.weight
create_tensor: loading tensor blk.9.attn_q.weight
create_tensor: loading tensor blk.9.attn_k.weight
create_tensor: loading tensor blk.9.attn_v.weight
create_tensor: loading tensor blk.9.attn_output.weight
create_tensor: loading tensor blk.9.attn_q.bias
create_tensor: loading tensor blk.9.attn_k.bias
create_tensor: loading tensor blk.9.attn_v.bias
create_tensor: loading tensor blk.9.ffn_norm.weight
create_tensor: loading tensor blk.9.ffn_gate.weight
create_tensor: loading tensor blk.9.ffn_down.weight
create_tensor: loading tensor blk.9.ffn_up.weight
create_tensor: loading tensor blk.10.attn_norm.weight
create_tensor: loading tensor blk.10.attn_q.weight
create_tensor: loading tensor blk.10.attn_k.weight
create_tensor: loading tensor blk.10.attn_v.weight
create_tensor: loading tensor blk.10.attn_output.weight
create_tensor: loading tensor blk.10.attn_q.bias
create_tensor: loading tensor blk.10.attn_k.bias
create_tensor: loading tensor blk.10.attn_v.bias
create_tensor: loading tensor blk.10.ffn_norm.weight
create_tensor: loading tensor blk.10.ffn_gate.weight
create_tensor: loading tensor blk.10.ffn_down.weight
create_tensor: loading tensor blk.10.ffn_up.weight
create_tensor: loading tensor blk.11.attn_norm.weight
create_tensor: loading tensor blk.11.attn_q.weight
create_tensor: loading tensor blk.11.attn_k.weight
create_tensor: loading tensor blk.11.attn_v.weight
create_tensor: loading tensor blk.11.attn_output.weight
create_tensor: loading tensor blk.11.attn_q.bias
create_tensor: loading tensor blk.11.attn_k.bias
create_tensor: loading tensor blk.11.attn_v.bias
create_tensor: loading tensor blk.11.ffn_norm.weight
create_tensor: loading tensor blk.11.ffn_gate.weight
create_tensor: loading tensor blk.11.ffn_down.weight
create_tensor: loading tensor blk.11.ffn_up.weight
create_tensor: loading tensor blk.12.attn_norm.weight
create_tensor: loading tensor blk.12.attn_q.weight
create_tensor: loading tensor blk.12.attn_k.weight
create_tensor: loading tensor blk.12.attn_v.weight
create_tensor: loading tensor blk.12.attn_output.weight
create_tensor: loading tensor blk.12.attn_q.bias
create_tensor: loading tensor blk.12.attn_k.bias
create_tensor: loading tensor blk.12.attn_v.bias
create_tensor: loading tensor blk.12.ffn_norm.weight
create_tensor: loading tensor blk.12.ffn_gate.weight
create_tensor: loading tensor blk.12.ffn_down.weight
create_tensor: loading tensor blk.12.ffn_up.weight
create_tensor: loading tensor blk.13.attn_norm.weight
create_tensor: loading tensor blk.13.attn_q.weight
create_tensor: loading tensor blk.13.attn_k.weight
create_tensor: loading tensor blk.13.attn_v.weight
create_tensor: loading tensor blk.13.attn_output.weight
create_tensor: loading tensor blk.13.attn_q.bias
create_tensor: loading tensor blk.13.attn_k.bias
create_tensor: loading tensor blk.13.attn_v.bias
create_tensor: loading tensor blk.13.ffn_norm.weight
create_tensor: loading tensor blk.13.ffn_gate.weight
create_tensor: loading tensor blk.13.ffn_down.weight
create_tensor: loading tensor blk.13.ffn_up.weight
create_tensor: loading tensor blk.14.attn_norm.weight
create_tensor: loading tensor blk.14.attn_q.weight
create_tensor: loading tensor blk.14.attn_k.weight
create_tensor: loading tensor blk.14.attn_v.weight
create_tensor: loading tensor blk.14.attn_output.weight
create_tensor: loading tensor blk.14.attn_q.bias
create_tensor: loading tensor blk.14.attn_k.bias
create_tensor: loading tensor blk.14.attn_v.bias
create_tensor: loading tensor blk.14.ffn_norm.weight
create_tensor: loading tensor blk.14.ffn_gate.weight
create_tensor: loading tensor blk.14.ffn_down.weight
create_tensor: loading tensor blk.14.ffn_up.weight
create_tensor: loading tensor blk.15.attn_norm.weight
create_tensor: loading tensor blk.15.attn_q.weight
create_tensor: loading tensor blk.15.attn_k.weight
create_tensor: loading tensor blk.15.attn_v.weight
create_tensor: loading tensor blk.15.attn_output.weight
create_tensor: loading tensor blk.15.attn_q.bias
create_tensor: loading tensor blk.15.attn_k.bias
create_tensor: loading tensor blk.15.attn_v.bias
create_tensor: loading tensor blk.15.ffn_norm.weight
create_tensor: loading tensor blk.15.ffn_gate.weight
create_tensor: loading tensor blk.15.ffn_down.weight
create_tensor: loading tensor blk.15.ffn_up.weight
create_tensor: loading tensor blk.16.attn_norm.weight
create_tensor: loading tensor blk.16.attn_q.weight
create_tensor: loading tensor blk.16.attn_k.weight
create_tensor: loading tensor blk.16.attn_v.weight
create_tensor: loading tensor blk.16.attn_output.weight
create_tensor: loading tensor blk.16.attn_q.bias
create_tensor: loading tensor blk.16.attn_k.bias
create_tensor: loading tensor blk.16.attn_v.bias
create_tensor: loading tensor blk.16.ffn_norm.weight
create_tensor: loading tensor blk.16.ffn_gate.weight
create_tensor: loading tensor blk.16.ffn_down.weight
create_tensor: loading tensor blk.16.ffn_up.weight
create_tensor: loading tensor blk.17.attn_norm.weight
create_tensor: loading tensor blk.17.attn_q.weight
create_tensor: loading tensor blk.17.attn_k.weight
create_tensor: loading tensor blk.17.attn_v.weight
create_tensor: loading tensor blk.17.attn_output.weight
create_tensor: loading tensor blk.17.attn_q.bias
create_tensor: loading tensor blk.17.attn_k.bias
create_tensor: loading tensor blk.17.attn_v.bias
create_tensor: loading tensor blk.17.ffn_norm.weight
create_tensor: loading tensor blk.17.ffn_gate.weight
create_tensor: loading tensor blk.17.ffn_down.weight
create_tensor: loading tensor blk.17.ffn_up.weight
create_tensor: loading tensor blk.18.attn_norm.weight
create_tensor: loading tensor blk.18.attn_q.weight
create_tensor: loading tensor blk.18.attn_k.weight
create_tensor: loading tensor blk.18.attn_v.weight
create_tensor: loading tensor blk.18.attn_output.weight
create_tensor: loading tensor blk.18.attn_q.bias
create_tensor: loading tensor blk.18.attn_k.bias
create_tensor: loading tensor blk.18.attn_v.bias
create_tensor: loading tensor blk.18.ffn_norm.weight
create_tensor: loading tensor blk.18.ffn_gate.weight
create_tensor: loading tensor blk.18.ffn_down.weight
create_tensor: loading tensor blk.18.ffn_up.weight
create_tensor: loading tensor blk.19.attn_norm.weight
create_tensor: loading tensor blk.19.attn_q.weight
create_tensor: loading tensor blk.19.attn_k.weight
create_tensor: loading tensor blk.19.attn_v.weight
create_tensor: loading tensor blk.19.attn_output.weight
create_tensor: loading tensor blk.19.attn_q.bias
create_tensor: loading tensor blk.19.attn_k.bias
create_tensor: loading tensor blk.19.attn_v.bias
create_tensor: loading tensor blk.19.ffn_norm.weight
create_tensor: loading tensor blk.19.ffn_gate.weight
create_tensor: loading tensor blk.19.ffn_down.weight
create_tensor: loading tensor blk.19.ffn_up.weight
create_tensor: loading tensor blk.20.attn_norm.weight
create_tensor: loading tensor blk.20.attn_q.weight
create_tensor: loading tensor blk.20.attn_k.weight
create_tensor: loading tensor blk.20.attn_v.weight
create_tensor: loading tensor blk.20.attn_output.weight
create_tensor: loading tensor blk.20.attn_q.bias
create_tensor: loading tensor blk.20.attn_k.bias
create_tensor: loading tensor blk.20.attn_v.bias
create_tensor: loading tensor blk.20.ffn_norm.weight
create_tensor: loading tensor blk.20.ffn_gate.weight
create_tensor: loading tensor blk.20.ffn_down.weight
create_tensor: loading tensor blk.20.ffn_up.weight
create_tensor: loading tensor blk.21.attn_norm.weight
create_tensor: loading tensor blk.21.attn_q.weight
create_tensor: loading tensor blk.21.attn_k.weight
create_tensor: loading tensor blk.21.attn_v.weight
create_tensor: loading tensor blk.21.attn_output.weight
create_tensor: loading tensor blk.21.attn_q.bias
create_tensor: loading tensor blk.21.attn_k.bias
create_tensor: loading tensor blk.21.attn_v.bias
create_tensor: loading tensor blk.21.ffn_norm.weight
create_tensor: loading tensor blk.21.ffn_gate.weight
create_tensor: loading tensor blk.21.ffn_down.weight
create_tensor: loading tensor blk.21.ffn_up.weight
create_tensor: loading tensor blk.22.attn_norm.weight
create_tensor: loading tensor blk.22.attn_q.weight
create_tensor: loading tensor blk.22.attn_k.weight
create_tensor: loading tensor blk.22.attn_v.weight
create_tensor: loading tensor blk.22.attn_output.weight
create_tensor: loading tensor blk.22.attn_q.bias
create_tensor: loading tensor blk.22.attn_k.bias
create_tensor: loading tensor blk.22.attn_v.bias
create_tensor: loading tensor blk.22.ffn_norm.weight
create_tensor: loading tensor blk.22.ffn_gate.weight
create_tensor: loading tensor blk.22.ffn_down.weight
create_tensor: loading tensor blk.22.ffn_up.weight
create_tensor: loading tensor blk.23.attn_norm.weight
create_tensor: loading tensor blk.23.attn_q.weight
create_tensor: loading tensor blk.23.attn_k.weight
create_tensor: loading tensor blk.23.attn_v.weight
create_tensor: loading tensor blk.23.attn_output.weight
create_tensor: loading tensor blk.23.attn_q.bias
create_tensor: loading tensor blk.23.attn_k.bias
create_tensor: loading tensor blk.23.attn_v.bias
create_tensor: loading tensor blk.23.ffn_norm.weight
create_tensor: loading tensor blk.23.ffn_gate.weight
create_tensor: loading tensor blk.23.ffn_down.weight
create_tensor: loading tensor blk.23.ffn_up.weight
create_tensor: loading tensor blk.24.attn_norm.weight
create_tensor: loading tensor blk.24.attn_q.weight
create_tensor: loading tensor blk.24.attn_k.weight
create_tensor: loading tensor blk.24.attn_v.weight
create_tensor: loading tensor blk.24.attn_output.weight
create_tensor: loading tensor blk.24.attn_q.bias
create_tensor: loading tensor blk.24.attn_k.bias
create_tensor: loading tensor blk.24.attn_v.bias
create_tensor: loading tensor blk.24.ffn_norm.weight
create_tensor: loading tensor blk.24.ffn_gate.weight
create_tensor: loading tensor blk.24.ffn_down.weight
create_tensor: loading tensor blk.24.ffn_up.weight
create_tensor: loading tensor blk.25.attn_norm.weight
create_tensor: loading tensor blk.25.attn_q.weight
create_tensor: loading tensor blk.25.attn_k.weight
create_tensor: loading tensor blk.25.attn_v.weight
create_tensor: loading tensor blk.25.attn_output.weight
create_tensor: loading tensor blk.25.attn_q.bias
create_tensor: loading tensor blk.25.attn_k.bias
create_tensor: loading tensor blk.25.attn_v.bias
create_tensor: loading tensor blk.25.ffn_norm.weight
create_tensor: loading tensor blk.25.ffn_gate.weight
create_tensor: loading tensor blk.25.ffn_down.weight
create_tensor: loading tensor blk.25.ffn_up.weight
create_tensor: loading tensor blk.26.attn_norm.weight
create_tensor: loading tensor blk.26.attn_q.weight
create_tensor: loading tensor blk.26.attn_k.weight
create_tensor: loading tensor blk.26.attn_v.weight
create_tensor: loading tensor blk.26.attn_output.weight
create_tensor: loading tensor blk.26.attn_q.bias
create_tensor: loading tensor blk.26.attn_k.bias
create_tensor: loading tensor blk.26.attn_v.bias
create_tensor: loading tensor blk.26.ffn_norm.weight
create_tensor: loading tensor blk.26.ffn_gate.weight
create_tensor: loading tensor blk.26.ffn_down.weight
create_tensor: loading tensor blk.26.ffn_up.weight
create_tensor: loading tensor blk.27.attn_norm.weight
create_tensor: loading tensor blk.27.attn_q.weight
create_tensor: loading tensor blk.27.attn_k.weight
create_tensor: loading tensor blk.27.attn_v.weight
create_tensor: loading tensor blk.27.attn_output.weight
create_tensor: loading tensor blk.27.attn_q.bias
create_tensor: loading tensor blk.27.attn_k.bias
create_tensor: loading tensor blk.27.attn_v.bias
create_tensor: loading tensor blk.27.ffn_norm.weight
create_tensor: loading tensor blk.27.ffn_gate.weight
create_tensor: loading tensor blk.27.ffn_down.weight
create_tensor: loading tensor blk.27.ffn_up.weight
load_tensors: tensor 'token_embd.weight' (q5_K) (and 338 others) cannot be used with preferred buffer type CPU_REPACK, using CPU instead
warning: failed to VirtualLock 1279545344-byte buffer (after previously locking 0 bytes): Invalid access to memory location.

load_tensors:          CPU model buffer size =  1220.27 MiB
load_all_data: no device found for buffer type CPU for async uploads
...........................................................................
[2026-02-01T15:42:59Z DEBUG llama_cpp_2::model] Loaded model path="C:\\Program Files\\RustServer\\Models\\qwen2.5-coder-1.5b-instruct-q5_k_m.gguf"
[2026-02-01T15:42:59Z INFO  sfcore_ai_engine::llama_engine] model loaded in 8203 ms
[2026-02-01T15:42:59Z INFO  sfcore_ai_server] HTTP-SSE listening on 0.0.0.0:8080
[2026-02-01T15:43:07Z DEBUG tower_http::trace::make_span] request; method=POST uri=/v1/inference version=HTTP/1.1
[2026-02-01T15:43:07Z DEBUG tower_http::trace::on_request] started processing request
[2026-02-01T15:43:07Z INFO  sfcore_ai_server::http_handler] Inference request - stream: true
[2026-02-01T15:43:07Z INFO  tracing::span] apply_chat_template;
[2026-02-01T15:43:07Z DEBUG tower_http::trace::on_response] finished processing request latency=1 ms status=200
[2026-02-01T15:43:07Z INFO  sfcore_ai_engine::llama_engine] === Generation Started ===
llama_context: constructing llama_context
llama_context: n_seq_max     = 1
llama_context: n_ctx         = 8192
llama_context: n_ctx_seq     = 8192
llama_context: n_batch       = 512
llama_context: n_ubatch      = 256
llama_context: causal_attn   = 1
llama_context: flash_attn    = auto
llama_context: kv_unified    = false
llama_context: freq_base     = 1000000.0
llama_context: freq_scale    = 1
llama_context: n_ctx_seq (8192) < n_ctx_train (32768) -- the full capacity of the model will not be utilized
set_abort_callback: call
llama_context:        CPU  output buffer size =     0.58 MiB
llama_kv_cache: layer   0: dev = CPU
llama_kv_cache: layer   1: dev = CPU
llama_kv_cache: layer   2: dev = CPU
llama_kv_cache: layer   3: dev = CPU
llama_kv_cache: layer   4: dev = CPU
llama_kv_cache: layer   5: dev = CPU
llama_kv_cache: layer   6: dev = CPU
llama_kv_cache: layer   7: dev = CPU
llama_kv_cache: layer   8: dev = CPU
llama_kv_cache: layer   9: dev = CPU
llama_kv_cache: layer  10: dev = CPU
llama_kv_cache: layer  11: dev = CPU
llama_kv_cache: layer  12: dev = CPU
llama_kv_cache: layer  13: dev = CPU
llama_kv_cache: layer  14: dev = CPU
llama_kv_cache: layer  15: dev = CPU
llama_kv_cache: layer  16: dev = CPU
llama_kv_cache: layer  17: dev = CPU
llama_kv_cache: layer  18: dev = CPU
llama_kv_cache: layer  19: dev = CPU
llama_kv_cache: layer  20: dev = CPU
llama_kv_cache: layer  21: dev = CPU
llama_kv_cache: layer  22: dev = CPU
llama_kv_cache: layer  23: dev = CPU
llama_kv_cache: layer  24: dev = CPU
llama_kv_cache: layer  25: dev = CPU
llama_kv_cache: layer  26: dev = CPU
llama_kv_cache: layer  27: dev = CPU
llama_kv_cache:        CPU KV buffer size =   224.00 MiB
llama_kv_cache: size =  224.00 MiB (  8192 cells,  28 layers,  1/1 seqs), K (f16):  112.00 MiB, V (f16):  112.00 MiB
llama_context: enumerating backends
llama_context: backend_ptrs.size() = 1
llama_context: max_nodes = 2712
llama_context: reserving full memory module
llama_context: worst-case: n_tokens = 256, n_seqs = 1, n_outputs = 1
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
llama_context: Flash Attention was auto, set to enabled
graph_reserve: reserving a graph for ubatch with n_tokens =  256, n_seqs =  1, n_outputs =  256
graph_reserve: reserving a graph for ubatch with n_tokens =    1, n_seqs =  1, n_outputs =    1
graph_reserve: reserving a graph for ubatch with n_tokens =  256, n_seqs =  1, n_outputs =  256
llama_context:        CPU compute buffer size =   162.38 MiB
llama_context: graph nodes  = 959
llama_context: graph splits = 1
[2026-02-01T15:43:13Z INFO  sfcore_ai_engine::llama_engine] Prompt tokens: 33
[2026-02-01T15:43:13Z INFO  sfcore_ai_engine::llama_engine] Prefill: 33 tokens in chunks of 512
[2026-02-01T15:43:13Z DEBUG sfcore_ai_engine::llama_engine] Processing chunk 0-33 (size=33)
