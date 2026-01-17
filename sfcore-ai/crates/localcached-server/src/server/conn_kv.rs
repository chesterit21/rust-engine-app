use tokio::net::UnixStream;
use tokio::io::AsyncWriteExt;
use bytes::BytesMut;
use localcached_proto::*;
use crate::server::Context;
use crate::framing::read_frame;
use crate::time::now_ms;

pub async fn handle_conn(stream: &mut UnixStream, ctx: &Context) -> anyhow::Result<()> {
    let mut buf = BytesMut::with_capacity(4096);

    loop {
        let frame = match read_frame(stream, ctx.cfg.max_frame_bytes, &mut buf).await {
            Ok(Some(f)) => f,
            Ok(None) => return Ok(()), // EOF
            Err(e) => return Err(e.into()),
        };

        // Backpressure: acquire semaphore permit before processing
        let _permit = ctx.op_semaphore.acquire().await
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Semaphore closed"))?;

        if frame.len() < 5 { break; } // Should adhere to frame struct
        
        // frame: [u32 len][u8 opcode][payload...]
        // read_frame returns full block [len][opcode][payload]
        // But we need to skip len(4) bytes
        let opcode_byte = frame[4];
        let payload = &frame[5..];

        match opcode_byte {
            0x01 => { // SET
                 handle_set(stream, payload, ctx).await?;
            }
            0x02 => { // GET
                 handle_get(stream, payload, ctx).await?;
            }
            0x03 => { // DEL
                 handle_del(stream, payload, ctx).await?;
            }
            0x04 => { // PING
                 let resp = encode_response(Status::Ok, &[]);
                 stream.write_all(&resp).await?;
            }
            0x05 => { // STATS
                 handle_stats(stream, ctx).await?;
            }
            0x06 => { // KEYS
                 handle_keys(stream, payload, ctx).await?;
            }
            0x07 => { // SET_CONFIG
                 handle_set_config(stream, payload, ctx).await?;
            }
            0x20 => { // SUBSCRIBE
                 // Switch to Sub mode!
                 // This function will TAKE OVER the stream and never return until disconnect.
                 crate::server::conn_sub::handle_sub_mode(stream, payload, ctx).await?;
                 return Ok(()); // Connection closed after sub loop
            }
            _ => {
                 let resp = encode_response(Status::ErrInternal, &[]); // Unknown Op
                 stream.write_all(&resp).await?;
            }
        }
    }
    Ok(())
}

async fn handle_set(s: &mut UnixStream, payload: &[u8], ctx: &Context) -> std::io::Result<()> {
    let req = match decode_set_payload(payload) {
        Ok(r) => r,
        Err(e) => {
            let st = match e {
               ProtoError::UnsupportedFormat => Status::ErrUnsupportedFormat,
               _ => Status::ErrBadPayload,
            };
            let resp = encode_response(st, &[]);
            s.write_all(&resp).await?;
            return Ok(());
        }
    };

    // Validation Key
    // Spec: "svc:table:pk"
    let parts = localcached_proto::validate_key_3parts(&req.key);
    if parts.is_err() {
        ctx.metrics.inc_invalid_key();
        let resp = encode_response(Status::ErrInvalidKeyFormat, &[]);
        s.write_all(&resp).await?;
        return Ok(());
    }

    let now = now_ms();
    let expires = if req.ttl_ms > 0 { now + req.ttl_ms } else { 0 };

    ctx.kv.set(req.key.clone(), req.format, req.value, expires, now);
    ctx.evictor.on_write(&req.key);

    // Auto publish
    if !req.suppress_publish {
         if let Ok(topic) = localcached_proto::topic_from_key(&req.key) {
             let msg = PushEvent {
                 event_type: EventType::TableChanged, // implicit upsert
                 topic: topic.clone(),
                 key: req.key,
                 ts_ms: now,
             };
             ctx.pubsub.publish(&topic, msg);
         }
    }

    let resp = encode_response(Status::Ok, &[]);
    s.write_all(&resp).await?;
    Ok(())
}

