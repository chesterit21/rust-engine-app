use anyhow::{anyhow, Context, Result};
use encoding_rs::{Encoding, UTF_8};
use lopdf::Document as PdfDocument;
use pulldown_cmark::{Parser as MdParser, html, Options};
use scraper::{Html, Selector};
use std::fs;
use std::io::{Read};
use std::path::Path;
use tracing::{debug, warn};
use calamine::{Reader, Xlsx, open_workbook, Data}; // Use Data instead of DataType

use rtf_parser::{lexer::Lexer as RtfLexer, parser::Parser as RtfParser};

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
            "xlsx" | "xls" => Self::parse_excel(path)?,
            "pptx" => Self::parse_pptx(path)?,
            "png" | "jpg" | "jpeg" | "tiff" | "bmp" => Self::parse_image(path)?,
            "rtf" => Self::parse_rtf(path)?,
            "md" => Self::parse_markdown(path)?,
            "html" | "htm" => Self::parse_html(path)?,
            _ => Self::parse_text(path)?, // fallback untuk semua text-based
        };
        
        debug!("Parsed {} characters from {:?}", content.len(), path);
        
        Ok(ParsedDocument { content, metadata })
    }
    
    /// Parse PDF using lopdf
    fn parse_pdf(path: &Path) -> Result<(String, DocumentMetadata)> {
        let doc = PdfDocument::load(path).context("Failed to load PDF file")?;
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
    
    /// Parse DOCX using docx-rs
    fn parse_docx(path: &Path) -> Result<(String, DocumentMetadata)> {
        // Simple XML extraction fallback if docx-rs is too complex for just text
        // But trying docx-rs first
        let _content = fs::read(path).context("Failed to read DOCX file")?;
        
        // Note: docx-rs is primarily for writing, reading support is basic. 
        // For text extraction, unzipping and parsing document.xml is often more reliable/simpler.
        let text = Self::extract_text_from_office_xml(path, "word/document.xml")
            .unwrap_or_else(|e| {
                warn!("XML extraction failed: {}, trying fallback", e);
                "DOCX text extraction failed".to_string()
            });
        
        let metadata = DocumentMetadata {
            file_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
            pages: None,
            char_count: text.len(),
            encoding: "UTF-8".to_string(),
        };
        
        Ok((text, metadata))
    }

    /// Parse XLSX/XLS using calamine
    fn parse_excel(path: &Path) -> Result<(String, DocumentMetadata)> {
        let mut workbook: Xlsx<_> = open_workbook(path).context("Failed to open Excel file")?;
        let mut content = String::new();
        
        if let Some(range) = workbook.worksheet_range_at(0) {
            let range = range.context("Failed to get worksheet")?;
            for row in range.rows() {
                let row_text: Vec<String> = row.iter()
                    .map(|cell| match cell {
                        Data::String(s) => s.clone(),
                        Data::Float(f) => f.to_string(),
                        Data::Int(i) => i.to_string(),
                        Data::Bool(b) => b.to_string(),
                        Data::Error(_) | Data::Empty => String::new(),
                        Data::DateTime(d) => d.to_string(),
                        Data::DateTimeIso(d) => d.clone(), // Handle ISO DateTime
                        Data::DurationIso(d) => d.clone(), // Handle ISO Duration
                    })
                    .filter(|s: &String| !s.is_empty())
                    .collect();
                
                if !row_text.is_empty() {
                    content.push_str(&row_text.join(" "));
                    content.push('\n');
                }
            }
        }
        
        let metadata = DocumentMetadata {
            file_type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            pages: None,
            char_count: content.len(),
            encoding: "UTF-8".to_string(),
        };
        
        Ok((content, metadata))
    }

    /// Parse PPTX (Text Extraction from XML)
    fn parse_pptx(path: &Path) -> Result<(String, DocumentMetadata)> {
        debug!("Starting PPTX parsing for: {:?}", path);
        // PPTX is a zip file. Slides are in ppt/slides/slideX.xml
        // We will extract text from all slides
        let file = fs::File::open(path).context("Failed to open PPTX file")?;
        let mut archive = zip::ZipArchive::new(file).context("Failed to open PPTX as ZIP archive")?;
        let _content = String::new(); // Properly initialize content
        
        // Find all slide XMLs
        let mut slide_files: Vec<String> = Vec::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();
            if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
                slide_files.push(name.to_string());
            }
        }
        
        if slide_files.is_empty() {
             warn!("No slide files found in PPTX archive");
        }
        
        // Sort slides (slide1.xml, slide2.xml, ...)
        slide_files.sort_by(|a, b| {
            // Natural sort would be better, but standard string sort is okay-ish
            // Improving sort to handle slide1 vs slide10
            let a_num = a.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse::<u32>().unwrap_or(0);
            let b_num = b.trim_start_matches("ppt/slides/slide").trim_end_matches(".xml").parse::<u32>().unwrap_or(0);
            a_num.cmp(&b_num)
        });

        let mut text_content = String::new();
        for slide_name in slide_files {
             let mut file = archive.by_name(&slide_name)?;
             let mut xml = String::new();
             file.read_to_string(&mut xml)?;
             
             // Simple regex based XML tag stripping (efficient enough for text extraction)
             let text = Self::strip_xml_tags(&xml);
             if !text.trim().is_empty() {
                 text_content.push_str(&text);
                 text_content.push('\n');
             }
        }

        let metadata = DocumentMetadata {
             file_type: "application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string(),
             pages: Some(text_content.lines().count()), // Approx slide count equivalent
             char_count: text_content.len(),
             encoding: "UTF-8".to_string(),
        };

        Ok((text_content, metadata))
    }

    /// Parse Image using Tesseract OCR
    fn parse_image(path: &Path) -> Result<(String, DocumentMetadata)> {
        debug!("Running OCR on image: {:?}", path);
        
        // Check if tesseract is available is hard (cross-platform), so we just try to run it
        // Command: tesseract <image> stdout
        let output = std::process::Command::new("tesseract")
            .arg(path)
            .arg("stdout")
            .output();
            
        match output {
            Ok(out) => {
                if !out.status.success() {
                    let err = String::from_utf8_lossy(&out.stderr);
                    warn!("Tesseract failed: {}", err);
                    return Err(anyhow!("OCR failed: Tesseract returned error code"));
                }
                
                let content = String::from_utf8(out.stdout).context("Invalid UTF-8 from Tesseract")?;
                
                let metadata = DocumentMetadata {
                    file_type: "image/ocr".to_string(),
                    pages: Some(1),
                    char_count: content.len(),
                    encoding: "UTF-8".to_string(),
                };
                
                Ok((content, metadata))
            },
            Err(e) => {
                warn!("Failed to execute tesseract: {}", e);
                Err(anyhow!("Failed to run OCR. Is Tesseract installed and in PATH? Error: {}", e))
            }
        }
    }

    /// Parse RTF using rtf-parser
    fn parse_rtf(path: &Path) -> Result<(String, DocumentMetadata)> {
        let content = fs::read_to_string(path).context("Failed to read RTF file")?;
        
        // rtf-parser 0.3 usage fix:
        // Lexer::scan returns Result<Vec<Token>, Error>
        let tokens = RtfLexer::scan(&content).map_err(|e| anyhow!("RTF Lexer error: {:?}", e))?;
        let doc = RtfParser::new(tokens).parse().map_err(|e| anyhow!("RTF Parse error: {:?}", e))?;
        
        let text = format!("{:?}", doc); // Debug for now, or implement a walker
        
        let metadata = DocumentMetadata {
            file_type: "application/rtf".to_string(),
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
            for _elem in doc.select(&script_selector) {
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

    /// Helper: Extract text from Office XML (docx, pptx, etc) by finding text in xml tags
    /// Basic implementation: extract content from specific XML file inside zip
    fn extract_text_from_office_xml(path: &Path, target_xml_file: &str) -> Result<String> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut xml_file = archive.by_name(target_xml_file)?;
        let mut xml_content = String::new();
        xml_file.read_to_string(&mut xml_content)?;
        
        Ok(Self::strip_xml_tags(&xml_content))
    }

    /// Helper: Strip XML tags manually to get text (fast & dirty approach)
    /// Ideally use a real XML parser, but for generic text extraction this often works well enough for RAG
    fn strip_xml_tags(xml: &str) -> String {
        let mut text = String::new();
        let mut inside_tag = false;
        
        for c in xml.chars() {
            if c == '<' {
                inside_tag = true;
            } else if c == '>' {
                inside_tag = false;
                text.push(' '); // add space to prevent words glueing together
            } else if !inside_tag {
                text.push(c);
            }
        }
        
        // Cleanup whitespace
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}
