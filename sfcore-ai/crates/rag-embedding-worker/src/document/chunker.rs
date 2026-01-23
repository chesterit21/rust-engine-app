use crate::config::ChunkStrategy;
use anyhow::Result;
use text_splitter::{ChunkConfig, TextSplitter};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub index: usize,
    pub content: String,
    pub char_count: usize,
    pub token_count: Option<usize>,
}

pub struct TextChunker {
    chunk_size: usize,
    chunk_overlap: usize,
    strategy: ChunkStrategy,
}

impl TextChunker {
    pub fn new(chunk_size: usize, chunk_overlap: usize, strategy: ChunkStrategy) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            strategy,
        }
    }
    
    /// Chunk text into smaller pieces
    pub fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        
        debug!("Chunking text: {} chars", text.len());
        
        let chunks = match self.strategy {
            ChunkStrategy::Semantic => self.chunk_semantic(text)?,
            ChunkStrategy::Fixed => self.chunk_fixed(text)?,
            ChunkStrategy::Recursive => self.chunk_recursive(text)?,
        };
        
        debug!("Created {} chunks", chunks.len());
        
        Ok(chunks)
    }
    
    /// Semantic chunking (best untuk RAG)
    fn chunk_semantic(&self, text: &str) -> Result<Vec<Chunk>> {
        let splitter = TextSplitter::new(
            ChunkConfig::new(self.chunk_size)
                .with_overlap(self.chunk_overlap)
                .unwrap()
        );
        
        let chunk_texts: Vec<&str> = splitter.chunks(text).collect();
        
        let chunks = chunk_texts
            .into_iter()
            .enumerate()
            .map(|(i, content)| Chunk {
                index: i,
                content: content.to_string(),
                char_count: content.len(),
                token_count: None, // bisa di-calculate kalau perlu
            })
            .collect();
        
        Ok(chunks)
    }
    
    /// Fixed size chunking
    fn chunk_fixed(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_chars = chars.len();
        
        let mut start = 0;
        let mut index = 0;
        
        while start < total_chars {
            let end = (start + self.chunk_size).min(total_chars);
            let content: String = chars[start..end].iter().collect();
            
            chunks.push(Chunk {
                index,
                content,
                char_count: end - start,
                token_count: None,
            });
            
            index += 1;
            start += self.chunk_size - self.chunk_overlap;
            
            if start >= total_chars {
                break;
            }
        }
        
        Ok(chunks)
    }
    
    /// Recursive character splitting
    fn chunk_recursive(&self, text: &str) -> Result<Vec<Chunk>> {
        // Split by paragraphs first
        let paragraphs: Vec<&str> = text
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .collect();
        
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut index = 0;
        
        for para in paragraphs {
            // If adding this paragraph exceeds chunk size, save current chunk
            if !current_chunk.is_empty() 
                && current_chunk.len() + para.len() > self.chunk_size 
            {
                chunks.push(Chunk {
                    index,
                    content: current_chunk.clone(),
                    char_count: current_chunk.len(),
                    token_count: None,
                });
                
                index += 1;
                
                // Start new chunk with overlap dari chunk sebelumnya
                let overlap_chars: String = current_chunk
                    .chars()
                    .rev()
                    .take(self.chunk_overlap)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                
                current_chunk = overlap_chars;
            }
            
            current_chunk.push_str(para);
            current_chunk.push_str("\n\n");
        }
        
        // Add last chunk
        if !current_chunk.is_empty() {
            chunks.push(Chunk {
                index,
                content: current_chunk.clone(),
                char_count: current_chunk.len(),
                token_count: None,
            });
        }
        
        Ok(chunks)
    }
}