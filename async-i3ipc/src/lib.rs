#![doc(html_root_url = "https://docs.rs/async-i3ipc/0.3.0")]
//! # async-i3ipc
//!
//! This crate provides types and functions for working with i3's IPC protocol
//! in an async context and with async-std (tokio version [here](https://docs.rs/crate/tokio-i3ipc)). It re-exports the subcrate `i3ipc-types`
//! because it is also used for a synchronous implementation of the protocol.
//!
//! This library follows a similar API to the synchronous version. All important
//! functions live on the [I3](struct.I3.html) type. You must first `await` a
//! [connect](struct.I3.html#method.connect) call, then you can execute
//! commands, send/read messages from i3, or subscribe to listen to `Event`s.
//!
//! ## Subscribe & Listen
//!
//! ```rust,no_run
//! use std::io;
//! use async_i3ipc::{
//!     event::{Event, Subscribe},
//!     I3,
//! };
//!
//! #[async_std::main]
//! async fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect().await?;
//!     let resp = i3.subscribe([Subscribe::Window]).await?;
//!
//!     println!("{:#?}", resp);
//!     let mut listener = i3.listen();
//!     while let Ok(event) = listener.next().await {
//!         match event {
//!             Event::Workspace(ev) => println!("workspace change event {:?}", ev),
//!             Event::Window(ev) => println!("window event {:?}", ev),
//!             Event::Output(ev) => println!("output event {:?}", ev),
//!             Event::Mode(ev) => println!("mode event {:?}", ev),
//!             Event::BarConfig(ev) => println!("bar config update {:?}", ev),
//!             Event::Binding(ev) => println!("binding event {:?}", ev),
//!             Event::Shutdown(ev) => println!("shutdown event {:?}", ev),
//!             Event::Tick(ev) => println!("tick event {:?}", ev),
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Sending/Reading from I3
//!
//! To [send messages](https://i3wm.org/docs/ipc.html#_sending_messages_to_i3) to i3,
//! call any of the `get_*` functions on [I3](struct.I3.html).
//!
//! ```no_run
//! use std::io;
//! use async_i3ipc::{reply, I3};
//!
//! #[async_std::main]
//! async fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect().await?;
//!     // this type can be inferred, here is written explicitly:
//!     let tree: reply::Node = i3.get_tree().await?;
//!     println!("{:#?}", tree);
//!
//!     Ok(())
//! }
//! ```
//!
//! All the `get_*` functions on [I3](struct.I3.html) are simple wrappers around
//! two main async functions. You could write any of them yourself, in fact:
//! ```no_run
//! # use std::io;
//! use async_i3ipc::{msg, reply, MsgResponse, I3};
//!
//! #[async_std::main]
//! # async fn main() -> io::Result<()> {
//! let mut i3 = I3::connect().await?;
//! // send msg RunCommand with a payload
//! let payload = "some_command";
//! i3.send_msg_body(msg::Msg::RunCommand, payload).await?;
//! let resp: MsgResponse<Vec<reply::Success>> = i3.read_msg().await?;
//! # Ok(())
//! # }
//! ```
pub use i3ipc_types::*;
pub mod stream;
mod util;

pub use stream::EventStream;
pub use util::*;

use async_std::{os::unix::net::UnixStream, prelude::*};
use serde::de::DeserializeOwned;
use std::io;

/// Newtype wrapper for `UnixStream` that implements i3's IPC
#[derive(Debug)]
pub struct I3 {
    stream: UnixStream,
}

// Implement `Future` for [I3](struct.I3.html) so it can be polled into a ready
// `UnixStream`
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

    /// Send a `Msg` and payload and receive a response. Convenience function
    /// over `send_msg` and `read_msg`
    pub async fn send_read<P, D>(&mut self, msg: msg::Msg, payload: P) -> io::Result<MsgResponse<D>>
    where
        P: AsRef<str>,
        D: DeserializeOwned,
    {
        self.send_msg_body(msg, payload).await?;
        self.read_msg().await
    }

    /// Returns a Future that will send a [Subscribe](event/enum.Subscribe.html)
    /// message to i3 along with a list of events to listen to.
    pub async fn subscribe<E>(&mut self, events: E) -> io::Result<reply::Success>
    where
        E: AsRef<[event::Subscribe]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg_body(msg::Msg::Subscribe, sub_json).await?;
        Ok(self.read_msg::<reply::Success>().await?.body)
    }

    /// Provides a type that implements `Stream` so you can `await` events in a
    /// loop
    pub fn listen(self) -> EventStream {
        EventStream::new(self.stream)
    }

    /// Run an arbitrary command on i3. Response is a `Vec` of success
    /// true/false.
    pub async fn run_command<S: AsRef<str>>(
        &mut self,
        payload: S,
    ) -> io::Result<Vec<reply::Success>> {
        self.send_msg_body(msg::Msg::RunCommand, payload).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future for getting the current
    /// [Workspaces](../reply/struct.Workspace.html), sends
    /// [Workspaces](../msg/enum.Msg.html#variant.Workspaces)
    pub async fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        self.send_msg(msg::Msg::Workspaces).await?;
        let resp: MsgResponse<Vec<reply::Workspace>> = self.read_msg().await?;
        Ok(resp.body)
    }

    /// Future that gets all [Outputs](../reply/struct.Outputs.html), sends
    /// [Outputs](../msg/enum.Msg.html#variant.Outputs)
    pub async fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
        self.send_msg(msg::Msg::Outputs).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get complete [Node](../reply/struct.Node.html), sends
    /// [Tree](../msg/enum.Msg.html#variant.Tree)
    pub async fn get_tree(&mut self) -> io::Result<reply::Node> {
        self.send_msg(msg::Msg::Tree).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Get all [Marks](../reply/struct.Marks.html), sends
    /// [Marks](../msg/enum.Msg.html#variant.Marks)
    pub async fn get_marks(&mut self) -> io::Result<reply::Marks> {
        self.send_msg(msg::Msg::Marks).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get all [BarIds](../reply/struct.BarIds.html), sends
    /// [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
    pub async fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        self.send_msg(msg::Msg::BarConfig).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future to get configs associated with a bar id responds with
    /// [BarConfig](../reply/struct.BarConfig.html), sends
    /// [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
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

    /// Future to get [BindingModes](../reply/struct.BindingModes.html), sends
    /// [BindingModes](../msg/enum.Msg.html#variant.BindingModes)
    pub async fn get_binding_modes(&mut self) -> io::Result<reply::BindingModes> {
        self.send_msg(msg::Msg::BindingModes).await?;
        Ok(self.read_msg().await?.body)
    }

    /// Future for [Config](../reply/struct.Config.html), sends
    /// [Config](../msg/enum.Msg.html#variant.Config)
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
