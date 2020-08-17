//! # i3-ipc
//!
//! Subscribing to events is easy:
//!
//! ```no_run
//! use i3_ipc::{
//!     event::{Event, Subscribe},
//!     I3Stream,
//! };
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut i3 = I3Stream::conn_sub(&[Subscribe::Window, Subscribe::Workspace])?;
//!     for e in i3.listen() {
//!         match e? {
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
//! Getting information is equally easy, use any `get_*` method or `run_command`
//! to send a message to i3:
//!
//! ```no_run
//! use i3_ipc::{Connect, I3};
//! use std::io;
//!
//! fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect()?;
//!     let workspaces = i3.get_workspaces()?;
//!     println!("{:?}", workspaces);
//!     Ok(())
//! }
//! ```

pub use i3ipc_types::*;

use serde::de::DeserializeOwned;

use std::{
    io::{self, Read, Write},
    os::unix::net::UnixStream,
};

/// Our connection type, we implement `Connect` for this
pub struct I3;

/// `I3Stream` will hold the underlying UnixStream that communicates with i3
#[derive(Debug)]
pub struct I3Stream(UnixStream);

/// Provides the `connect` method for `I3`
impl Connect for I3 {
    type Stream = I3Stream;

    fn connect() -> io::Result<I3Stream> {
        Ok(I3Stream(UnixStream::connect(socket_path()?)?))
    }
}

impl I3Stream {
    /// Connect & subscribe in one method
    pub fn conn_sub<E>(events: E) -> io::Result<Self>
    where
        E: AsRef<[event::Subscribe]>,
    {
        let mut i3 = I3::connect()?;
        i3.subscribe(events)?;
        Ok(i3)
    }

    /// sends a subscribe message to i3 with a json encoded array of types of
    /// events to listen to
    pub fn subscribe<E>(&mut self, events: E) -> io::Result<reply::Success>
    where
        E: AsRef<[event::Subscribe]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg(msg::Msg::Subscribe, &sub_json)?;
        let resp: MsgResponse<reply::Success> = self.receive_msg()?;
        Ok(resp.body)
    }

    /// Returns a type that implements `Iterator`, allowing us to listen to
    /// events
    pub fn listen(&'_ mut self) -> I3Iter<'_> {
        I3Iter { stream: self }
    }

    /// same as `listen`
    pub fn iter(&'_ mut self) -> I3Iter<'_> {
        I3Iter { stream: self }
    }

    /// Send a message and payload, used for `get_*` commands and `run_command`
    pub fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<usize>
    where
        P: AsRef<str>,
    {
        let buf = self.encode_msg_body(msg, payload);
        self.write(&buf[..])
    }

    /// Receive some message from the socket. Holds a `Msg` type and payload
    pub fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>> {
        let (msg_type, payload_bytes) = self.decode_msg()?;
        MsgResponse::new(msg_type, payload_bytes)
    }

    /// Like `receive_msg` but for `event::Event`
    pub fn receive_event(&mut self) -> io::Result<event::Event> {
        let (evt_type, payload_bytes) = self.decode_msg()?;
        decode_event(evt_type, payload_bytes)
    }

    /// Send a `Msg` and payload and receive a response. Convenience function
    /// over `send_msg` and `receive_msg`
    pub fn send_receive<P, D>(&mut self, msg: msg::Msg, payload: P) -> io::Result<MsgResponse<D>>
    where
        P: AsRef<str>,
        D: DeserializeOwned,
    {
        self.send_msg(msg, payload)?;
        self.receive_msg()
    }

    /// Run an arbitrary command on i3.
    pub fn run_command<S: AsRef<str>>(&mut self, payload: S) -> io::Result<Vec<reply::Success>> {
        self.send_msg(msg::Msg::RunCommand, payload)?;
        Ok(self.receive_msg()?.body)
    }

    /// Get active workspaces
    pub fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        let buf = self.encode_msg(msg::Msg::Workspaces);
        self.write_all(&buf[..])?;
        let resp: MsgResponse<Vec<reply::Workspace>> = self.receive_msg()?;
        Ok(resp.body)
    }

    /// Get active workspaces
    pub fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
        // self.send_msg(msg::Msg::Outputs, "")?;
        let buf = self.encode_msg(msg::Msg::Outputs);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get tree of all `Node`s in i3
    pub fn get_tree(&mut self) -> io::Result<reply::Node> {
        // self.send_msg(msg::Msg::Tree, "")?;
        let buf = self.encode_msg(msg::Msg::Tree);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get marks
    pub fn get_marks(&mut self) -> io::Result<reply::Marks> {
        // self.send_msg(msg::Msg::Marks, "")?;
        let buf = self.encode_msg(msg::Msg::Marks);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get your active bar ids
    pub fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        // self.send_msg(msg::Msg::BarConfig, "")?;
        let buf = self.encode_msg(msg::Msg::BarConfig);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get bar config by id (`get_bar_ids`)
    pub fn get_bar_config<S: AsRef<str>>(&mut self, bar_id: S) -> io::Result<reply::BarConfig> {
        self.send_msg(msg::Msg::BarConfig, bar_id)?;
        Ok(self.receive_msg()?.body)
    }

    /// Get i3 version and config location
    pub fn get_version(&mut self) -> io::Result<reply::Version> {
        let buf = self.encode_msg(msg::Msg::Version);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get i3 binding modes
    pub fn get_binding_modes(&mut self) -> io::Result<reply::BindingModes> {
        let buf = self.encode_msg(msg::Msg::BindingModes);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Get i3 config
    pub fn get_config(&mut self) -> io::Result<reply::Config> {
        let buf = self.encode_msg(msg::Msg::Config);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Convenience over `msg::Msg::Tick` and response
    pub fn get_tick(&mut self) -> io::Result<reply::Success> {
        let buf = self.encode_msg(msg::Msg::Tick);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    /// Convenience over `msg::Msg::Sync` and response
    pub fn get_sync(&mut self) -> io::Result<reply::Success> {
        let buf = self.encode_msg(msg::Msg::Sync);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }
}

impl Read for I3Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for I3Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// I3 event iterator, after you're subscribed to events (with the `subscribe`
/// method). The iterator will advance each iteration on receiving an `Event`.
/// These are decoded using `serde_json` and returned
#[derive(Debug)]
pub struct I3Iter<'a> {
    stream: &'a mut I3Stream,
}

impl<'a> Iterator for I3Iter<'a> {
    type Item = io::Result<event::Event>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.stream.receive_event())
    }
}
