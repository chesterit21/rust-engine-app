/// Token estimation for Indonesian/English mixed content
/// Improved accuracy with error handling and performance optimization

use crate::database::DocumentChunk;

/// Estimate tokens from text using improved word-based heuristic
/// Handles edge cases: empty strings, special characters, mixed languages
pub fn estimate_tokens(text: &str) -> usize {
    // Early return for empty text
    if text.is_empty() {
        return 0;
    }
    
    // Count words (whitespace-separated)
    let words = text.split_whitespace().count();
    
    // Handle edge case: text with no words (all whitespace or special chars)
    if words == 0 {
        // Fallback to character-based estimation
        return (text.len() / 4).max(1);
    }
    
    // Indonesian/English mixed: 1.3 tokens per word
    // Add small overhead for formatting (5 tokens)
    let base_tokens = (words as f64 * 1.3).ceil() as usize;
    
    // Add overhead, capped at reasonable limit
    (base_tokens + 5).min(500_000) // Prevent overflow for huge texts
}

/// Estimate tokens for multiple chunks with safety checks
pub fn estimate_chunks_tokens(chunks: &[DocumentChunk]) -> usize {
    // Handle empty input
    if chunks.is_empty() {
        return 0;
    }
    
    chunks.iter()
        .map(|chunk| {
            // Estimate per chunk with individual cap
            let tokens = estimate_tokens(&chunk.content);
            tokens.min(50_000) // Cap individual chunk at 50K tokens
        })
        .sum::<usize>()
        .min(1_000_000) // Cap total at 1M tokens to prevent overflow
}

/// Check if adding text would exceed limit (with safety)
pub fn would_exceed_limit(
    current_tokens: usize,
    new_text: &str,
    max_tokens: usize,
) -> bool {
    // Safety check: prevent overflow
    if current_tokens >= usize::MAX - 10_000 {
        return true; // Already at limit
    }
    
    let new_tokens = estimate_tokens(new_text);
    
    // Safe addition with overflow check
    match current_tokens.checked_add(new_tokens) {
        Some(total) => total > max_tokens,
        None => true, // Overflow occurred, definitely exceeds limit
    }
}

/// Estimate tokens for system prompt + context (convenience function)
pub fn estimate_system_tokens(system_prompt: &str, context: &str) -> usize {
    let prompt_tokens = estimate_tokens(system_prompt);
    let context_tokens = estimate_tokens(context);
    
    // Safe addition with overflow protection
    prompt_tokens.saturating_add(context_tokens).saturating_add(10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        let text = "Ini adalah dokumen test yang berisi informasi";
        let tokens = estimate_tokens(text);
        // 7 words * 1.3 + 5 = ~14
        assert!(tokens >= 10 && tokens <= 20, "Got: {}", tokens);
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_whitespace_only() {
        let text = "   \n\t  ";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0, "Should handle whitespace-only text");
    }

    #[test]
    fn test_would_exceed() {
        let current = 1000;
        let text = "word ".repeat(500); // ~650 tokens
        
        assert!(would_exceed_limit(current, &text, 1500));
        assert!(!would_exceed_limit(current, &text, 2000));
    }

    #[test]
    fn test_overflow_protection() {
        let huge = usize::MAX - 100;
        assert!(would_exceed_limit(huge, "test", 1000));
    }

    #[test]
    fn test_chunks_tokens_empty() {
        let chunks: Vec<crate::database::DocumentChunk> = vec![];
        assert_eq!(estimate_chunks_tokens(&chunks), 0);
    }
}