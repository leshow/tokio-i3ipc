// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;
pub mod codec;
pub mod get;

use futures::prelude::*;
use futures::{future, sync::mpsc::Sender, try_ready, Async, Future, Poll, };
use serde::{de::DeserializeOwned, Serialize};
use std::{io, marker::PhantomData};
use tokio::codec::FramedRead;
use tokio::io::{self as tio, AsyncRead, AsyncWrite};
use tokio_uds::{ConnectFuture, UnixStream};


/// Newtype wrapper for `ConnectFuture` meant to resolve some Stream, mostly likely `UnixStream`
#[derive(Debug)]
pub struct I3 {
    conn: ConnectFuture,
}

pub trait Connect {
    type Connected: AsyncI3IPC;
    type Error;
    type Future: Future<Item = Self::Connected, Error = Self::Error>;

    fn connect() -> io::Result<Self::Future>;
}

impl Connect for I3 {
    type Connected = UnixStream;
    type Future = ConnectFuture;
    type Error = io::Error;
    fn connect() -> io::Result<Self::Future> {
        Ok(UnixStream::connect(socket_path()?))
    }
}

/// `I3IPC` provides default implementations for reading/writing buffers into a format i3 understands. This
/// trait expresses that being asyncronous
pub trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC {}

/// Add the default trait to `UnixStream`
impl AsyncI3IPC for UnixStream {}

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

#[derive(Debug)]
pub struct I3MsgAnd<D, S = UnixStream> {
    state: State<S, Option<MsgResponse<D>>>,
    _marker: PhantomData<D>,
}

#[derive(Debug)]
enum State<S, D> {
    Reading { stream: S, resp: D },
    Empty,
}

impl<D, S> I3MsgAnd<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    pub fn new(stream: S) -> Self {
        Self {
            state: State::Reading { stream, resp: None },
            _marker: PhantomData,
        }
    }
}

impl<D, S> Future for I3MsgAnd<D, S>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    type Item = (S, MsgResponse<D>);
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

pub fn read_msg_and<D, S>(stream: S) -> I3MsgAnd<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    I3MsgAnd::new(stream)
}

/// Future for encoding a message, writing it to the stream, then reading the response
/// After which, it returns the stream and the message result in a tuple
#[derive(Debug)]
pub struct I3Command<D, P = String, S = UnixStream> {
    msg: msg::Msg,
    payload: Option<P>,
    state: State<S, Option<MsgResponse<D>>>,
    _marker: PhantomData<D>,
}

impl<D, P, S> Future for I3Command<D, P, S>
where
    D: DeserializeOwned,
    P: AsRef<str>,
    S: AsyncI3IPC,
{
    type Item = (S, MsgResponse<D>);
    type Error = io::Error;
    fn poll(&mut self) -> Poll<Self::Item, io::Error> {
        match self.state {
            State::Reading {
                ref mut stream,
                ref mut resp,
            } => {
                let msg = self.msg;
                let payload = self.payload.take();
                let send = stream._encode_msg(msg, payload);
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

fn _run_msg<D, P, S>(stream: S, msg: msg::Msg, payload: Option<P>) -> I3Command<D, P, S>
where
    S: AsyncI3IPC,
    D: DeserializeOwned,
    P: AsRef<str>,
{
    I3Command {
        msg,
        payload,
        state: State::Reading { stream, resp: None },
        _marker: PhantomData,
    }
}

pub fn run_msg_payload<D, P, S>(stream: S, msg: msg::Msg, payload: P) -> I3Command<D, P, S>
where
    S: AsyncI3IPC,
    D: DeserializeOwned,
    P: AsRef<str>,
{
    _run_msg(stream, msg, Some(payload))
}

pub fn run_msg_json<D, P, S>(
    stream: S,
    msg: msg::Msg,
    payload: P,
) -> io::Result<I3Command<D, String, S>>
where
    S: AsyncI3IPC,
    D: DeserializeOwned,
    P: Serialize,
{
    Ok(run_msg_payload(
        stream,
        msg,
        serde_json::to_string(&payload)?,
    ))
}

pub fn run_msg<D, S>(stream: S, msg: msg::Msg) -> I3Command<D, String, S>
where
    S: AsyncI3IPC,
    D: DeserializeOwned,
{
    _run_msg(stream, msg, None)
}

/// Convenience function that decodes a single response and passes the type and buffer to a closure
pub fn decode_response<F, T, S>(stream: S, f: F) -> impl Future<Item = (S, T), Error = io::Error>
where
    F: Fn(u32, Vec<u8>) -> T,
    S: AsyncRead,
{
    let buf = [0; 14];
    tokio::io::read_exact(stream, buf).and_then(|(stream, init)| {
        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        dbg!(payload_len);
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);

        let buf = vec![0; payload_len];
        tokio::io::read_exact(stream, buf)
            .and_then(move |(stream, buf)| future::ok((stream, f(msg_type, buf))))
    })
}

/// Convenience function that uses `decode_response`, formatting the reply in a `MsgResponse`
pub fn decode_msg<D, S>(
    stream: S,
) -> impl Future<Item = (S, io::Result<MsgResponse<D>>), Error = io::Error>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    decode_response(stream, MsgResponse::new)
}

/// Convenience function that returns the result in `Event` format
pub fn decode_event_fut<D, S>(
    stream: S,
) -> impl Future<Item = (S, io::Result<event::Event>), Error = io::Error>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    decode_response(stream, decode_event)
}

// subscribe functions
/// This does the initial sending of the subscribe command with a list of things to listen to
pub fn send_sub<E: AsRef<[event::Subscribe]>>(
    stream: UnixStream,
    events: E,
) -> io::Result<I3Command<reply::Success, String, UnixStream>> {
    run_msg_json(stream, msg::Msg::Subscribe, events.as_ref())
}

/// An easy-to-use subscribe, all you need to do is pass a runtime handle and a `Sender` half of a channel, then listen on
/// the `rx` side for events
pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<event::Subscribe>,
) -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| send_sub(stream, events).expect("failed to subscribe"))
        .and_then(|(stream, _)| {
            let framed = FramedRead::new(stream, codec::EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
                })
                .map_err(|err| println!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    rt.spawn(fut);
    Ok(())
}


#[cfg(test)]
mod test {

    use futures::{future, stream::Stream, sync::mpsc};
    use i3ipc_types::event::{self, Subscribe};
    use std::io;

    use super::subscribe;
    #[test]
    fn test_sub() -> io::Result<()> {
        let mut rt =
            tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
        let (tx, rx) = mpsc::channel(5);
        subscribe(rt.handle(), tx, vec![Subscribe::Window])?;
        let fut = rx.for_each(|e: event::Event| {
            println!("received");
            println!("{:#?}", e);
            future::ok(())
        });
        rt.spawn(fut);
        rt.run().expect("failed runtime");
        Ok(())
    }
}
