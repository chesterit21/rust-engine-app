use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub content: String,
    pub start_pos: usize,
    pub end_pos: usize,
}

pub struct TextChunker {
    chunk_size: usize,
    overlap: usize,
}

impl TextChunker {
    pub fn new(chunk_size: usize, overlap: usize) -> Self {
        Self {
            chunk_size,
            overlap,
        }
    }
    
    pub fn chunk(&self, text: &str) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let total_len = chars.len();
        
        if total_len == 0 {
            return Ok(chunks);
        }
        
        let mut start = 0;
        
        while start < total_len {
            let end = std::cmp::min(start + self.chunk_size, total_len);
            
            let chunk_content: String = chars[start..end].iter().collect();
            
            chunks.push(Chunk {
                content: chunk_content,
                start_pos: start,
                end_pos: end,
            });
            
            if end >= total_len {
                break;
            }
            
            start += self.chunk_size - self.overlap;
        }
        
        Ok(chunks)
    }
}
