// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;
mod codec;
pub use codec::*;

use futures::{try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use tokio::io::{self as tio, AsyncRead, AsyncWrite};
use tokio_uds::{ConnectFuture, UnixStream};

use std::{io, marker::PhantomData};

/// Newtype wrapper for `ConnectFuture` meant to resolve some Stream, mostly likely `UnixStream`
#[derive(Debug)]
pub struct I3 {
    conn: ConnectFuture,
    // _marker: PhantomData<R>
}

trait AsyncConnect {
    type Stream: AsyncI3IPC;
    fn conn() -> io::Result<Self>
    where
        Self: Sized;
}

/// `I3IPC` provides default implementations for reading/writing buffers into a format i3 understands. This
/// trait expresses that being asyncronous
pub trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC {}

/// Add the default trait to `UnixStream`
impl AsyncI3IPC for UnixStream {}

/// Here we implement `AsyncConnect` for I3, it's implementation provides a way to get a socket path and returns `ConnectFuture`
// impl<R> AsyncConnect for I3<R> where R: AsyncI3IPC {
impl AsyncConnect for I3 {
    type Stream = UnixStream; // R;
    fn conn() -> io::Result<Self> {
        Ok(I3 {
            conn: UnixStream::connect(socket_path()?),
        }) //, _marker: PhantomData })
    }
}

// Implement `Future` for `I3` so it can be polled into a ready `UnixStream`
// impl<R> Future for I3<R> where R: AsyncI3IPC {
impl Future for I3 where {
    type Item = UnixStream;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let stream = try_ready!(self.conn.poll());
        Ok(Async::Ready(stream))
    }
}

#[derive(Debug)]
pub struct I3Msg<D, R = UnixStream> {
    stream: R,
    _marker: PhantomData<D>,
}

impl<D, R> I3Msg<D, R>
where
    D: DeserializeOwned,
{
    pub fn new(stream: R) -> Self {
        Self {
            stream,
            _marker: PhantomData,
        }
    }
}

impl<D, R> Future for I3Msg<D, R>
where
    R: AsyncRead,
    D: DeserializeOwned,
{
    type Item = MsgResponse<D>;
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        let mut buf = [0_u8; 14];

        let (rd, init) = try_ready!(tio::read_exact(&mut self.stream, &mut buf).poll());

        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        dbg!(payload_len);
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);
        dbg!(msg_type);
        let mut buf = vec![0_u8; payload_len];
        let (_rdr, payload) = try_ready!(tio::read_exact(rd, &mut buf).poll());

        Ok(Async::Ready(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload[..])?,
        }))
    }
}

pub fn read_msg<D, R>(stream: R) -> I3Msg<D, R>
where
    R: AsyncRead,
    D: DeserializeOwned,
{
    I3Msg {
        stream,
        _marker: PhantomData,
    }
}

#[derive(Debug)]
pub struct I3MsgAnd<D, R = UnixStream> {
    state: State<R, Option<MsgResponse<D>>>,
    _marker: PhantomData<D>,
}

#[derive(Debug)]
enum State<R, D> {
    Reading { stream: R, resp: D },
    Empty,
}

impl<D, R> I3MsgAnd<D, R>
where
    R: AsyncRead,
    D: DeserializeOwned,
{
    pub fn new(stream: R) -> Self {
        Self {
            state: State::Reading { stream, resp: None },
            _marker: PhantomData,
        }
    }
}

impl<D, R> Future for I3MsgAnd<D, R>
where
    D: DeserializeOwned,
    R: AsyncRead,
{
    type Item = (R, MsgResponse<D>);
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        match self.state {
            State::Reading {
                ref mut stream,
                ref mut resp,
            } => {
                let msg = try_ready!(read_msg::<D, _>(stream).poll());
                *resp = Some(msg);
            }
            State::Empty => panic!("poll a ReadExact after it's done"),
        }

        match std::mem::replace(&mut self.state, State::Empty) {
            State::Reading { stream, resp, .. } => Ok(Async::Ready((
                stream,
                resp.expect("Should always contains something after read"),
            ))),
            State::Empty => panic!(),
        }
    }
}

pub fn read_msg_and<D, R>(stream: R) -> I3MsgAnd<D, R>
where
    R: AsyncRead,
    D: DeserializeOwned,
{
    I3MsgAnd::new(stream)
}

#[derive(Debug)]
pub struct I3Command<D, I3 = UnixStream> {
    msg: msg::Msg,
    state: State<I3, Option<MsgResponse<D>>>,
    _marker: PhantomData<D>,
}

impl<D, I3> Future for I3Command<D, I3>
where
    D: DeserializeOwned,
    I3: AsyncI3IPC,
{
    type Item = (I3, MsgResponse<D>);
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        match self.state {
            State::Reading {
                ref mut stream,
                ref mut resp,
            } => {
                let send = stream.encode_msg(self.msg);
                let (stream, _size) = try_ready!(tio::write_all(stream, send).poll());

                let msg = try_ready!(read_msg::<D, _>(stream).poll());
                *resp = Some(msg);
            }
            State::Empty => panic!("poll a ReadExact after it's done"),
        }

        match std::mem::replace(&mut self.state, State::Empty) {
            State::Reading { stream, resp, .. } => Ok(Async::Ready((
                stream,
                resp.expect("Should always contains something after read"),
            ))),
            State::Empty => panic!(),
        }
    }
}

pub fn run_msg<D, I3>(stream: I3, msg: msg::Msg) -> I3Command<D, I3>
where
    I3: AsyncI3IPC,
    D: DeserializeOwned,
{
    I3Command {
        msg,
        state: State::Reading { stream, resp: None },
        _marker: PhantomData,
    }
}
