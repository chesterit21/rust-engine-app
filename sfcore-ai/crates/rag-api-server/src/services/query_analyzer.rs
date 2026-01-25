/// Query Intent Analyzer
/// Detects whether user is asking about document overview/metadata
/// vs asking specific questions that need vector search

use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryIntent {
    /// Generic questions about document overview
    /// Examples: "ini dokumen tentang apa?", "what is this document about?"
    DocumentOverview,
    
    /// Questions asking for summary/main topic
    /// Examples: "ringkas dokumen ini", "summarize this"
    DocumentSummary,
    
    /// Specific factual questions that need vector search
    /// Examples: "berapa harga produk X?", "what is the deadline?"
    SpecificContent,
    
    /// Clarification about previous answers
    /// Examples: "maksudnya?", "bisa jelasin lagi?"
    Clarification,
}

pub struct QueryAnalyzer;

impl QueryAnalyzer {
    /// Analyze query intent based on pattern matching
    pub fn analyze_intent(query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();
        
        // Pattern 1: Document overview questions
        let overview_patterns = [
            "dokumen tentang apa",
            "dokumen ini tentang apa",
            "isi dokumen",
            "ini tentang apa",
            "topik apa",
            "membahas apa",
            "document about",
            "what is this document",
            "what does this document",
            "what's in this document",
            "apa isi dokumen",
            "dokumen apa ini",
            "ini dokumen apa",
        ];
        
        for pattern in &overview_patterns {
            if query_lower.contains(pattern) {
                debug!("Detected DocumentOverview intent: matched '{}'", pattern);
                return QueryIntent::DocumentOverview;
            }
        }
        
        // Pattern 2: Summary requests
        let summary_patterns = [
            "ringkas",
            "summarize",
            "rangkum",
            "kesimpulan",
            "intisari",
            "summary",
            "tldr",
            "ringkasan",
            "buatkan ringkasan",
            "tolong ringkas",
        ];
        
        for pattern in &summary_patterns {
            if query_lower.contains(pattern) {
                debug!("Detected DocumentSummary intent: matched '{}'", pattern);
                return QueryIntent::DocumentSummary;
            }
        }
        
        // Pattern 3: Clarification questions
        let clarification_patterns = [
            "maksudnya",
            "maksud",
            "apa artinya",
            "jelaskan",
            "what do you mean",
            "explain",
            "bisa jelaskan",
            "tolong jelaskan",
            "coba jelaskan lagi",
            "apa maksud",
        ];
        
        for pattern in &clarification_patterns {
            if query_lower.contains(pattern) {
                debug!("Detected Clarification intent: matched '{}'", pattern);
                return QueryIntent::Clarification;
            }
        }
        
        // Default: specific content query (needs vector search)
        debug!("Defaulting to SpecificContent intent");
        QueryIntent::SpecificContent
    }
    
    /// Check if query is likely a meta-question (overview or summary)
    pub fn is_meta_question(query: &str) -> bool {
        matches!(
            Self::analyze_intent(query),
            QueryIntent::DocumentOverview | QueryIntent::DocumentSummary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_overview_intent() {
        assert_eq!(
            QueryAnalyzer::analyze_intent("ini dokumen tentang apa ya?"),
            QueryIntent::DocumentOverview
        );
        
        assert_eq!(
            QueryAnalyzer::analyze_intent("What is this document about?"),
            QueryIntent::DocumentOverview
        );
        
        assert_eq!(
            QueryAnalyzer::analyze_intent("Dokumen ini membahas apa?"),
            QueryIntent::DocumentOverview
        );
    }

    #[test]
    fn test_summary_intent() {
        assert_eq!(
            QueryAnalyzer::analyze_intent("ringkas dokumen ini"),
            QueryIntent::DocumentSummary
        );
        
        assert_eq!(
            QueryAnalyzer::analyze_intent("give me a summary"),
            QueryIntent::DocumentSummary
        );
    }

    #[test]
    fn test_clarification_intent() {
        assert_eq!(
            QueryAnalyzer::analyze_intent("maksudnya apa?"),
            QueryIntent::Clarification
        );
        
        assert_eq!(
            QueryAnalyzer::analyze_intent("bisa jelaskan lagi?"),
            QueryIntent::Clarification
        );
    }

    #[test]
    fn test_specific_content_intent() {
        assert_eq!(
            QueryAnalyzer::analyze_intent("berapa harga produk X?"),
            QueryIntent::SpecificContent
        );
        
        assert_eq!(
            QueryAnalyzer::analyze_intent("what is the deadline for project Y?"),
            QueryIntent::SpecificContent
        );
    }

    #[test]
    fn test_is_meta_question() {
        assert!(QueryAnalyzer::is_meta_question("ini dokumen tentang apa?"));
        assert!(QueryAnalyzer::is_meta_question("ringkas dokumen ini"));
        assert!(!QueryAnalyzer::is_meta_question("berapa harga produk?"));
    }
}
