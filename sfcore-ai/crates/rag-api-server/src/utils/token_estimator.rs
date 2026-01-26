// File: src/utils/token_estimator.rs (CREATE NEW)

/// Token estimation for Indonesian/English mixed content
/// More accurate than simple char/4 approximation

use crate::database::DocumentChunk;

/// Estimate tokens from text using word-based heuristic
/// Rule: Indonesian/English avg ~1.3 tokens per word
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    
    // Count words (more accurate than char count)
    let words = text.split_whitespace().count();
    
    // Indonesian/English mixed: 1.3 tokens per word
    // Add small overhead for formatting
    ((words as f64 * 1.3) + 5.0).ceil() as usize
}

/// Estimate tokens for multiple chunks
pub fn estimate_chunks_tokens(chunks: &[DocumentChunk]) -> usize {
    chunks.iter()
        .map(|chunk| estimate_tokens(&chunk.content))
        .sum()
}

/// Check if adding text would exceed limit
pub fn would_exceed_limit(
    current_tokens: usize,
    new_text: &str,
    max_tokens: usize,
) -> bool {
    let new_tokens = estimate_tokens(new_text);
    current_tokens + new_tokens > max_tokens
}

/// Estimate tokens for system prompt + context
pub fn estimate_system_tokens(system_prompt: &str, context: &str) -> usize {
    estimate_tokens(system_prompt) + estimate_tokens(context) + 10 // overhead
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        // "Ini adalah dokumen test yang berisi informasi" = 7 words
        let text = "Ini adalah dokumen test yang berisi informasi";
        let tokens = estimate_tokens(text);
        // 7 * 1.3 + 5 = 14.1 ≈ 15
        assert!(tokens >= 13 && tokens <= 16);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_would_exceed() {
        let current = 1000;
        // ~500 words = 650 tokens
        let text = "word ".repeat(500);
        assert!(would_exceed_limit(current, &text, 1500));
        assert!(!would_exceed_limit(current, &text, 2000));
    }
}