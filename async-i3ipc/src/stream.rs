//! Implements `tokio_codec`'s `Decoder` trait
//!
//! Using `EventCodec` to subscribe to [event::Event](../event/enum.Event.html)
//! from i3:
//!
//! ```no_run
//! # use tokio::stream::StreamExt;
//! # use std::io;
//! use tokio_i3ipc::{event::Subscribe, I3};
//!
//! #[tokio::main(basic_scheduler)]
//! async fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect().await?;
//!     i3.subscribe([Subscribe::Window]).await?;
//!
//!     let mut listener = i3.listen();
//!     while let Some(event) = listener.next().await {
//!         println!("{:#?}", event);
//!     }
//!     Ok(())
//! }
//! ```
// use bytes::{buf::Buf, BytesMut};
// use tokio_util::codec::Decoder;

use async_std::{
    io::{Read, ReadExt},
    os::unix::net::UnixStream,
    stream::Stream,
    task,
};
use i3ipc_types::{decode_event, event, MAGIC};
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

/// This codec only impls `Decoder` because it's only job is to read messages
/// from i3 and turn them into frames of Events. All other interactions with i3
/// over the IPC are simple send/receive operations. Events received will be
/// relative to what was subscribed.
pub struct EventStream {
    inner: UnixStream,
}

impl EventStream {
    pub fn new(inner: UnixStream) -> Self {
        Self { inner }
    }

    pub async fn next_event(&mut self) -> io::Result<event::Event> {
        let mut buf = Vec::new();
        loop {
            let _len = self.inner.read(&mut buf).await?;
            if buf.len() > 14 {
                if &buf[0..6] != MAGIC.as_bytes() {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Expected 'i3-ipc' but received: {:?}", &buf[0..6]),
                    ));
                }
                let payload_len = u32::from_ne_bytes([buf[6], buf[7], buf[8], buf[9]]) as usize;
                let evt_type = u32::from_ne_bytes([buf[10], buf[11], buf[12], buf[13]]);
                // ends at payload + original 14 bytes
                let end_len = 14 + payload_len;
                if buf.len() < end_len {
                    continue;
                } else {
                    return Ok(decode_event(evt_type, &buf[14..end_len])?);
                }
            } else {
                continue;
            }
        }
    }
}

impl Stream for EventStream {
    type Item = io::Result<event::Event>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buf = Vec::new();
        loop {
            if buf.len() > 14 {
                if &buf[0..6] != MAGIC.as_bytes() {
                    return Poll::Ready(Some(Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Expected 'i3-ipc' but received: {:?}", &buf[0..6]),
                    ))));
                }
                let payload_len = u32::from_ne_bytes([buf[6], buf[7], buf[8], buf[9]]) as usize;
                let evt_type = u32::from_ne_bytes([buf[10], buf[11], buf[12], buf[13]]);
                // ends at payload + original 14 bytes
                let end_len = 14 + payload_len;
                if buf.len() < end_len {
                    return Poll::Pending;
                } else {
                    let evt = decode_event(evt_type, &buf[14..end_len])?;
                    buf.clear();
                    return Poll::Ready(Some(Ok(evt)));
                }
            } else {
                let _len = task::ready!(Pin::new(&mut self.inner).poll_read(cx, &mut buf))?;
            }
        }
    }
}
