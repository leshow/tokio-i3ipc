use crate::*;

use async_std::io::Read;
use serde::de::DeserializeOwned;
use std::io as stio;

/// Convenience function that decodes a single response and passes the type and
/// undecoded buffer to a closure
pub async fn decode_response<F, T, S>(stream: &mut S, f: F) -> stio::Result<T>
where
    F: Fn(u32, Vec<u8>) -> T,
    S: Read + Unpin,
{
    let mut buf = [0; 14];
    stream.read_exact(&mut buf).await?;
    if &buf[0..6] != MAGIC.as_bytes() {
        panic!("Magic str not received");
    }
    let payload_len = u32::from_ne_bytes([buf[6], buf[7], buf[8], buf[9]]) as usize;
    let msg_type = u32::from_ne_bytes([buf[10], buf[11], buf[12], buf[13]]);

    let mut payload = vec![0; payload_len];
    stream.read_exact(&mut payload).await?;
    Ok(f(msg_type, payload))
}

/// Decode a response into a [MsgResponse](struct.MsgResponse.html)
pub async fn decode_msg<D, S>(stream: &mut S) -> stio::Result<stio::Result<MsgResponse<D>>>
where
    D: DeserializeOwned,
    S: Read + Unpin,
{
    decode_response(stream, MsgResponse::new).await
}

/// Decode a response into an [Event](event/enum.Event.html)
pub async fn decode_event_future<D, S>(stream: &mut S) -> stio::Result<stio::Result<event::Event>>
where
    D: DeserializeOwned,
    S: Read + Unpin,
{
    decode_response(stream, decode_event).await
}