async fn handle_get(s: &mut UnixStream, payload: &[u8], ctx: &Context) -> std::io::Result<()> {
    let key = match decode_key_only(payload) {
        Ok(k) => k,
        Err(_) => {
            let resp = encode_response(Status::ErrBadPayload, &[]);
            s.write_all(&resp).await?;
            return Ok(());
        }
    };

    let now = now_ms();
    match ctx.kv.get(&key, now) {
        Some((fmt, val, ttl)) => {
            ctx.metrics.inc_hit();  // Track cache hit
            // Encode OK + payload
            // Custom encoding for GET response: [u8 format][u32 val_len][val][u64 ttl]
            let mut out = BytesMut::with_capacity(1 + 4 + val.len() + 8);
            use bytes::BufMut;
            out.put_u8(fmt as u8);
            out.put_u32_le(val.len() as u32);
            out.extend_from_slice(&val);
            out.put_u64_le(ttl);
            
            let resp = encode_response(Status::Ok, &out);
            s.write_all(&resp).await?;
        }
        None => {
            ctx.metrics.inc_miss();  // Track cache miss
            let resp = encode_response(Status::NotFound, &[]);
            s.write_all(&resp).await?;
        }
    }
    Ok(())
}

async fn handle_del(s: &mut UnixStream, payload: &[u8], ctx: &Context) -> std::io::Result<()> {
    let key = match decode_key_only(payload) {
        Ok(k) => k,
        Err(_) => {
            let resp = encode_response(Status::ErrBadPayload, &[]);
            s.write_all(&resp).await?;
            return Ok(());
        }
    };

    let existed = ctx.kv.del(&key);
    
    // Auto publish invalidate if it existed (or always? usually only if it existed to reduce noise)
    // Spec says: "default: SET/DEL publish table_changed". 
    // And "invalidate used if needed". 
    // Spec 5) "SET/DEL parse table -> publish table_changed".
    // Spec 4.3) "Invalidate: 1, TableChanged: 2".
    
    // Let's stick to "TableChanged" for DEL as well?
    // Wait, DEL usually means 'data removed'. TableChanged(upsert) implies data is there.
    // If we use TableChanged for DEL, client might re-fetch and find nothing (NotFound). That is acceptable invalidation.
    // OR we can send Invalidate event.
    // Spec says: "Invalidate ... for key_full".
    // Let's send Invalidate for DEL.
    
    if existed {
         if let Ok(topic) = localcached_proto::topic_from_key(&key) {
             let msg = PushEvent {
                 event_type: EventType::Invalidate,
                 topic: topic.clone(),
                 key,
                 ts_ms: now_ms(),
             };
             ctx.pubsub.publish(&topic, msg);
         }
         let resp = encode_response(Status::Ok, &[]);
         s.write_all(&resp).await?;
    } else {
         let resp = encode_response(Status::NotFound, &[]);
         s.write_all(&resp).await?;
    }
    Ok(())
}

async fn handle_stats(s: &mut UnixStream, ctx: &Context) -> std::io::Result<()> {
    use crate::sys::meminfo::{read_meminfo, pressure_bp};
    let mi = read_meminfo().unwrap_or(crate::sys::meminfo::MemInfo { mem_total_kb: 0, mem_available_kb: 0 });
    
    let stats = localcached_proto::StatsV1 {
        uptime_ms: ctx.metrics.uptime_ms(),
        keys_count: ctx.kv.len(),
        approx_mem_bytes: ctx.kv.approx_mem_bytes(),
        mem_available_bytes: mi.mem_available_kb * 1024,  // Convert KB to bytes
        evictions_total: ctx.metrics.evictions_total.load(std::sync::atomic::Ordering::Relaxed),
        pubsub_topics: ctx.pubsub.topic_count(),
        events_published_total: ctx.metrics.events_published_total.load(std::sync::atomic::Ordering::Relaxed),
        events_lagged_total: ctx.metrics.events_lagged_total.load(std::sync::atomic::Ordering::Relaxed),
        invalid_key_total: ctx.metrics.invalid_key_total.load(std::sync::atomic::Ordering::Relaxed),
        hits_total: ctx.metrics.hits_total.load(std::sync::atomic::Ordering::Relaxed),
        misses_total: ctx.metrics.misses_total.load(std::sync::atomic::Ordering::Relaxed),
        mem_pressure_bp: pressure_bp(mi),
        pressure_limit_bp: ctx.runtime_cfg.get_pressure_hot_bp(),
    };
    
    let payload = encode_stats_v1(&stats);
    let resp = encode_response(Status::Ok, &payload);
    s.write_all(&resp).await?;

    Ok(())
}

