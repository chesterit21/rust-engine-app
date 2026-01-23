use anyhow::Result;
use std::path::Path;

#[derive(Debug)]
pub struct ParsedDocument {
    pub content: String,
    pub page_count: Option<usize>,
}

pub struct DocumentParser;

impl DocumentParser {
    pub fn parse(file_path: &Path) -> Result<ParsedDocument> {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "pdf" => Self::parse_pdf(file_path),
            "docx" | "doc" => Self::parse_docx(file_path),
            "txt" | "md" => Self::parse_text(file_path),
            _ => Self::parse_text(file_path), // fallback
        }
    }
    
    fn parse_pdf(file_path: &Path) -> Result<ParsedDocument> {
        use lopdf::Document;
        
        let doc = Document::load(file_path)?;
        let page_count = doc.get_pages().len();
        
        let mut content = String::new();
        
        for page_num in 1..=page_count {
            if let Ok(text) = doc.extract_text(&[page_num as u32]) {
                content.push_str(&text);
                content.push('\n');
            }
        }
        
        Ok(ParsedDocument {
            content,
            page_count: Some(page_count),
        })
    }
    
    fn parse_docx(file_path: &Path) -> Result<ParsedDocument> {
        use docx_rs::*;
        
        let data = std::fs::read(file_path)?;
        let docx = read_docx(&data)?;
        
        let mut content = String::new();
        
        // Extract text from paragraphs
        for child in docx.document.children {
            if let DocumentChild::Paragraph(para) = child {
                for child in para.children {
                    if let ParagraphChild::Run(run) = child {
                        for child in run.children {
                            if let RunChild::Text(text) = child {
                                content.push_str(&text.text);
                            }
                        }
                    }
                }
                content.push('\n');
            }
        }
        
        Ok(ParsedDocument {
            content,
            page_count: None,
        })
    }
    
    fn parse_text(file_path: &Path) -> Result<ParsedDocument> {
        let content = std::fs::read_to_string(file_path)?;
        
        Ok(ParsedDocument {
            content,
            page_count: None,
        })
    }
}
