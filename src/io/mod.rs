//! Collection of functions & types implementing `Future` for interacting with the raw i3 stream
//!
//! # Reading
//! Use [read_msg_and](fn.read_msg_and.html) to read from i3 and continue using the stream
//!
//! # Writing
//! Use [write_msg](fn.write_msg.html), [write_msg_json](fn.write_msg.html), or [send_msg](fn.send_msg.html)
//! to write a buffer, write json data, or simply send a msg with no content (respectively).
//!
mod read_and;
mod read_msg;
mod write_msg;

/// decode a response and return the `Stream` to be used again
pub use read_and::*;
/// decode a single response
pub use read_msg::*;
/// Write a message and return the stream
pub use write_msg::*;
