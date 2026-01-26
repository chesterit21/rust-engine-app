Perfect bro! Sekarang gua ngerti struktur DB lo.

**Main Core Tables (GAK BOLEH DIUBAH):**

- `TblDocuments`
- `TblDocumentFiles`
- `TblCategories`
- `TblDocumentShared`
- `TblDocumentSharedPrivillege`

**RAG Tables (BISA DIUBAH/ALTER):**

- `rag_document_chunks` ✅
- `rag_document_metadata` ✅
- `rag_document_processing` ✅
- `rag_ingestion_log` ✅
- `tbl_activity_logs` ✅

---

## **🎯 IMPLEMENTATION PLAN - TERSTRUKTUR & BERURUTAN**

Gua susun dari **database migrations** → **code changes** → **testing** → **deployment**

---

# **PHASE 1: Database Enhancements**

## **Step 1.1: Alter RAG Tables untuk Support Multi-Doc & Token Tracking**

```sql
-- Migration: 001_enhance_rag_chunks.sql
-- Timestamp: 2026-01-26

-- Add metadata JSONB column untuk flexible attributes
ALTER TABLE rag_document_chunks 
ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';

-- Create GIN index untuk metadata queries
CREATE INDEX IF NOT EXISTS idx_rag_chunks_metadata 
ON rag_document_chunks USING GIN(metadata);

-- Metadata structure example:
-- {
--   "doc_title": "Budget_Q1_2024.pdf",
--   "section_title": "Financial Summary",
--   "keywords": ["budget", "revenue", "Q1"],
--   "estimated_tokens": 150,
--   "language": "id"
-- }

COMMENT ON COLUMN rag_document_chunks.metadata IS 
'Flexible JSONB field for document metadata, section info, and token estimates';
```

```sql
-- Migration: 002_enhance_activity_logs.sql
-- Timestamp: 2026-01-26

-- Add columns untuk track retrieval iterations & context quality
ALTER TABLE tbl_activity_logs
ADD COLUMN IF NOT EXISTS retrieval_iterations INTEGER DEFAULT 1,
ADD COLUMN IF NOT EXISTS context_truncated BOOLEAN DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS context_token_count INTEGER,
ADD COLUMN IF NOT EXISTS documents_retrieved INTEGER,
ADD COLUMN IF NOT EXISTS verification_result VARCHAR(50); -- 'answered', 'need_more', 'not_relevant'

CREATE INDEX IF NOT EXISTS idx_activity_logs_verification 
ON tbl_activity_logs(verification_result) 
WHERE verification_result IS NOT NULL;

COMMENT ON COLUMN tbl_activity_logs.retrieval_iterations IS 
'Number of retrieval attempts made for this query';
COMMENT ON COLUMN tbl_activity_logs.context_truncated IS 
'Whether context was truncated due to token limits';
COMMENT ON COLUMN tbl_activity_logs.verification_result IS 
'LLM verification result: answered, need_more_context, not_relevant';
```

```sql
-- Migration: 003_add_retrieval_cache.sql (OPTIONAL - untuk optimization)
-- Timestamp: 2026-01-26

CREATE TABLE IF NOT EXISTS rag_retrieval_cache (
    cache_id SERIAL PRIMARY KEY,
    query_hash VARCHAR(64) NOT NULL, -- MD5/SHA256 of normalized query
    document_id INTEGER,
    user_id INTEGER NOT NULL,
    chunk_ids INTEGER[] NOT NULL, -- Array of chunk IDs
    embedding_vector vector, -- Optional: store query embedding
    similarity_threshold DOUBLE PRECISION,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    access_count INTEGER DEFAULT 1
);

CREATE INDEX idx_retrieval_cache_hash 
ON rag_retrieval_cache(query_hash, document_id, user_id);

CREATE INDEX idx_retrieval_cache_created 
ON rag_retrieval_cache(created_at);

-- TTL: Delete cache older than 7 days
CREATE INDEX idx_retrieval_cache_ttl 
ON rag_retrieval_cache(created_at) 
WHERE created_at < NOW() - INTERVAL '7 days';

COMMENT ON TABLE rag_retrieval_cache IS 
'Cache for frequently accessed query-document retrieval results';
```

