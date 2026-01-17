use tokio::net::UnixStream;
use tokio::io::AsyncWriteExt;
use localcached_proto::*;
use crate::server::Context;
use crate::framing::read_frame;
use bytes::BytesMut;

pub async fn handle_sub_mode(s: &mut UnixStream, initial_payload: &[u8], ctx: &Context) -> anyhow::Result<()> {
    // 1. Decode first SUBSCRIBE
    let topic = match decode_subscribe_payload(initial_payload) {
        Ok(t) => t,
        Err(_) => {
            let resp = encode_response(Status::ErrBadPayload, &[]);
            s.write_all(&resp).await?;
            return Ok(());
        }
    };
    
    // Ack OK
    let resp = encode_response(Status::Ok, &[]);
    s.write_all(&resp).await?;

    // 2. Subscribe
    let mut rx = ctx.pubsub.subscribe(&topic);

    // 3. Loop: Select between Incoming commands (Frame) OR Incoming Event (Rx)
    // In Sub mode, client *might* send Unsubscribe or Subscribe (other topic? - Spec v1 only 1 connection per stream recommended, but let's see).
    // Spec says Conn-B is "Subscription stream". 
    // Usually Redis PubSub allows multiple subscriptions in one connection.
    // For simplicity V1: Single subscription per connection? Or Multi?
    // "SUBSCRIBE ... Server replies OK, then starts pushing events".
    // If we want to support Unsubscribe/Subscribe more topics, we need select!.
    
    let mut buf = BytesMut::with_capacity(4096);

    loop {
        tokio::select! {
            // A. Incoming Event to Push
            res = rx.recv() => {
                match res {
                    Ok(ev) => {
                        let payload = encode_push_event_payload(&ev);
                        // Encode PUSH_EVENT frame manually as it has special Opcode and no Request context
                        // Response frame structure: [len][status][payload]
                        // But Push is server->client "Event". 
                        // Spec 4.3 PUSH_EVENT: [u8 status=0x80][...] (Wait, spec says PUSH_EVENT is 0x80 but in types we defined it as Opcode 0x80? Or Status?)
                        // Spec: "Response: [len][status][payload]".
                        // "PUSH_EVENT (server->client) ... [status=0x80]".
                        // So we reuse the Response structure but with Status=0x80 (PushEvent).
                        
                        let frame = encode_response(
                            unsafe { std::mem::transmute(0x80u8) }, // Hack if 0x80 is not in Status enum?
                            // Wait, Status enum has Ok=0, etc. 
                            // Let's check types.rs. Opcode::PushEvent = 0x80. Status doesn't have 0x80. 
                            // The spec says: Response [len][status][payload].
                            // "PUSH_EVENT ... [status=0x80]".
                            // Let's assume Status can hold 0x80 for Push. Or we treat it as a special frame.
                            // Ideally, we should add PushEvent to Status or handle it raw.
                            // In valid Rust enum, we cast if safe or use u8.
                            // Let's assume Status::Ok for "Push"? No, that confuses client.
                            // Let's use raw bytes construction to follow spec exactly.
                            
                            &payload
                        );
                        // Actually, encode_push_event_payload return the inner part [event_type][...]
                        // We need [len][status 0x80][payload].
                        
                        // Let's construct manually to avoid enum safety issues if Status doesn't have 0x80.
                        let len = 1 + payload.len(); // 1 byte for status 0x80
                        let mut out = BytesMut::with_capacity(4 + len);
                        use bytes::BufMut;
                        out.put_u32_le(len as u32);
                        out.put_u8(0x80);
                        out.extend_from_slice(&payload);
                        
                        if let Err(_) = s.write_all(&out).await {
                            break; // Client closed
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        ctx.metrics.inc_lagged();
                        let resp = encode_response(Status::ErrLagged, &[]);
                        if let Err(_) = s.write_all(&resp).await { break; }
                    }
                    Err(_) => break, // Channel closed
                }
            }
            
            // B. Incoming Command (Unsubscribe / Ping / Close)
            res = read_frame(s, ctx.cfg.max_frame_bytes, &mut buf) => {
                 match res {
                     Ok(Some(frame)) => {
                         let frame: BytesMut = frame;
                         if frame.len() < 5 { break; }
                         let opcode = frame[4];
                         match opcode {
                             0x21 => { // UNSUBSCRIBE - for V1 just close? Or handle gracefully?
                                 // Simple V1: Break loop (close)
                                 let resp = encode_response(Status::Ok, &[]);
                                 let _ = s.write_all(&resp).await;
                                 return Ok(());
                             }
                             0x04 => { // PING
                                 let resp = encode_response(Status::Ok, &[]);
                                 let _ = s.write_all(&resp).await;
                             }
                             _ => {
                                 // Ignore or Error? In sub mode, maybe ignore.
                             }
                         }
                     }
                     Ok(None) => return Ok(()), // EOF
                     Err(_) => return Ok(()), // Error
                 }
            }
        }
    }
    Ok(())
}
