//! SFCore AI Engine - High-performance LLM inference using llama.cpp
//!
//! This crate provides Rust bindings to llama.cpp for fast CPU inference.

// Re-export everything from llama_engine
mod llama_engine;
pub use llama_engine::{ChatMessage, GenerationResult, LlamaCppEngine, LlamaCppOptions};

// Metrics module for observability
pub mod metrics {
    use sysinfo::System;

    #[derive(Debug, Clone, Copy)]
    pub struct RuntimeMetrics {
        pub process_rss_mb: f64,
        pub total_mem_mb: f64,
        pub cpu_usage_percent: f32,
    }

    impl RuntimeMetrics {
        pub fn capture() -> Self {
            let mut sys = System::new_all();
            sys.refresh_all();
            let pid = sysinfo::get_current_pid().ok();
            let (rss_bytes, cpu) = if let Some(p) = pid.and_then(|pid| sys.process(pid)) {
                (p.memory(), p.cpu_usage())
            } else {
                (0, 0.0)
            };
            let total_bytes = sys.total_memory();
            Self {
                process_rss_mb: (rss_bytes as f64) / (1024.0 * 1024.0),
                total_mem_mb: (total_bytes as f64) / (1024.0 * 1024.0),
                cpu_usage_percent: cpu,
            }
        }
    }
}
