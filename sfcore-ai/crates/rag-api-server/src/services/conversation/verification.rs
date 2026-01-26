/// verification.rs - LLM response verification with improved parsing
use tracing::{info, warn};

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
    
    /// Context is completely irrelevant to query
    NotRelevant {
        reason: String,
    },
}

pub struct LlmVerifier {
    #[allow(dead_code)]
    max_iterations: usize,
}

impl LlmVerifier {
    pub fn new(max_iterations: usize) -> Self {
        Self { max_iterations }
    }
    
    /// Parse LLM response to detect verification tags
    /// Handles malformed tags gracefully with fallback
    pub fn parse_response(&self, response: &str) -> VerificationResult {
        // Safety check: handle empty response
        if response.trim().is_empty() {
            warn!("Empty LLM response received");
            return VerificationResult::Answered("Maaf, saya tidak dapat memberikan jawaban.".to_string());
        }
        
        // Priority 1: Check NOT_RELEVANT tag
        if response.contains("<NOT_RELEVANT/>") {
            let cleaned = response.replace("<NOT_RELEVANT/>", "").trim().to_string();
            info!("LLM marked context as NOT_RELEVANT");
            
            return VerificationResult::NotRelevant {
                reason: cleaned,
            };
        }
        
        // Priority 2: Check NEED_MORE_CONTEXT tag (with error handling)
        if let Some(result) = self.try_parse_need_more_context(response) {
            return result;
        }
        
        // Default: Normal answered response
        let cleaned = response
            .replace("<NEED_MORE_CONTEXT", "")
            .replace("<NOT_RELEVANT/>", "")
            .trim()
            .to_string();
        
        VerificationResult::Answered(cleaned)
    }
    
    /// Try to parse NEED_MORE_CONTEXT tag with error handling
    fn try_parse_need_more_context(&self, response: &str) -> Option<VerificationResult> {
        // Use regex for robust parsing
        let re = match regex::Regex::new(r#"<NEED_MORE_CONTEXT\s+doc_ids="([^"]+)"\s*/>"#) {
            Ok(r) => r,
            Err(e) => {
                warn!("Regex compilation failed: {}", e);
                return None;
            }
        };
        
        if let Some(caps) = re.captures(response) {
            let doc_ids_str = &caps[1];
            
            // Parse doc_ids with validation
            let doc_ids: Vec<i64> = doc_ids_str
                .split(',')
                .filter_map(|s: &str| {
                    s.trim()
                        .strip_prefix("doc_")
                        .and_then(|id: &str| id.parse::<i64>().ok())
                })
                .collect();
            
            // Validate we got at least one valid doc_id
            if !doc_ids.is_empty() {
                info!("LLM needs more context from docs: {:?}", doc_ids);
                
                let cleaned = response
                    .replace(&caps[0], "")
                    .trim()
                    .to_string();
                
                return Some(VerificationResult::NeedMoreContext {
                    doc_ids,
                    reason: cleaned,
                });
            } else {
                warn!("NEED_MORE_CONTEXT tag found but no valid doc_ids: '{}'", doc_ids_str);
            }
        }
        
        None
    }
    
    /// Build enhanced system prompt with verification instructions
    pub fn build_verification_prompt(&self, base_instruction: &str) -> String {
        format!(
            r#"{base_instruction}

**━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━**
**CRITICAL VERIFICATION & CITATION RULES (MANDATORY)**
**━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━**

1️⃣ **SOURCE CITATION (WAJIB):**
   - SETIAP klaim faktual HARUS disertai sumber: [doc_ID]
   - Contoh: "Menurut [doc_123], budget Q1 adalah 500 juta rupiah"
   - Jika membandingkan dokumen:
     * "Dari [doc_123]: Budget 2023 adalah 500 juta"
     * "Sedangkan dari [doc_456]: Budget 2024 naik menjadi 750 juta"

2️⃣ **CONTEXT VERIFICATION:**
   A. Jika konteks TIDAK CUKUP tapi dokumen relevan:
      → Respond: <NEED_MORE_CONTEXT doc_ids="doc_1,doc_3"/>
      → Jelaskan singkat kenapa butuh info tambahan
   
   B. Jika konteks SAMA SEKALI TIDAK RELEVAN:
      → Respond: <NOT_RELEVANT/>
      → Jelaskan kenapa dokumen tidak relevan
   
   C. Jika konteks CUKUP:
      → Jawab lengkap dengan citation

3️⃣ **RESPONSE QUALITY:**
   - Mulai dengan jawaban langsung (jangan bertele-tele)
   - Gunakan angka/tanggal/nama spesifik dari dokumen
   - Jika ada konflik antar dokumen, sebutkan KEDUA versi
   - Jangan mengarang informasi yang tidak ada
   - Jika ragu, minta konteks tambahan

4️⃣ **MULTI-DOCUMENT HANDLING:**
   - Jika jawaban dari BEBERAPA dokumen, struktur seperti ini:
```
     Berdasarkan dokumen yang tersedia:
     
     1. [doc_123] menyatakan: (info dari doc 123)
     2. [doc_456] menunjukkan: (info dari doc 456)
     
     Kesimpulan: (synthesis)
```

**INGAT: Citation adalah WAJIB untuk setiap klaim faktual!**
**━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━**"#
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
        let response = r#"Info kurang. <NEED_MORE_CONTEXT doc_ids="doc_1,doc_3"/>"#;
        
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
        let response = "Dokumen tidak relevan. <NOT_RELEVANT/>";
        
        match verifier.parse_response(response) {
            VerificationResult::NotRelevant { .. } => {}
            _ => panic!("Expected NotRelevant"),
        }
    }

    #[test]
    fn test_parse_malformed_tag() {
        let verifier = LlmVerifier::new(3);
        let response = r#"<NEED_MORE_CONTEXT doc_ids="invalid,text"/>"#;
        
        // Should fallback to Answered since no valid doc_ids
        match verifier.parse_response(response) {
            VerificationResult::Answered(_) => {}
            _ => panic!("Expected Answered fallback"),
        }
    }

    #[test]
    fn test_parse_empty_response() {
        let verifier = LlmVerifier::new(3);
        let response = "";
        
        match verifier.parse_response(response) {
            VerificationResult::Answered(text) => {
                assert!(text.contains("tidak dapat"));
            }
            _ => panic!("Expected Answered with error message"),
        }
    }

    #[test]
    fn test_parse_multiple_doc_ids() {
        let verifier = LlmVerifier::new(3);
        let response = r#"<NEED_MORE_CONTEXT doc_ids="doc_10,doc_25,doc_100"/> Butuh info tambahan"#;
        
        match verifier.parse_response(response) {
            VerificationResult::NeedMoreContext { doc_ids, .. } => {
                assert_eq!(doc_ids, vec![10, 25, 100]);
            }
            _ => panic!("Expected NeedMoreContext"),
        }
    }
}