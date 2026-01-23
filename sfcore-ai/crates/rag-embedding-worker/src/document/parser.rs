use anyhow::{anyhow, Result};
use encoding_rs::{Encoding, UTF_8};
use lopdf::Document as PdfDocument;
use pulldown_cmark::{Parser as MdParser, html, Options};
use scraper::{Html, Selector};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub content: String,
    pub metadata: DocumentMetadata,
}

#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub file_type: String,
    pub pages: Option<usize>,
    pub char_count: usize,
    pub encoding: String,
}

pub struct DocumentParser;

impl DocumentParser {
    /// Parse document dari path
    pub fn parse(path: &Path) -> Result<ParsedDocument> {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("No file extension"))?
            .to_lowercase();
        
        debug!("Parsing file: {:?} (type: {})", path, extension);
        
        let (content, metadata) = match extension.as_str() {
            "pdf" => Self::parse_pdf(path)?,
            "docx" => Self::parse_docx(path)?,
            "md" => Self::parse_markdown(path)?,
            "html" | "htm" => Self::parse_html(path)?,
            _ => Self::parse_text(path)?, // fallback untuk semua text-based
        };
        
        debug!("Parsed {} characters from {:?}", content.len(), path);
        
        Ok(ParsedDocument { content, metadata })
    }
    
    /// Parse PDF using lopdf
    fn parse_pdf(path: &Path) -> Result<(String, DocumentMetadata)> {
        let doc = PdfDocument::load(path)?;
        let pages = doc.get_pages();
        let page_count = pages.len();
        
        let mut content = String::new();
        
        for (page_num, _) in pages.iter() {
            match doc.extract_text(&[*page_num]) {
                Ok(text) => {
                    content.push_str(&text);
                    content.push('\n');
                }
                Err(e) => {
                    warn!("Failed to extract text from page {}: {}", page_num, e);
                }
            }
        }
        
        let metadata = DocumentMetadata {
            file_type: "application/pdf".to_string(),
            pages: Some(page_count),
            char_count: content.len(),
            encoding: "UTF-8".to_string(),
        };
        
        Ok((content, metadata))
    }
    
    /// Parse DOCX
    fn parse_docx(path: &Path) -> Result<(String, DocumentMetadata)> {
        let content = fs::read(path)?;
        let doc = docx_rs::read_docx(&content)?;
        
        // Extract text dari document
        // TODO: Implement proper DOCX text extraction
        let text = format!("DOCX content extraction placeholder for {:?}", path);
        
        let metadata = DocumentMetadata {
            file_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                .to_string(),
            pages: None,
            char_count: text.len(),
            encoding: "UTF-8".to_string(),
        };
        
        Ok((text, metadata))
    }
    
    /// Parse Markdown dan convert ke plain text
    fn parse_markdown(path: &Path) -> Result<(String, DocumentMetadata)> {
        let raw_content = fs::read(path)?;
        let (content, encoding) = Self::decode_text(&raw_content)?;
        
        // Parse markdown ke HTML dulu, lalu extract text
        let parser = MdParser::new_ext(&content, Options::all());
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        // Extract text from HTML
        let text = Self::extract_text_from_html(&html_output)?;
        
        let metadata = DocumentMetadata {
            file_type: "text/markdown".to_string(),
            pages: None,
            char_count: text.len(),
            encoding: encoding.name().to_string(),
        };
        
        Ok((text, metadata))
    }
    
    /// Parse HTML dan extract text
    fn parse_html(path: &Path) -> Result<(String, DocumentMetadata)> {
        let raw_content = fs::read(path)?;
        let (content, encoding) = Self::decode_text(&raw_content)?;
        
        let text = Self::extract_text_from_html(&content)?;
        
        let metadata = DocumentMetadata {
            file_type: "text/html".to_string(),
            pages: None,
            char_count: text.len(),
            encoding: encoding.name().to_string(),
        };
        
        Ok((text, metadata))
    }
    
    /// Extract text dari HTML menggunakan scraper
    fn extract_text_from_html(html: &str) -> Result<String> {
        let document = Html::parse_document(html);
        
        // Remove script dan style tags
        let body_selector = Selector::parse("body").unwrap();
        let script_selector = Selector::parse("script, style").unwrap();
        
        let mut text = String::new();
        
        for element in document.select(&body_selector) {
            let html_text = element.html();
            let doc = Html::parse_fragment(&html_text);
            
            // Remove scripts/styles
            for elem in doc.select(&script_selector) {
                // Skip these elements
                continue;
            }
            
            // Extract text
            text.push_str(&element.text().collect::<String>());
        }
        
        // Cleanup: remove excessive whitespace
        let cleaned = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        
        Ok(cleaned)
    }
    
    /// Parse plain text
    fn parse_text(path: &Path) -> Result<(String, DocumentMetadata)> {
        let raw_content = fs::read(path)?;
        let (content, encoding) = Self::decode_text(&raw_content)?;
        
        let metadata = DocumentMetadata {
            file_type: "text/plain".to_string(),
            pages: None,
            char_count: content.len(),
            encoding: encoding.name().to_string(),
        };
        
        Ok((content, metadata))
    }
    
    /// Decode text dengan encoding detection
    fn decode_text(bytes: &[u8]) -> Result<(String, &'static Encoding)> {
        // Try UTF-8 first
        if let Ok(text) = std::str::from_utf8(bytes) {
            return Ok((text.to_string(), UTF_8));
        }
        
        // Auto-detect encoding
        let (encoding, _, _) = UTF_8.decode(bytes);
        
        Ok((encoding.to_string(), UTF_8))
    }
}