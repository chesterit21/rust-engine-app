use crate::{EventType, Opcode, ProtoError, Status, ValueFormat};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug)]
pub struct SetReq {
    pub format: ValueFormat,
    pub suppress_publish: bool, // flags bit0
    pub key: String,
    pub value: Bytes,
    pub ttl_ms: u64,
}

#[derive(Debug)]
pub struct GetReq {
    pub key: String,
}

#[derive(Debug)]
pub struct DelReq {
    pub key: String,
}

#[derive(Debug)]
pub struct SubscribeReq {
    pub topic: String,
}

#[derive(Debug, Clone)]
pub struct PushEvent {
    pub event_type: EventType,
    pub topic: String,
    pub key: String,
    pub ts_ms: u64,
}

pub fn encode_request(op: Opcode, payload: &[u8]) -> BytesMut {
    // frame: [u32 len][u8 opcode][payload]
    let len = 1 + payload.len();
    let mut out = BytesMut::with_capacity(4 + len);
    out.put_u32_le(len as u32);
    out.put_u8(op as u8);
    out.extend_from_slice(payload);
    out
}

pub fn encode_response(status: Status, payload: &[u8]) -> BytesMut {
    let len = 1 + payload.len();
    let mut out = BytesMut::with_capacity(4 + len);
    out.put_u32_le(len as u32);
    out.put_u8(status as u8);
    out.extend_from_slice(payload);
    out
}

pub fn decode_set_payload(mut p: &[u8]) -> Result<SetReq, ProtoError> {
    if p.remaining() < 1 + 1 + 2 {
        return Err(ProtoError::BadPayload);
    }
    let fmt = p.get_u8();
    let flags = p.get_u8();
    let format = match fmt {
        1 => ValueFormat::Json,
        2 => ValueFormat::MsgPack,
        _ => return Err(ProtoError::UnsupportedFormat),
    };
    let suppress_publish = (flags & 0b0000_0001) != 0;

    let key_len = p.get_u16_le() as usize;
    if p.remaining() < key_len + 4 + 8 {
        return Err(ProtoError::BadPayload);
    }
    let key_bytes = &p[..key_len];
    let key = std::str::from_utf8(key_bytes)
        .map_err(|_| ProtoError::InvalidUtf8)?
        .to_string();
    p.advance(key_len);

    let val_len = p.get_u32_le() as usize;
    if val_len == 0 || p.remaining() < val_len + 8 {
        return Err(ProtoError::BadPayload);
    }
    let val = Bytes::copy_from_slice(&p[..val_len]);
    p.advance(val_len);

    let ttl_ms = p.get_u64_le();
    Ok(SetReq {
        format,
        suppress_publish,
        key,
        value: val,
        ttl_ms,
    })
}

pub fn encode_set_payload(
    format: ValueFormat,
    suppress_publish: bool,
    key: &str,
    value: &[u8],
    ttl_ms: u64,
) -> BytesMut {
    let flags = if suppress_publish { 1u8 } else { 0u8 };
    let mut out = BytesMut::with_capacity(1 + 1 + 2 + key.len() + 4 + value.len() + 8);
    out.put_u8(format as u8);
    out.put_u8(flags);
    out.put_u16_le(key.len() as u16);
    out.extend_from_slice(key.as_bytes());
    out.put_u32_le(value.len() as u32);
    out.extend_from_slice(value);
    out.put_u64_le(ttl_ms);
    out
}

pub fn decode_key_only(mut p: &[u8]) -> Result<String, ProtoError> {
    if p.remaining() < 2 {
        return Err(ProtoError::BadPayload);
    }
    let klen = p.get_u16_le() as usize;
    if p.remaining() < klen {
        return Err(ProtoError::BadPayload);
    }
    let key = std::str::from_utf8(&p[..klen])
        .map_err(|_| ProtoError::InvalidUtf8)?
        .to_string();
    Ok(key)
}

pub fn encode_key_only(key: &str) -> BytesMut {
    let mut out = BytesMut::with_capacity(2 + key.len());
    out.put_u16_le(key.len() as u16);
    out.extend_from_slice(key.as_bytes());
    out
}

pub fn decode_subscribe_payload(mut p: &[u8]) -> Result<String, ProtoError> {
    if p.remaining() < 2 {
        return Err(ProtoError::BadPayload);
    }
    let tlen = p.get_u16_le() as usize;
    if p.remaining() < tlen {
        return Err(ProtoError::BadPayload);
    }
    let topic = std::str::from_utf8(&p[..tlen])
        .map_err(|_| ProtoError::InvalidUtf8)?
        .to_string();
    Ok(topic)
}

pub fn encode_subscribe_payload(topic: &str) -> BytesMut {
    let mut out = BytesMut::with_capacity(2 + topic.len());
    out.put_u16_le(topic.len() as u16);
    out.extend_from_slice(topic.as_bytes());
    out
}

pub fn decode_push_event_payload(mut p: &[u8]) -> Result<PushEvent, ProtoError> {
    if p.remaining() < 1 + 2 {
        return Err(ProtoError::BadPayload);
    }
    let et = p.get_u8();
    let event_type = match et {
        1 => EventType::Invalidate,
        2 => EventType::TableChanged,
        _ => return Err(ProtoError::BadPayload),
    };
    let tlen = p.get_u16_le() as usize;
    if p.remaining() < tlen + 2 {
        return Err(ProtoError::BadPayload);
    }
    let topic = std::str::from_utf8(&p[..tlen])
        .map_err(|_| ProtoError::InvalidUtf8)?
        .to_string();
    p.advance(tlen);

    let klen = p.get_u16_le() as usize;
    if p.remaining() < klen + 8 {
        return Err(ProtoError::BadPayload);
    }
    let key = std::str::from_utf8(&p[..klen])
        .map_err(|_| ProtoError::InvalidUtf8)?
        .to_string();
    p.advance(klen);

    let ts_ms = p.get_u64_le();
    Ok(PushEvent {
        event_type,
        topic,
        key,
        ts_ms,
    })
}

pub fn encode_push_event_payload(ev: &PushEvent) -> BytesMut {
    let mut out = BytesMut::with_capacity(1 + 2 + ev.topic.len() + 2 + ev.key.len() + 8);
    out.put_u8(ev.event_type as u8);
    out.put_u16_le(ev.topic.len() as u16);
    out.extend_from_slice(ev.topic.as_bytes());
    out.put_u16_le(ev.key.len() as u16);
    out.extend_from_slice(ev.key.as_bytes());
    out.put_u64_le(ev.ts_ms);
    out
}