/// Maximum allowed limit is 85% (8500 basis points) - server enforced
const MAX_PRESSURE_LIMIT_BP: u16 = 8500;

async fn handle_set_config(s: &mut UnixStream, payload: &[u8], ctx: &Context) -> std::io::Result<()> {
    // Payload format: [u8 config_type][u16 value]
    // config_type:
    //   0x01 = pressure_limit_bp (memory limit in basis points)
    
    if payload.len() < 3 {
        let resp = encode_response(Status::ErrBadPayload, &[]);
        s.write_all(&resp).await?;
        return Ok(());
    }

    let config_type = payload[0];
    let value = u16::from_le_bytes([payload[1], payload[2]]);

    match config_type {
        0x01 => {
            // Set pressure limit
            // Validation: max 85% (8500 bp)
            if value > MAX_PRESSURE_LIMIT_BP {
                // Return error with max allowed value
                let mut err_payload = vec![0x01u8]; // Error type: value too high
                err_payload.extend_from_slice(&MAX_PRESSURE_LIMIT_BP.to_le_bytes());
                let resp = encode_response(Status::ErrBadPayload, &err_payload);
                s.write_all(&resp).await?;
                return Ok(());
            }

            let old_value = ctx.runtime_cfg.set_pressure_hot_bp(value);
            
            // If new limit is lower than current cache usage, trigger eviction
            let mi = crate::sys::meminfo::read_meminfo().unwrap_or(
                crate::sys::meminfo::MemInfo { mem_total_kb: 0, mem_available_kb: 0 }
            );
            let available_bytes = mi.mem_available_kb * 1024;
            let new_limit_bytes = (available_bytes as f64 * (value as f64 / 10000.0)) as u64;
            
            if ctx.kv.approx_mem_bytes() > new_limit_bytes {
                // Force eviction to meet new target
                let evicted = ctx.evictor.force_evict_to_target(value);
                tracing::info!("SET_CONFIG: limit {} -> {}, evicted {} keys", old_value, value, evicted);
            }

            // Response: [u16 old_value][u16 new_value]
            let mut resp_payload = Vec::with_capacity(4);
            resp_payload.extend_from_slice(&old_value.to_le_bytes());
            resp_payload.extend_from_slice(&value.to_le_bytes());
            
            let resp = encode_response(Status::Ok, &resp_payload);
            s.write_all(&resp).await?;
        }
        _ => {
            let resp = encode_response(Status::ErrBadPayload, &[]);
            s.write_all(&resp).await?;
        }
    }

    Ok(())
}

async fn handle_keys(s: &mut UnixStream, payload: &[u8], ctx: &Context) -> std::io::Result<()> {
    // Payload: [u16 prefix_len][prefix_bytes]
    // Empty prefix = list all keys
    let prefix = if payload.len() < 2 {
        String::new()
    } else {
        let plen = u16::from_le_bytes([payload[0], payload[1]]) as usize;
        if payload.len() < 2 + plen {
            let resp = encode_response(Status::ErrBadPayload, &[]);
            s.write_all(&resp).await?;
            return Ok(());
        }
        String::from_utf8_lossy(&payload[2..2 + plen]).to_string()
    };

    let now = now_ms();
    let keys = ctx.kv.keys(&prefix, now);

    // Response: [u32 count][{u16 key_len, key_bytes}...]
    let mut out = BytesMut::with_capacity(4 + keys.len() * 64);
    use bytes::BufMut;
    out.put_u32_le(keys.len() as u32);
    for key in keys {
        let kb = key.as_bytes();
        out.put_u16_le(kb.len() as u16);
        out.extend_from_slice(kb);
    }

    let resp = encode_response(Status::Ok, &out);
    s.write_all(&resp).await?;
    Ok(())
}
