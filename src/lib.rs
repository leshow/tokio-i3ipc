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
mod util;

pub use util::*;
use codec::EventCodec;

use serde::de::DeserializeOwned;
use std::io;
use tokio::{codec::FramedRead, io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt}};
use tokio_uds::UnixStream;

/// [I3IPC](trait.I3IPC.html) provides default implementations for reading/writing buffers into a format i3 understands. This
/// trait expresses that + asynchronousity
pub trait AsyncI3IPC: AsyncRead + AsyncWrite + AsyncReadExt + AsyncWriteExt + I3Protocol {}

/// Add the default trait to `UnixStream`
impl AsyncI3IPC for UnixStream {}
impl<'a, T: ?Sized + AsyncI3IPC + Unpin> AsyncI3IPC for &'a mut T {}
impl<T: ?Sized + AsyncI3IPC + Unpin> AsyncI3IPC for Box<T> {}

/// Newtype wrapper for `ConnectFuture` meant to resolve some Stream, mostly likely `UnixStream`
#[derive(Debug)]
pub struct I3 {
    stream: UnixStream,
}

// Implement `Future` for [I3](struct.I3.html) so it can be polled into a ready `UnixStream`
impl I3 {
    /// Sends a message and payload, used for `get_*` commands and `run_command`
    async fn _send_msg<P>(&mut self, msg: msg::Msg, payload: Option<P>) -> io::Result<()>
    where
        P: AsRef<str>,
    {
        let buf = self.stream._encode_msg(msg, payload);
        self.stream.write_all(&buf).await
    }
    async fn _decode_msg(&mut self) -> io::Result<(u32, Vec<u8>)> {
        let mut init = [0_u8; 14];
        let _len = self.stream.read_exact(&mut init).await?;

        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);

        let mut payload = vec![0_u8; payload_len];
        let _len_read = self.stream.read_exact(&mut payload).await?;

        Ok((msg_type, payload))
    }

    /// Connects to I3 over `UnixStream`
    pub async fn connect() -> io::Result<Self> {
        Ok(I3 {
            stream: UnixStream::connect(socket_path()?).await?,
        })
    }

    pub async fn send_msg_body<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<()>
    where
        P: AsRef<str>,
    {
        self._send_msg(msg, Some(payload)).await
    }

    pub async fn send_msg(&mut self, msg: msg::Msg) -> io::Result<()> {
        self._send_msg::<&str>(msg, None).await
    }

    /// Receive some message from the socket. Holds a `Msg` type and payload
    pub async fn read_msg<D>(&mut self) -> io::Result<MsgResponse<D>>
    where
        D: DeserializeOwned,
    {
        let (msg_type, payload) = self._decode_msg().await?;
        Ok(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload[..])?,
        })
    }

    /// Like `read_msg` but for `event::Event`
    pub async fn read_event(&mut self) -> io::Result<event::Event> {
        let (evt_type, payload_bytes) = self._decode_msg().await?;
        decode_event(evt_type, payload_bytes)
    }

    /// Send a `Msg` and payload and receive a response. Convenience function over `send_msg` and `read_msg`
    pub async fn send_read<P, D>(&mut self, msg: msg::Msg, payload: P) -> io::Result<MsgResponse<D>>
    where
        P: AsRef<str>,
        D: DeserializeOwned,
    {
        self.send_msg_body(msg, payload).await?;
        self.read_msg().await
    }

    /// Returns a Future that will send a [Subscribe](event/enum.Subscribe.html) message to i3 along with a list of events to listen to.
    pub async fn subscribe<E>(&mut self, events: E) -> io::Result<MsgResponse<reply::Success>>
    where
        E: AsRef<[event::Subscribe]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg_body(msg::Msg::Subscribe, sub_json).await?;
        Ok(self.read_msg::<reply::Success>().await?)
    }

    /// Provides a type that implements `Stream` so you can `await` events in a loop
    pub fn listen(self) -> FramedRead<UnixStream, D> {
         FramedRead::new(self.stream, EventCodec)
    }

    /// Run an arbitrary command on i3. Response is a `Vec` of success true/false.
    pub async fn run_command<S: AsRef<str>>(
        &mut self,
        payload: S,
    ) -> io::Result<Vec<reply::Success>> {
        self.send_msg_body(msg::Msg::RunCommand, payload).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future for getting the current [Workspaces](../reply/struct.Workspace.html), sends [Workspaces](../msg/enum.Msg.html#variant.Workspaces)
    pub async fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        self.send_msg(msg::Msg::Workspaces).await?;
        let resp: MsgResponse<Vec<reply::Workspace>> = self.read_msg().await?;
        Ok(resp.body)
    }

    /// Future that gets all [Outputs](../reply/struct.Outputs.html), sends [Outputs](../msg/enum.Msg.html#variant.Outputs)
    pub async fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
        self.send_msg(msg::Msg::Outputs).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get complete [Node](../reply/struct.Node.html), sends [Tree](../msg/enum.Msg.html#variant.Tree)
    pub async fn get_tree(&mut self) -> io::Result<reply::Node> {
        self.send_msg(msg::Msg::Tree).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Get all [Marks](../reply/struct.Marks.html), sends [Marks](../msg/enum.Msg.html#variant.Marks)
    pub async fn get_marks(&mut self) -> io::Result<reply::Marks> {
        self.send_msg(msg::Msg::Marks).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get all [BarIds](../reply/struct.BarIds.html), sends [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
    pub async fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        self.send_msg(msg::Msg::BarConfig).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get configs associated with a bar id responds with [BarConfig](../reply/struct.BarConfig.html), sends [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
    pub async fn get_bar_config<S: AsRef<str>>(
        &mut self,
        bar_id: S,
    ) -> io::Result<reply::BarConfig> {
        self.send_msg_body(msg::Msg::BarConfig, bar_id).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Get i3 version
    pub async fn get_version(&mut self) -> io::Result<reply::Version> {
        self.send_msg(msg::Msg::Version).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get [BindingModes](../reply/struct.BindingModes.html), sends [BindingModes](../msg/enum.Msg.html#variant.BindingModes)
    pub async fn get_binding_modes(&mut self) -> io::Result<reply::BindingModes> {
        self.send_msg(msg::Msg::BindingModes).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future for [Config](../reply/struct.Config.html), sends [Config](../msg/enum.Msg.html#variant.Config)
    pub async fn get_config(&mut self) -> io::Result<reply::Config> {
        self.send_msg(msg::Msg::Config).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future sends [Tick](../msg/enum.Msg.html#variant.Tick)
    pub async fn get_tick(&mut self) -> io::Result<reply::Success> {
        self.send_msg(msg::Msg::Tick).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future [Sync](../msg/enum.Msg.html#variant.Sync)
    pub async fn get_sync(&mut self) -> io::Result<reply::Success> {
        self.send_msg(msg::Msg::Sync).await?;
        Ok(self.read_msg().await?.body)
    }
}
