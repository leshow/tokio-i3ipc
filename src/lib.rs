#![doc(html_root_url = "https://docs.rs/tokio-i3ipc/0.6.0")]
//! # tokio-i3ipc  
//!
//! This crate provides types and functions for working with i3's IPC protocol
//! in an async context and with tokio. It re-exports the subcrate `i3ipc-types`
//! because it is also used for a synchronous implementation of the protocol.
//!
//! This library follows a similar API to the synchronous version. All important
//! functions live on the [I3](struct.I3.html) type. You must first `await` a
//! [connect](struct.I3.html#method.connect) call, then you can execute
//! commands, send/read messages from i3, or subscribe to listen to `Event`s.
//!
//! ## Subscribe & Listen
//!
//! ```should_panic
//! # use futures::stream::StreamExt;
//! # use std::io;
//! use tokio_i3ipc::{event::{Event,Subscribe}, I3};
//!
//! #[tokio::main]
//! async fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect().await?;
//!     i3.subscribe([Subscribe::Window]).await?;
//!
//!     let mut listener = i3.listen();
//!     while let Some(event) = listener.next().await {
//!         match event? {
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
//! ```should_panic
//! use std::io;
//!
//! use tokio_i3ipc::{reply, I3};
//!
//! #[tokio::main]
//! async fn main() -> io::Result<()> {
//!     let mut i3 = I3::connect().await?;
//!     // this type can be inferred, here is written explicitly:
//!     let worksp: reply::Workspaces = i3.get_workspaces().await?;
//!     println!("{:#?}", worksp);
//!
//!     Ok(())
//! }
//! ```
//!
//! All the `get_*` functions on [I3](struct.I3.html) are simple wrappers around
//! two main async functions. You could write any of them yourself, in fact:
//! ```should_panic
//! # use std::io;
//! use tokio_i3ipc::{msg, reply, MsgResponse, I3};
//!
//! #[tokio::main]
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

#[cfg(feature = "default")]
pub mod tokio;
#[cfg(feature = "default")]
pub use crate::tokio::codec::*;
#[cfg(feature = "default")]
pub use crate::tokio::util::*;
#[cfg(feature = "default")]
pub use crate::tokio::*;

#[cfg(feature = "asyncstd")]
pub mod astd;
#[cfg(feature = "asyncstd")]
pub use astd::*;
