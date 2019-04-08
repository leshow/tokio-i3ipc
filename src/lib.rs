// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;
mod codec;
pub use codec::*;

use bytes::{Buf, BufMut, ByteOrder, Bytes, BytesMut, LittleEndian};
use futures::{try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use tokio::prelude::*;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_uds::{ConnectFuture, UnixStream};

use std::{io, marker::PhantomData};

#[derive(Debug)]
pub struct I3(ConnectFuture);

trait AsyncConnect {
    type Stream: AsyncI3IPC;
    fn new() -> io::Result<Self>
    where
        Self: Sized;
}

// unused so far
trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC {}
// add default impls to UnixStream
impl AsyncI3IPC for UnixStream {}

impl AsyncConnect for I3 {
    type Stream = UnixStream;
    fn new() -> io::Result<Self> {
        Ok(I3(UnixStream::connect(socket_path()?)))
    }
}

impl Future for I3 {
    type Item = UnixStream;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let stream = try_ready!(self.0.poll());
        Ok(Async::Ready(stream))
    }
}

#[derive(Debug)]
pub struct I3Msg<D> {
    stream: UnixStream,
    _marker: PhantomData<D>,
}

impl<D: DeserializeOwned> Future for I3Msg<D> {
    type Item = MsgResponse<D>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        let mut initial = BytesMut::with_capacity(1024);
        try_ready!(self.stream.read_buf(&mut initial));
        dbg!(&initial);
        if &initial[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
        dbg!(payload_len);
        let msg_type = LittleEndian::read_u32(&initial[10..14]);
        dbg!(msg_type);
        // try_ready!(self.stream.read_buf(&mut initial));

        Ok(Async::Ready(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&initial[14..])?,
        }))
    }
}
