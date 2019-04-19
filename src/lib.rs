// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;
pub mod codec;
pub mod get;
pub mod io;

use io as i3io;

use futures::prelude::*;
use futures::{future, sync::mpsc::Sender, try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use std::io as stio;
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

    fn connect() -> stio::Result<Self::Future>;
}

impl Connect for I3 {
    type Connected = UnixStream;
    type Future = ConnectFuture;
    type Error = stio::Error;
    fn connect() -> stio::Result<Self::Future> {
        Ok(UnixStream::connect(socket_path()?))
    }
}

/// `I3IPC` provides default implementations for reading/writing buffers into a format i3 understands. This
/// trait expresses that being asyncronous
pub trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC {}

/// Add the default trait to `UnixStream`
impl AsyncI3IPC for UnixStream {}
impl<'a, T: ?Sized + AsyncI3IPC> AsyncI3IPC for &'a mut T {}
impl<T: ?Sized + AsyncI3IPC> AsyncI3IPC for Box<T> {}

// Implement `Future` for `I3` so it can be polled into a ready `UnixStream`
// impl<R> Future for I3<R> where R: AsyncI3IPC {
impl Future for I3 where {
    type Item = UnixStream;
    type Error = stio::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let stream = try_ready!(self.conn.poll());
        Ok(Async::Ready(stream))
    }
}

/// Convenience function that decodes a single response and passes the type and buffer to a closure
pub fn decode_response<F, T, S>(stream: S, f: F) -> impl Future<Item = (S, T), Error = stio::Error>
where
    F: Fn(u32, Vec<u8>) -> T,
    S: AsyncRead,
{
    let buf = [0; 14];
    tio::read_exact(stream, buf).and_then(|(stream, init)| {
        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);

        let buf = vec![0; payload_len];
        tio::read_exact(stream, buf)
            .and_then(move |(stream, buf)| future::ok((stream, f(msg_type, buf))))
    })
}

/// Convenience function that uses `decode_response`, formatting the reply in a `MsgResponse`
pub fn decode_msg<D, S>(
    stream: S,
) -> impl Future<Item = (S, stio::Result<MsgResponse<D>>), Error = stio::Error>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    decode_response(stream, MsgResponse::new)
}

/// Convenience function that returns the result in `Event` format
pub fn decode_event_fut<D, S>(
    stream: S,
) -> impl Future<Item = (S, stio::Result<event::Event>), Error = stio::Error>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    decode_response(stream, decode_event)
}

pub fn subscribe_future<E: AsRef<[event::Subscribe]>>(
    stream: UnixStream,
    events: E,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Success>), Error = stio::Error> {
    i3io::write_msg_json(stream, msg::Msg::Subscribe, events.as_ref())
        .expect("Encoding failed")
        .and_then(i3io::read_msg_and::<reply::Success, _>)
}
/// An easy-to-use subscribe, all you need to do is pass a runtime handle and a `Sender` half of a channel, then listen on
/// the `rx` side for events
pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<event::Subscribe>,
) -> stio::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream| {
            i3io::write_msg_json(stream, msg::Msg::Subscribe, events).expect("Encoding failed")
        })
        .and_then(i3io::read_msg_and::<reply::Success, _>)
        .and_then(|(stream, _r)| {
            let framed = FramedRead::new(stream, codec::EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| stio::Error::new(stio::ErrorKind::BrokenPipe, e))
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
        println!("starting");
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
