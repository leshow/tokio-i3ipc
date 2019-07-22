use i3ipc_types::*;

use futures::{ready, Future, Poll};
use std::{pin::Pin, task::{Context}};
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
    type Output = stio::Result<MsgResponse<D>>;
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut buf = [0_u8; 14];

        let (rd, init) = ready!(tio::read_exact(&mut self.stream, &mut buf).poll());

        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);
        let mut buf = vec![0_u8; payload_len];
        let (_rdr, payload) = ready!(tio::read_exact(rd, &mut buf).poll());

        Poll::Ready(Ok(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload[..])?,
        }))
    }
}

/// A future which can be used to read a message from i3 (doesn't return a stream)
/// Created by the [read_msg](fn.read_msg.html) function.
/// For example:
/// ```rust
/// # use tokio_uds::UnixStream;
/// # use futures::future::Future;
/// # use std::io;
/// # use tokio_i3ipc::{reply, msg::Msg, MsgResponse, event, io as i3io};
///
/// pub fn get_outputs(
///     stream: UnixStream,
/// ) -> impl Future<Item = (UnixStream, MsgResponse<reply::Outputs>), Error = io::Error> {
///     i3io::send_msg(stream, Msg::Outputs).and_then(i3io::read_msg_and)
/// }
/// ```
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
