#![feature(async_await)]
#![doc(html_root_url = "https://docs.rs/tokio-i3ipc/0.5.0")]
//! # tokio-i3ipc  
//!
//! This crate provides types and functions for working with i3's IPC protocol within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.
//!
//! There are many ways you can interact with this library. You can import an already written future and simply spawn/run it, or you can use the building blocks to construct your own futures.
//!
//! # Subscribing
//!
//! ```should_panic
//! # use futures::{
//! #     future,
//! #     sink::Sink,
//! #     stream::Stream,
//! #     sync::mpsc::{self, Sender},
//! #     Future,
//! # };
//! # use std::io;
//! use tokio_i3ipc::{subscribe, event::{self, Subscribe}};
//!
//! fn main() -> io::Result<()> {
//!     let mut rt =
//!         tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
//!     // create a channel to receive responses
//!     let (tx, rx) = mpsc::channel(5);
//!     // pass a handle and `Sender` to `subscribe`
//!     subscribe(rt.handle(), tx, vec![Subscribe::Window])?;
//!     // handle the events received on the channel
//!     let fut = rx.for_each(|e: event::Event| {
//!         println!("received");
//!         println!("{:#?}", e);
//!         future::ok(())
//!     });
//!     rt.spawn(fut);
//!     rt.run().expect("failed runtime");
//!     Ok(())
//! }
//! ```
//!
//! `send_msg`, `write_msg_json` and `write_msg` will handle writing to i3. `read_msg` and `read_msg_and` will handle reading. The latter returns the stream again to continue using it.
//!
//! ## Sending Messages to i3
//!
//! To [send messages](https://i3wm.org/docs/ipc.html#_sending_messages_to_i3) to i3, there are a number of convenience futures that need only be passed a `UnixStream` and then run in your event loop.
//!
//! ```should_panic
//! # use futures::future::Future;
//! # use tokio_uds::UnixStream;
//! # use tokio;
//! use tokio_i3ipc::{I3, Connect, MsgResponse, get, reply};
//!
//! fn main() {
//!     let fut = I3::connect()
//!         .expect("unable to get socket")
//!         .and_then(get::get_workspaces)
//!         .and_then(
//!             |(_stream, reply): (UnixStream, MsgResponse<reply::Workspaces>)| {
//!                 // do something w/ reply::Workspaces
//!                 futures::future::ok(())
//!             },
//!         )
//!         .map(|_| ())
//!         .map_err(|_| ());
//!     tokio::run(fut);
//! }
//! ```

pub use i3ipc_types::*;
pub mod codec;
pub mod get;
pub mod io;
mod util;

pub use util::*;

use futures::{ready, Future};
use std::{io as stio, pin::Pin, task::{Context, Poll}};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use tokio_uds::{ConnectFuture, UnixStream};

/// Newtype wrapper for `ConnectFuture` meant to resolve some Stream, mostly likely `UnixStream`
#[derive(Debug)]
pub struct I3 {
    conn: ConnectFuture,
}

/// Provides a `connect` function for types which returns a `Future`
pub trait Connect {
    type Connected: AsyncI3IPC;
    type Future: Future<Output=Self::Connected>;

    fn connect() -> stio::Result<Self::Future>;
}

impl Connect for I3 {
    type Connected = UnixStream;
    type Future = ConnectFuture;
    fn connect() -> stio::Result<Self::Future> {
        Ok(UnixStream::connect(socket_path()?))
    }
}

/// [I3IPC](trait.I3IPC.html) provides default implementations for reading/writing buffers into a format i3 understands. This
/// trait expresses that + asynchronousity
pub trait AsyncI3IPC: AsyncRead + AsyncWrite + I3IPC + AsyncReadExt + AsyncWriteExt {}

/// Add the default trait to `UnixStream`
impl AsyncI3IPC for UnixStream {}
impl<'a, T: ?Sized + AsyncI3IPC + Unpin> AsyncI3IPC for &'a mut T {}
impl<T: ?Sized + AsyncI3IPC + Unpin> AsyncI3IPC for Box<T> {}

// Implement `Future` for [I3](struct.I3.html) so it can be polled into a ready `UnixStream`
impl Future for I3 where {
    type Output = stio::Result<UnixStream>;
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let stream = ready!(self.conn.poll(ctx));
        Poll::Ready(Ok(stream))
    }
}

// impl I3 {
//     pub async fn connect(&mut self) -> stio::Result<UnixStream> {
//         Ok(self.conn.await?)
//     }
// }
