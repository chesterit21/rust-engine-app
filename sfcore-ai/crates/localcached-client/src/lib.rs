use localcached_proto::{
    encode_request,
    types::{Opcode, Status},
    ProtoError,
};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Cache statistics from the server
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub uptime_ms: u64,
    pub keys_count: u64,
    pub cache_mem_bytes: u64,       // Memory used by cache
    pub available_mem_bytes: u64,   // Available RAM on system
    pub evictions_total: u64,
    pub hits_total: u64,            // Cache hits
    pub misses_total: u64,          // Cache misses
    pub system_pressure_bp: u16,    // System-wide memory pressure (0-10000)
    pub memory_limit_bp: u16,       // Current cache limit (0-8500)
}

impl CacheStats {
    /// Cache memory usage as percentage of available RAM
    pub fn cache_usage_percent(&self) -> f64 {
        if self.available_mem_bytes == 0 {
            return 0.0;
        }
        (self.cache_mem_bytes as f64 / self.available_mem_bytes as f64) * 100.0
    }

    /// Current memory limit as percentage
    pub fn memory_limit_percent(&self) -> u8 {
        (self.memory_limit_bp / 100) as u8
    }

    /// Format cache memory as human-readable string
    pub fn cache_mem_human(&self) -> String {
        format_bytes(self.cache_mem_bytes)
    }

    /// Format available memory as human-readable string
    pub fn available_mem_human(&self) -> String {
        format_bytes(self.available_mem_bytes)
    }

