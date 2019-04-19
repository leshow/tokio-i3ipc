use i3ipc_types::*;

use futures::{try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use std::{io as stio, marker::PhantomData};
use tokio::io::{self as tio, AsyncRead};
use tokio_uds::UnixStream;

#[derive(Debug)]
pub struct I3Msg<D, S = UnixStream> {
    stream: S,
    _marker: PhantomData<D>,
}

impl<D, S> I3Msg<D, S>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            _marker: PhantomData,
        }
    }
}

impl<D, S> Future for I3Msg<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    type Item = MsgResponse<D>;
    type Error = stio::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut buf = [0_u8; 14];

        let (rd, init) = try_ready!(tio::read_exact(&mut self.stream, &mut buf).poll());

        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);
        let mut buf = vec![0_u8; payload_len];
        let (_rdr, payload) = try_ready!(tio::read_exact(rd, &mut buf).poll());

        Ok(Async::Ready(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload[..])?,
        }))
    }
}

pub fn read_msg<D, S>(stream: S) -> I3Msg<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    I3Msg {
        stream,
        _marker: PhantomData,
    }
}
