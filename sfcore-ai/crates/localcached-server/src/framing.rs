use bytes::{BytesMut, Buf};
use tokio::io::AsyncReadExt;

pub async fn read_frame<R: AsyncReadExt + Unpin>(
    r: &mut R,
    max_frame: usize,
    buf: &mut BytesMut,
) -> std::io::Result<Option<BytesMut>> {
    while buf.len() < 4 {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    let mut len_bytes = &buf[..4];
    let len = len_bytes.get_u32_le() as usize;
    
    if len == 0 || len > max_frame {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame too large"));
    }
    let total = 4 + len;
    while buf.len() < total {
        let n = r.read_buf(buf).await?;
        if n == 0 { return Ok(None); }
    }
    Ok(Some(buf.split_to(total)))
}
