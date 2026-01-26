// File: tests/integration_test.rs (CREATE NEW)

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_estimation_accuracy() {
        use crate::utils::token_estimator;
        
        let samples = vec![
            ("Halo dunia", 4),  // ~2 words * 1.3 + 5 = 7.6 ≈ 8
            ("Ini adalah dokumen yang berisi informasi penting", 10), // ~7 words
            ("", 0),
        ];
        
        for (text, expected_range) in samples {
            let tokens = token_estimator::estimate_tokens(text);
            assert!(
                tokens >= expected_range - 2 && tokens <= expected_range + 2,
                "Token estimation for '{}' out of range. Got: {}, Expected: ~{}",
                text, tokens, expected_range
            );
        }
    }
    
    #[tokio::test]
    async fn test_multi_document_grouping() {
        // Setup mock chunks from different documents
        let chunks = vec![
            create_mock_chunk(1, 123, "Budget Q1", 0.95),
            create_mock_chunk(2, 123, "Budget Q2", 0.90),
            create_mock_chunk(3, 456, "Forecast 2024", 0.85),
            create_mock_chunk(4, 456, "Revenue projections", 0.80),
        ];
        
        // Group chunks
        let rag_service = create_mock_rag_service();
        let grouped = rag_service.group_chunks_by_document(chunks);
        
        // Assertions
        assert_eq!(grouped.len(), 2, "Should group into 2 documents");
        assert!(grouped.contains_key(&123));
        assert!(grouped.contains_key(&456));
        
        let doc_123 = &grouped[&123];
        assert_eq!(doc_123.chunks.len(), 2);
        assert!(doc_123.avg_similarity > 0.9);
    }
    
    #[test]
    fn test_verification_tag_parsing() {
        use crate::services::conversation::verification::{LlmVerifier, VerificationResult};
        
        let verifier = LlmVerifier::new(3);
        
        // Test NEED_MORE_CONTEXT
        let response = r#"Info tidak lengkap. <NEED_MORE_CONTEXT doc_ids="doc_1,doc_5,doc_10"/>"#;
        match verifier.parse_response(response) {
            VerificationResult::NeedMoreContext { doc_ids, .. } => {
                assert_eq!(doc_ids, vec![1, 5, 10]);
            }
            _ => panic!("Should parse as NeedMoreContext"),
        }
        
        // Test NOT_RELEVANT
        let response = "Dokumen tidak relevan. <NOT_RELEVANT/>";
        match verifier.parse_response(response) {
            VerificationResult::NotRelevant { .. } => {}
            _ => panic!("Should parse as NotRelevant"),
        }
        
        // Test normal answer
        let response = "Budget adalah 500 juta [doc_123]";
        match verifier.parse_response(response) {
            VerificationResult::Answered(text) => {
                assert!(text.contains("500 juta"));
            }
            _ => panic!("Should parse as Answered"),
        }
    }
    
    fn create_mock_chunk(id: i64, doc_id: i32, content: &str, sim: f32) -> DocumentChunk {
        DocumentChunk {
            chunk_id: id,
            document_id: doc_id,
            document_title: format!("Doc_{}", doc_id),
            content: content.to_string(),
            similarity: sim,
            chunk_index: 0,
            page_number: None,
        }
    }
}