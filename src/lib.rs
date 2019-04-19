#![cfg_attr(feature = "nightly", feature(external_doc))]

pub use i3ipc_types::*;
pub mod codec;
pub mod get;
pub mod io;
mod util;

pub use util::*;

use futures::{try_ready, Async, Future, Poll};
use std::io as stio;
use tokio::io::{AsyncRead, AsyncWrite};
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

#[cfg(feature = "nightly")]
#[doc(include = "../README.md")]
type _READMETEST = ();