---

# **PHASE 2: Code Implementation - Berurutan**

## **Step 2.1: Create Token Estimator Module**

```rust
// File: src/utils/token_estimator.rs (NEW FILE)

/// Token estimation utilities untuk manage context window
use crate::database::DocumentChunk;

/// Estimate tokens dari text (rough approximation)
/// Rule: ~1.3 tokens per word for Indonesian/English mix
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    
    // Count words (split by whitespace)
    let words = text.split_whitespace().count();
    
    // Indonesian/English average: 1.3 tokens per word
    (words as f64 * 1.3).ceil() as usize
}

/// Estimate total tokens from multiple chunks
pub fn estimate_chunks_tokens(chunks: &[DocumentChunk]) -> usize {
    chunks.iter()
        .map(|chunk| estimate_tokens(&chunk.content))
        .sum()
}

/// Check if adding chunk would exceed token limit
pub fn would_exceed_limit(
    current_tokens: usize,
    new_text: &str,
    max_tokens: usize,
) -> bool {
    let new_tokens = estimate_tokens(new_text);
    current_tokens + new_tokens > max_tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_estimation() {
        // "Halo dunia ini adalah test" = 5 words
        let text = "Halo dunia ini adalah test";
        let tokens = estimate_tokens(text);
        assert_eq!(tokens, 7); // 5 * 1.3 = 6.5 → 7
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn test_exceed_limit() {
        let current = 1000;
        let text = "a ".repeat(500); // ~500 words = 650 tokens
        assert!(would_exceed_limit(current, &text, 1500));
        assert!(!would_exceed_limit(current, &text, 2000));
    }
}
```

```rust
// File: src/utils/mod.rs
// Add module declaration

pub mod error;
pub mod similarity;
pub mod token_estimator; // NEW
```

---

## **Step 2.2: Enhanced Document Grouping & Structured Context**

