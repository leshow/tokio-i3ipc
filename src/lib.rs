// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;
mod codec;
pub use codec::*;

use bytes::{ByteOrder, LittleEndian};
use futures::{try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use tokio_io::{io::read_exact, AsyncRead, AsyncWrite};
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

// Provide default impls from I3IPC plus require `AsyncRead` and `AsyncWrite`
trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC {}

// Add default impls to UnixStream
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
    stream: UnixStream, // R,
    _marker: PhantomData<D>,
}

// impl<R, D> I3Msg<R, D>
impl<D> I3Msg<D>
where
    // R: AsyncRead + AsyncWrite,
    D: DeserializeOwned,
{
    pub fn new(stream: UnixStream) -> Self {
        I3Msg {
            stream,
            _marker: PhantomData,
        }
    }
}

// impl<R, D> Future for I3Msg<R, D>
impl<D> Future for I3Msg<D>
where
    D: DeserializeOwned,
    // R: AsyncRead + AsyncWrite,
{
    type Item = MsgResponse<D>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        let mut buf = [0_u8; 14];
        let (rdr, initial) = try_ready!(read_exact(&self.stream, &mut buf).poll());

        if &initial[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
        dbg!(payload_len);
        let msg_type = LittleEndian::read_u32(&initial[10..14]);
        dbg!(msg_type);
        let mut buf = vec![0_u8; payload_len];
        let (_rdr, payload) = try_ready!(read_exact(rdr, &mut buf).poll());

        Ok(Async::Ready(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload[..])?,
        }))
    }
}
