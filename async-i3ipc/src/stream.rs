use async_std::{io::ReadExt, os::unix::net::UnixStream};
use i3ipc_types::{decode_event, event, MAGIC};
use std::io;

pub struct EventStream {
    inner: UnixStream,
}

impl EventStream {
    pub fn new(inner: UnixStream) -> Self {
        Self { inner }
    }

    // doesn't actually use the Stream trait because we need to use `read_exact`
    //  and I don't feel like doing all the logic for that
    pub async fn next(&mut self) -> io::Result<event::Event> {
        let mut init = [0_u8; 14];
        let _len = self.inner.read_exact(&mut init).await?;

        assert!(!(&init[0..6] != MAGIC.as_bytes()), "Magic str not received");
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);

        let mut payload = vec![0_u8; payload_len];
        let _len_read = self.inner.read_exact(&mut payload).await?;

        decode_event(msg_type, payload)
    }
}