```rust
// File: src/services/rag_service.rs
// Add new methods AFTER existing retrieve_with_embedding()

use std::collections::HashMap;
use crate::utils::token_estimator;

/// Grouped chunks by document with metadata
#[derive(Debug, Clone)]
pub struct GroupedDocument {
    pub doc_id: i32,
    pub doc_title: String,
    pub chunks: Vec<DocumentChunk>,
    pub avg_similarity: f32,
    pub total_tokens: usize,
}

impl RagService {
    /// Build STRUCTURED context dengan XML tags untuk multi-document clarity
    pub fn build_structured_context(
        &self,
        chunks: Vec<DocumentChunk>,
        max_tokens: usize,
    ) -> (String, ContextMetrics) {
        if chunks.is_empty() {
            return (
                "Tidak ada konteks yang relevan ditemukan.".to_string(),
                ContextMetrics::default(),
            );
        }
        
        // Group chunks by document
        let grouped = self.group_chunks_by_document(chunks);
        
        // Sort documents by relevance (highest similarity first)
        let mut sorted_docs: Vec<GroupedDocument> = grouped.into_values().collect();
        sorted_docs.sort_by(|a, b| {
            b.avg_similarity.partial_cmp(&a.avg_similarity).unwrap()
        });
        
        // Build context dengan token-aware truncation
        self.format_grouped_context(sorted_docs, max_tokens)
    }
    
    /// Group chunks by document ID
    fn group_chunks_by_document(
        &self,
        chunks: Vec<DocumentChunk>,
    ) -> HashMap<i32, GroupedDocument> {
        let mut grouped: HashMap<i32, GroupedDocument> = HashMap::new();
        
        for chunk in chunks {
            let entry = grouped.entry(chunk.document_id)
                .or_insert_with(|| GroupedDocument {
                    doc_id: chunk.document_id,
                    doc_title: chunk.document_title.clone(),
                    chunks: Vec::new(),
                    avg_similarity: 0.0,
                    total_tokens: 0,
                });
            
            entry.total_tokens += token_estimator::estimate_tokens(&chunk.content);
            entry.chunks.push(chunk);
        }
        
        // Calculate average similarity per document
        for doc in grouped.values_mut() {
            let sum: f32 = doc.chunks.iter()
                .filter_map(|c| c.similarity)
                .sum();
            doc.avg_similarity = if doc.chunks.is_empty() {
                0.0
            } else {
                sum / doc.chunks.len() as f32
            };
        }
        
        grouped
    }
    
    /// Format grouped documents dengan XML structure
    fn format_grouped_context(
        &self,
        sorted_docs: Vec<GroupedDocument>,
        max_tokens: usize,
    ) -> (String, ContextMetrics) {
        let mut context = String::from("DOKUMEN YANG TERSEDIA:\n\n");
        let mut metrics = ContextMetrics::default();
        let mut current_tokens = token_estimator::estimate_tokens(&context);
        
        for doc in sorted_docs {
            // Document header
            let doc_header = format!(
                "<document id=\"doc_{}\" title=\"{}\" relevance=\"{:.3}\">\n",
                doc.doc_id,
                doc.doc_title,
                doc.avg_similarity
            );
            
            let header_tokens = token_estimator::estimate_tokens(&doc_header);
            
            // Check if adding this doc would exceed limit
            if current_tokens + header_tokens > max_tokens {
                metrics.truncated = true;
                break;
            }
            
            context.push_str(&doc_header);
            current_tokens += header_tokens;
            metrics.documents_included += 1;
            
            // Add chunks for this document
            for chunk in &doc.chunks {
                let chunk_text = format!(
                    "<chunk id=\"chunk_{}\" page=\"{}\" similarity=\"{:.3}\">\n{}\n</chunk>\n\n",
                    chunk.chunk_id,
                    chunk.page_number.unwrap_or(0),
                    chunk.similarity.unwrap_or(0.0),
                    chunk.content.trim()
                );
                
                let chunk_tokens = token_estimator::estimate_tokens(&chunk_text);
                
                if current_tokens + chunk_tokens > max_tokens {
                    metrics.truncated = true;
                    break;
                }
                
                context.push_str(&chunk_text);
                current_tokens += chunk_tokens;
                metrics.chunks_included += 1;
            }
            
            context.push_str("</document>\n\n");
            current_tokens += 2; // closing tag tokens
            
            if metrics.truncated {
                break;
            }
        }
        
        metrics.total_tokens = current_tokens;
        
        (context, metrics)
    }
}

/// Metrics untuk track context quality
#[derive(Debug, Default, Clone)]
pub struct ContextMetrics {
    pub total_tokens: usize,
    pub documents_included: usize,
    pub chunks_included: usize,
    pub truncated: bool,
}
```

---

## **Step 2.3: LLM Verification & Iterative Retrieval**

