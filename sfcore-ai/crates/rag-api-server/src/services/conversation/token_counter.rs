use unicode_segmentation::UnicodeSegmentation;
use rand::Rng;
use crate::models::chat::ChatMessage;
use super::types::TokenCount;

pub struct TokenCounter;

impl TokenCounter {
    pub fn count_text(text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        let char_count = text.graphemes(true).count();
        let mut rng = rand::thread_rng();
        // Simple estimation: avg 2.5 chars per token
        let chars_per_token = if rng.gen_bool(0.5) { 2 } else { 3 };
        
        ((char_count + chars_per_token - 1) / chars_per_token).max(1)
    }

    pub fn count_messages(messages: &[ChatMessage]) -> usize {
        messages.iter()
            .map(|msg| msg.estimate_tokens())
            .sum()
    }

    pub fn count_payload(
        system_context: &str,
        messages: &[ChatMessage],
        current_message: &str,
    ) -> TokenCount {
        let system_tokens = Self::count_text(system_context);
        let history_tokens = Self::count_messages(messages);
        let current_message_tokens = Self::count_text(current_message);

        TokenCount {
            total: system_tokens + history_tokens + current_message_tokens,
            system_tokens,
            history_tokens,
            current_message_tokens,
        }
    }

    pub fn estimate_total(
        system_approx: usize,
        messages: &[ChatMessage],
        current_message: &str,
    ) -> usize {
        system_approx 
            + Self::count_messages(messages) 
            + Self::count_text(current_message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_text() {
        let text = "Hello world";
        let tokens = TokenCounter::count_text(text);
        assert!(tokens >= 3 && tokens <= 6);
    }

    #[test]
    fn test_count_messages() {
        let messages = vec![
            ChatMessage::user("What is RAG?"),
            ChatMessage::assistant("RAG is Retrieval-Augmented Generation"),
        ];
        let tokens = TokenCounter::count_messages(&messages);
        assert!(tokens > 0);
    }

    #[test]
    fn test_empty_text() {
        assert_eq!(TokenCounter::count_text(""), 0);
    }
}
