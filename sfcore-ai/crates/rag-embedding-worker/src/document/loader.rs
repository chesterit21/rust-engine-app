use anyhow::Result;
use mime_guess;
use mime_guess::mime;
use std::fs;
use std::path::Path;
use tracing::debug;

pub struct DocumentLoader;

impl DocumentLoader {
    /// Load file content from path
    pub fn load_file(path: &Path) -> Result<Vec<u8>> {
        if !path.exists() {
            anyhow::bail!("File not found: {:?}", path);
        }
        
        if !path.is_file() {
            anyhow::bail!("Path is not a file: {:?}", path);
        }
        
        let content = fs::read(path)?;
        debug!("Loaded file: {:?} ({} bytes)", path, content.len());
        
        Ok(content)
    }
    
    /// Detect file type from path
    pub fn detect_file_type(path: &Path) -> Result<String> {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let file_type = mime.essence_str().to_string();
        
        debug!("Detected file type: {} for {:?}", file_type, path);
        
        Ok(file_type)
    }
    
    /// Check if file is supported for text extraction
    pub fn is_supported(path: &Path) -> bool {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());
        
        match extension.as_deref() {
            // Documents
            Some("txt") | Some("md") | Some("pdf") | Some("docx") | Some("doc") => true,
            Some("pptx") | Some("ppt") | Some("xlsx") | Some("xls") | Some("rtf") => true,
            
            // Images (OCR)
            Some("png") | Some("jpg") | Some("jpeg") | Some("tiff") | Some("bmp") => true,
            
            // Web
            Some("html") | Some("htm") | Some("xml") => true,
            
            // Code
            Some("rs") | Some("py") | Some("js") | Some("ts") | Some("java") |
            Some("c") | Some("cpp") | Some("cs") | Some("go") | Some("rb") |
            Some("php") | Some("swift") | Some("kt") => true,
            
            // Config/Data
            Some("json") | Some("yaml") | Some("yml") | Some("toml") |
            Some("csv") | Some("sql") => true,
            
            // Other text
            Some("log") | Some("sh") | Some("bash") | Some("css") => true,
            
            _ => {
                // Check MIME type as fallback
                let mime = mime_guess::from_path(path).first();
                match mime {
                    Some(m) if m.type_() == mime::TEXT => true,
                    _ => false,
                }
            }
        }
    }
    
    /// Validate file before processing
    pub fn validate_file(path: &Path, max_size_mb: u64) -> Result<()> {
        if !path.exists() {
            anyhow::bail!("File not found: {:?}", path);
        }
        
        if !Self::is_supported(path) {
            anyhow::bail!("Unsupported file type: {:?}", path);
        }
        
        let metadata = fs::metadata(path)?;
        let size_mb = metadata.len() / 1024 / 1024;
        
        if size_mb > max_size_mb {
            anyhow::bail!(
                "File too large: {} MB (max: {} MB)",
                size_mb,
                max_size_mb
            );
        }
        
        Ok(())
    }
}