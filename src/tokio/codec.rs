//! Implements `tokio_codec`'s `Decoder` trait
//!
//! Using `EventCodec` to subscribe to [event::Event](../event/enum.Event.html)
//! from i3:
//!
//! ```should_panic
//! # use futures::stream::StreamExt;
//! # use std::io;
//! use tokio_i3ipc::{event::Subscribe, I3};
//!
//! #[tokio::main]
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
use bytes::BytesMut;
use tokio::codec::Decoder;

use i3ipc_types::{decode_event, event, MAGIC};

use std::io;

/// This codec only impls `Decoder` because it's only job is to read messages
/// from i3 and turn them into frames of Events. All other interactions with i3
/// over the IPC are simple send/receive operations. Events received will be
/// relative to what was subscribed.
pub struct EventCodec;

impl Decoder for EventCodec {
    type Error = io::Error;
    type Item = event::Event;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        if src.len() > 14 {
            if &src[0..6] != MAGIC.as_bytes() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Expected 'i3-ipc' but received: {:?}", &src[0..6]),
                ));
            }
            let payload_len = u32::from_ne_bytes([src[6], src[7], src[8], src[9]]) as usize;
            let evt_type = u32::from_ne_bytes([src[10], src[11], src[12], src[13]]);
            // ends at payload + original 14 bytes
            let end_len = 14 + payload_len;
            if src.len() < end_len {
                Ok(None)
            } else {
                let evt = decode_event(evt_type, &src[14..end_len])?;
                src.advance(end_len);
                Ok(Some(evt))
            }
        } else {
            Ok(None)
        }
    }
}
