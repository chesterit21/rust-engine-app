use crate::ProtoError;
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, Copy)]
pub struct StatsV1 {
    pub uptime_ms: u64,
    pub keys_count: u64,
    pub approx_mem_bytes: u64,    // Memory used by cache
    pub mem_available_bytes: u64, // Available RAM on system (from MemAvailable)
    pub evictions_total: u64,
    pub pubsub_topics: u64,
    pub events_published_total: u64,
    pub events_lagged_total: u64,
    pub invalid_key_total: u64,
    pub hits_total: u64,        // Cache hits
    pub misses_total: u64,      // Cache misses
    pub mem_pressure_bp: u16,   // System memory pressure (basis points, 0-10000)
    pub pressure_limit_bp: u16, // Current limit setting (basis points, default 8500 = 85%)
}

pub fn encode_stats_v1(s: &StatsV1) -> BytesMut {
    let mut out = BytesMut::with_capacity(1 + 11 * 8 + 2 + 2);
    out.put_u8(1); // stats_version
    out.put_u64_le(s.uptime_ms);
    out.put_u64_le(s.keys_count);
    out.put_u64_le(s.approx_mem_bytes);
    out.put_u64_le(s.mem_available_bytes);
    out.put_u64_le(s.evictions_total);
    out.put_u64_le(s.pubsub_topics);
    out.put_u64_le(s.events_published_total);
    out.put_u64_le(s.events_lagged_total);
    out.put_u64_le(s.invalid_key_total);
    out.put_u64_le(s.hits_total);
    out.put_u64_le(s.misses_total);
    out.put_u16_le(s.mem_pressure_bp);
    out.put_u16_le(s.pressure_limit_bp);
    out
}

pub fn decode_stats_v1(mut p: &[u8]) -> Result<StatsV1, ProtoError> {
    if p.remaining() < 1 {
        return Err(ProtoError::BadPayload);
    }
    let ver = p.get_u8();
    if ver != 1 {
        return Err(ProtoError::BadPayload);
    }
    if p.remaining() < 11 * 8 + 2 + 2 {
        return Err(ProtoError::BadPayload);
    }

    Ok(StatsV1 {
        uptime_ms: p.get_u64_le(),
        keys_count: p.get_u64_le(),
        approx_mem_bytes: p.get_u64_le(),
        mem_available_bytes: p.get_u64_le(),
        evictions_total: p.get_u64_le(),
        pubsub_topics: p.get_u64_le(),
        events_published_total: p.get_u64_le(),
        events_lagged_total: p.get_u64_le(),
        invalid_key_total: p.get_u64_le(),
        hits_total: p.get_u64_le(),
        misses_total: p.get_u64_le(),
        mem_pressure_bp: p.get_u16_le(),
        pressure_limit_bp: p.get_u16_le(),
    })
}