```rust
// File: src/services/conversation/verification.rs (NEW FILE)

use anyhow::Result;
use regex::Regex;
use tracing::{debug, info};

/// LLM response verification result
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// LLM successfully answered the question
    Answered(String),
    /// LLM needs more context from specific documents
    NeedMoreContext {
        doc_ids: Vec<i64>,
        reason: String,
    },
    /// Context is not relevant to the query
    NotRelevant(String),
}

pub struct LlmVerifier {
    max_iterations: usize,
}

impl LlmVerifier {
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }
    
    /// Parse LLM response untuk detect verification tags
    pub fn parse_response(&self, response: &str) -> VerificationResult {
        // Check for NOT_RELEVANT tag
        if response.contains("<NOT_RELEVANT/>") {
            let cleaned = response.replace("<NOT_RELEVANT/>", "").trim().to_string();
            info!("LLM marked context as NOT_RELEVANT");
            return VerificationResult::NotRelevant(cleaned);
        }
        
        // Check for NEED_MORE_CONTEXT tag
        let re = Regex::new(r#"<NEED_MORE_CONTEXT\s+doc_ids="([^"]+)"\s*/>"#).unwrap();
        
        if let Some(caps) = re.captures(response) {
            let doc_ids_str = &caps[1];
            let doc_ids: Vec<i64> = doc_ids_str
                .split(',')
                .filter_map(|s| s.trim().strip_prefix("doc_"))
                .filter_map(|s| s.parse().ok())
                .collect();
            
            if !doc_ids.is_empty() {
                info!("LLM needs more context from docs: {:?}", doc_ids);
                
                let cleaned = response
                    .replace(&caps[0], "")
                    .trim()
                    .to_string();
                
                return VerificationResult::NeedMoreContext {
                    doc_ids,
                    reason: cleaned,
                };
            }
        }
        
        // Normal response - clean up any residual tags
        let cleaned = response
            .replace("<NEED_MORE_CONTEXT", "")
            .replace("<NOT_RELEVANT/>", "")
            .trim()
            .to_string();
        
        VerificationResult::Answered(cleaned)
    }
    
    /// Build enhanced system prompt dengan verification instructions
    pub fn build_verification_prompt(&self, base_instruction: &str) -> String {
        format!(
            r#"{base_instruction}

**CRITICAL VERIFICATION & CITATION RULES:**

1. **Source Citation (MANDATORY):**
   - ALWAYS cite sources using: [doc_ID] or [doc_ID, chunk_ID]
   - Example: "Menurut [doc_123], budget Q1 adalah 500 juta"
   - When comparing documents: "Dari [doc_123]: X. Sedangkan dari [doc_456]: Y."

2. **Context Verification:**
   - If context is INSUFFICIENT but documents are relevant: 
     Respond with: <NEED_MORE_CONTEXT doc_ids="doc_1,doc_3"/>
   - If context is COMPLETELY IRRELEVANT:
     Respond with: <NOT_RELEVANT/>
   - Otherwise: Provide complete answer with citations

3. **Response Structure:**
   - Start with direct answer
   - Include citations inline
   - If multiple docs: clearly distinguish sources
   - Admit when info is missing

4. **Quality Guidelines:**
   - Never invent information
   - Be specific with numbers/dates/names
   - Flag conflicts between documents
   - Keep responses concise but complete

REMEMBER: Citations are MANDATORY for every factual claim!"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_answered() {
        let verifier = LlmVerifier::new(3);
        let response = "Budget Q1 adalah 500 juta [doc_123]";
        
        match verifier.parse_response(response) {
            VerificationResult::Answered(text) => {
                assert!(text.contains("500 juta"));
            }
            _ => panic!("Expected Answered"),
        }
    }

    #[test]
    fn test_parse_need_more_context() {
        let verifier = LlmVerifier::new(3);
        let response = r#"Informasi kurang lengkap. <NEED_MORE_CONTEXT doc_ids="doc_1,doc_3"/>"#;
        
        match verifier.parse_response(response) {
            VerificationResult::NeedMoreContext { doc_ids, .. } => {
                assert_eq!(doc_ids, vec![1, 3]);
            }
            _ => panic!("Expected NeedMoreContext"),
        }
    }

    #[test]
    fn test_parse_not_relevant() {
        let verifier = LlmVerifier::new(3);
        let response = "Maaf, dokumen tidak relevan. <NOT_RELEVANT/>";
        
        match verifier.parse_response(response) {
            VerificationResult::NotRelevant(_) => {}
            _ => panic!("Expected NotRelevant"),
        }
    }
}
```

---

## **Step 2.4: Update ConversationManager dengan Iterative Flow**