    /// Calculate cache hit rate (0.0 - 100.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits_total + self.misses_total;
        if total == 0 {
            0.0
        } else {
            (self.hits_total as f64 / total as f64) * 100.0
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Result of set_memory_limit operation
#[derive(Debug, Clone)]
pub enum SetLimitResult {
    Success { old_percent: u8, new_percent: u8 },
    TooHigh { max_percent: u8 },
}

pub struct Client {
    stream: UnixStream,
}

impl Client {
    /// Connect to the server at the given socket path
    pub async fn connect<P: AsRef<Path>>(path: P) -> Result<Self, ProtoError> {
        let stream = UnixStream::connect(path).await?;
        Ok(Self { stream })
    }

    /// Set a key-value pair with TTL in milliseconds
    pub async fn set(&mut self, key: &str, value: Vec<u8>, ttl_ms: u64) -> Result<(), ProtoError> {
        let mut payload = Vec::with_capacity(1 + 1 + 2 + key.len() + 4 + value.len() + 8);
        
        // fmt=1 (JSON default for simplicity), flags=0
        payload.push(1); 
        payload.push(0);

        // Key Len + Key
        payload.extend_from_slice(&(key.len() as u16).to_le_bytes());
        payload.extend_from_slice(key.as_bytes());

        // Val Len + Val
        payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
        payload.extend_from_slice(&value);

        // TTL
        payload.extend_from_slice(&ttl_ms.to_le_bytes());

        self.send_frame(Opcode::Set, &payload).await?;
        self.expect_ok().await
    }

    /// Get a value by key
    pub async fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>, ProtoError> {
        let mut payload = Vec::with_capacity(2 + key.len());
        payload.extend_from_slice(&(key.len() as u16).to_le_bytes());
        payload.extend_from_slice(key.as_bytes());

        self.send_frame(Opcode::Get, &payload).await?;

        let (status, body) = self.read_response().await?;
        
        if status == Status::NotFound {
            return Ok(None);
        }
        if status != Status::Ok {
            return Err(ProtoError::ServerError(format!("{:?}", status)));
        }

        // Body: [fmt][vlen][val][ttl]
        if body.len() < 5 {
            return Err(ProtoError::InvalidFrame("Body too short".into()));
        }

        let _fmt = body[0];
        let vlen = u32::from_le_bytes([body[1], body[2], body[3], body[4]]) as usize;
        if body.len() < 5 + vlen {
            return Err(ProtoError::InvalidFrame("Val mismatch".into()));
        }

        let val = body[5..5 + vlen].to_vec();
        Ok(Some(val))
    }

    /// Delete a key
    pub async fn del(&mut self, key: &str) -> Result<(), ProtoError> {
        let mut payload = Vec::with_capacity(2 + key.len());
        payload.extend_from_slice(&(key.len() as u16).to_le_bytes());
        payload.extend_from_slice(key.as_bytes());

        self.send_frame(Opcode::Del, &payload).await?;
        
        let (status, _) = self.read_response().await?;
        // NotFound is also a valid response for del (key didn't exist)
        if status == Status::Ok || status == Status::NotFound {
            Ok(())
        } else {
            Err(ProtoError::ServerError(format!("{:?}", status)))
        }
    }

    /// List all keys with optional prefix filter (empty string = all keys)
    pub async fn keys(&mut self, prefix: &str) -> Result<Vec<String>, ProtoError> {
        let mut payload = Vec::with_capacity(2 + prefix.len());
        payload.extend_from_slice(&(prefix.len() as u16).to_le_bytes());
        payload.extend_from_slice(prefix.as_bytes());

        self.send_frame(Opcode::Keys, &payload).await?;

        let (status, body) = self.read_response().await?;
        
        if status != Status::Ok {
            return Err(ProtoError::ServerError(format!("{:?}", status)));
        }

        // Parse: [u32 count][{u16 key_len, key_bytes}...]
        if body.len() < 4 {
            return Err(ProtoError::InvalidFrame("Keys response too short".into()));
        }

        let count = u32::from_le_bytes([body[0], body[1], body[2], body[3]]) as usize;
        let mut keys = Vec::with_capacity(count);
        let mut offset = 4;

        for _ in 0..count {
            if offset + 2 > body.len() {
                return Err(ProtoError::InvalidFrame("Keys truncated".into()));
            }
            let klen = u16::from_le_bytes([body[offset], body[offset + 1]]) as usize;
            offset += 2;

            if offset + klen > body.len() {
                return Err(ProtoError::InvalidFrame("Key data truncated".into()));
            }
            let key = String::from_utf8_lossy(&body[offset..offset + klen]).to_string();
            keys.push(key);
            offset += klen;
        }

        Ok(keys)
    }

    /// Clear all keys (delete everything)
    pub async fn clear_all(&mut self) -> Result<usize, ProtoError> {
        let keys = self.keys("").await?;
        let count = keys.len();
        for key in keys {
            // Ignore errors (key might have expired between list and delete)
            let _ = self.del(&key).await;
        }
        Ok(count)
    }

    /// Get server statistics
    pub async fn stats(&mut self) -> Result<CacheStats, ProtoError> {
        self.send_frame(Opcode::Stats, &[]).await?;

        let (status, body) = self.read_response().await?;
        
        if status != Status::Ok {
            return Err(ProtoError::ServerError(format!("{:?}", status)));
        }

        // Parse StatsV1 format
        // [u8 version][u64 uptime_ms][u64 keys_count][u64 approx_mem_bytes][u64 mem_available_bytes]
        // [u64 evictions][u64 topics][u64 events_pub][u64 events_lag][u64 invalid_key]
        // [u64 hits_total][u64 misses_total]
        // [u16 pressure_bp][u16 limit_bp]
        if body.len() < 1 + 11 * 8 + 2 + 2 {
            return Err(ProtoError::InvalidFrame("Stats too short".into()));
        }

        let _version = body[0];
        let mut offset = 1;

        let uptime_ms = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let keys_count = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let cache_mem_bytes = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let available_mem_bytes = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let evictions_total = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let _topics = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let _events_pub = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let _events_lag = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let _invalid_key = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let hits_total = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let misses_total = u64::from_le_bytes(body[offset..offset+8].try_into().unwrap());
        offset += 8;
        let system_pressure_bp = u16::from_le_bytes(body[offset..offset+2].try_into().unwrap());
        offset += 2;
        let memory_limit_bp = u16::from_le_bytes(body[offset..offset+2].try_into().unwrap());

        Ok(CacheStats {
            uptime_ms,
            keys_count,
            cache_mem_bytes,
            available_mem_bytes,
            evictions_total,
            hits_total,
            misses_total,
            system_pressure_bp,
            memory_limit_bp,
        })
    }

    /// Set memory limit (percentage of available RAM for cache)
    /// limit_percent: 1-85 (max 85%)
    /// Returns Ok((old_pct, new_pct)) on success
    /// Returns Err with max allowed if limit is too high
    pub async fn set_memory_limit(&mut self, limit_percent: u8) -> Result<SetLimitResult, ProtoError> {
        let limit_bp = (limit_percent as u16) * 100;

        // Client-side validation
        if limit_percent > 85 {
            return Ok(SetLimitResult::TooHigh { max_percent: 85 });
        }
        if limit_percent == 0 {
            return Err(ProtoError::InvalidFrame("Limit must be at least 1%".into()));
        }

        // Payload: [u8 config_type=0x01][u16 value_bp]
        let mut payload = vec![0x01u8];
        payload.extend_from_slice(&limit_bp.to_le_bytes());

        self.send_frame(Opcode::SetConfig, &payload).await?;

        let (status, body) = self.read_response().await?;

        if status == Status::ErrBadPayload && body.len() >= 3 && body[0] == 0x01 {
            // Server rejected: value too high
            let max_bp = u16::from_le_bytes([body[1], body[2]]);
            return Ok(SetLimitResult::TooHigh { max_percent: (max_bp / 100) as u8 });
        }

        if status != Status::Ok {
            return Err(ProtoError::ServerError(format!("{:?}", status)));
        }

        // Response: [u16 old_bp][u16 new_bp]
        if body.len() < 4 {
            return Err(ProtoError::InvalidFrame("SetConfig response too short".into()));
        }

        let old_bp = u16::from_le_bytes([body[0], body[1]]);
        let new_bp = u16::from_le_bytes([body[2], body[3]]);

        Ok(SetLimitResult::Success {
            old_percent: (old_bp / 100) as u8,
            new_percent: (new_bp / 100) as u8,
        })
    }

    // --- Helpers ---

    async fn send_frame(&mut self, opcode: Opcode, payload: &[u8]) -> Result<(), ProtoError> {
        let frame = encode_request(opcode, payload);
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn read_response(&mut self) -> Result<(Status, Vec<u8>), ProtoError> {
        // Read [u32 len]
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_le_bytes(len_buf) as usize;

        if len == 0 {
            return Err(ProtoError::InvalidFrame("Empty response".into()));
        }

        // Read body [status][payload...]
        let mut body = vec![0u8; len];
        self.stream.read_exact(&mut body).await?;

        let status = Status::from(body[0]);
        let payload = body[1..].to_vec();

        Ok((status, payload))
    }

    async fn expect_ok(&mut self) -> Result<(), ProtoError> {
        let (status, _) = self.read_response().await?;
        match status {
            Status::Ok => Ok(()),
            Status::NotFound => Err(ProtoError::NotFound),
            _ => Err(ProtoError::ServerError(format!("{:?}", status))),
        }
    }
}