```rust
// File: src/services/conversation/manager.rs
// Add imports dan update handle_message()

use crate::services::conversation::verification::{LlmVerifier, VerificationResult};
use crate::services::rag_service::ContextMetrics;
use std::collections::HashSet;

impl ConversationManager {
    // Update existing handle_message() method
    pub async fn handle_message(
        &self,
        session_id: SessionId,
        user_id: i64,
        message: String,
        document_id: Option<i64>,
    ) -> Result<impl Stream<Item = Result<String>>> {
        
        // ... existing code until retrieval decision ...
        
        // NEW: Iterative retrieval loop
        let verifier = LlmVerifier::new(3); // max 3 iterations
        let mut tried_chunk_ids: HashSet<i64> = HashSet::new();
        let mut iteration = 0;
        let max_iterations = 3;
        
        let final_response = loop {
            iteration += 1;
            
            // Execute retrieval
            let chunks = self.execute_retrieval_with_decision(
                &decision,
                user_id,
                &message,
                &query_embedding,
                document_id,
                &tried_chunk_ids,
            ).await?;
            
            // Track tried chunks
            for chunk in &chunks {
                tried_chunk_ids.insert(chunk.chunk_id);
            }
            
            // Build structured context
            const MAX_CONTEXT_TOKENS: usize = 20_000; // Reserve 12K for system+response
            let (context, metrics) = self.rag_service.build_structured_context(
                chunks,
                MAX_CONTEXT_TOKENS,
            );
            
            // Build messages dengan enhanced prompt
            let enhanced_system = verifier.build_verification_prompt(
                &self.base_system_prompt
            );
            
            let messages = vec![
                crate::models::chat::ChatMessage {
                    role: "system".to_string(),
                    content: format!("{}\n\n{}", enhanced_system, context),
                },
                crate::models::chat::ChatMessage {
                    role: "user".to_string(),
                    content: message.clone(),
                },
            ];
            
            // Call LLM
            let llm_response = if self.stream_enabled {
                // Handle streaming...
                self.llm_service.stream_complete(&messages).await?
            } else {
                self.llm_service.complete_simple(&messages).await?
            };
            
            // Verify response
            match verifier.parse_response(&llm_response) {
                VerificationResult::Answered(answer) => {
                    // Log success
                    self.logger.log_success(
                        session_id,
                        user_id,
                        &message,
                        &answer,
                        iteration,
                        metrics.total_tokens,
                        metrics.documents_included as i32,
                    ).await;
                    
                    break answer;
                }
                
                VerificationResult::NeedMoreContext { doc_ids, .. } => {
                    if iteration >= max_iterations {
                        let fallback = "Maaf, saya tidak menemukan cukup informasi setelah beberapa kali pencarian.".to_string();
                        
                        self.logger.log_insufficient_context(
                            session_id,
                            user_id,
                            &message,
                            iteration,
                        ).await;
                        
                        break fallback;
                    }
                    
                    info!("Iteration {}: Need more context from docs {:?}", iteration, doc_ids);
                    
                    // Get next batch - akan di-handle di loop berikutnya
                    // Update decision untuk target specific docs
                    continue;
                }
                
                VerificationResult::NotRelevant(_) => {
                    if iteration >= max_iterations {
                        let fallback = "Maaf, tidak ada dokumen yang relevan dengan pertanyaan Anda.".to_string();
                        
                        self.logger.log_not_relevant(
                            session_id,
                            user_id,
                            &message,
                            iteration,
                        ).await;
                        
                        break fallback;
                    }
                    
                    info!("Iteration {}: Context not relevant, expanding search", iteration);
                    
                    // Expand to different documents
                    continue;
                }
            }
        };
        
        // Return final response as stream
        Ok(stream::once(async move { Ok(final_response) }))
    }
    
    /// Execute retrieval dengan filter untuk exclude tried chunks
    async fn execute_retrieval_with_decision(
        &self,
        decision: &RetrievalDecision,
        user_id: i64,
        query: &str,
        embedding: &[f32],
        document_id: Option<i64>,
        exclude_chunks: &HashSet<i64>,
    ) -> Result<Vec<RetrievalChunk>> {
        // ... existing retrieval logic ...
        
        let mut chunks = self.rag_service.search(
            user_id,
            embedding,
            document_id,
        ).await?;
        
        // Filter out already-tried chunks
        chunks.retain(|c| !exclude_chunks.contains(&c.chunk_id));
        
        Ok(chunks)
    }
}
```

---

**Bro, ini udah panjang banget. Mau gua lanjutin ke:**

1. **Step 2.5**: Update ActivityLogger untuk track iterations?
2. **Step 2.6**: Testing strategy?
3. **Step 2.7**: Deployment checklist?

Atau lo mau gua compile semua jadi **SATU DOKUMEN LENGKAP** yang bisa langsung lo implement step-by-step?
