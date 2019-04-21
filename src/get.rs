//! Convenience functions for getting data from i3. All of the following functions take a `UnixStream`
//! and return a `Future` that will produce some data. They are mappings of `send_msg` and `read_msg_and`
//! their appropriate output.
//!
//! While the protocol technically can work over any `AsyncRead`+`AsyncWrite`, in reality it's only
//! implemented for `UnixStream`. So all the types are monomorphized here. However, if you need raw access
//! use [send_msg](../io/fn.send_msg.html), [write_msg](../io/fn.write_msg.html), or
//!  [write_msg_json](../io/fn.write_msg_json.html) (sends json payload along with message).
use futures::Future;
use tokio_uds::UnixStream;

use crate::{io as i3io, *};
use i3ipc_types::{msg::Msg, reply, MsgResponse};

use std::io;

/// Run an arbitrary command for i3 and decode the responses, represented as vector of success true/false
pub fn connect_and_run_command<S>(
    command: S,
) -> impl Future<Item = MsgResponse<Vec<reply::Success>>, Error = io::Error>
where
    S: AsRef<str>,
{
    I3::connect()
        .expect("unable to get socket")
        .and_then(|stream| i3io::write_msg(stream, msg::Msg::RunCommand, command))
        .and_then(i3io::read_msg_and)
        .map(|(_stream, resp)| resp)
}

/// Run an arbitrary command on i3. Response is a `Vec` of success true/false.
pub fn run_command<S: AsRef<str>>(
    stream: UnixStream,
    command: S,
) -> impl Future<Item = (UnixStream, MsgResponse<Vec<reply::Success>>), Error = io::Error> {
    i3io::write_msg(stream, Msg::RunCommand, command).and_then(i3io::read_msg_and)
}

/// Future for getting the current [Workspaces](../reply/struct.Workspace.html), sends [Workspaces](../msg/enum.Msg.html#variant.Workspaces)
pub fn get_workspaces(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Workspaces>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Workspaces).and_then(i3io::read_msg_and)
}

/// Future that gets all [Outputs](../reply/struct.Outputs.html), sends [Outputs](../msg/enum.Msg.html#variant.Outputs)
pub fn get_outputs(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Outputs>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Outputs).and_then(i3io::read_msg_and)
}

/// Future to get complete [Node](../reply/struct.Node.html), sends [Tree](../msg/enum.Msg.html#variant.Tree)
pub fn get_tree(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Node>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Tree).and_then(i3io::read_msg_and)
}

/// Get all [Marks](../reply/struct.Marks.html), sends [Marks](../msg/enum.Msg.html#variant.Marks)
pub fn get_marks(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Marks>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Marks).and_then(i3io::read_msg_and)
}

/// Future to get all [BarIds](../reply/struct.BarIds.html), sends [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
pub fn get_bar_ids(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::BarIds>), Error = io::Error> {
    i3io::send_msg(stream, Msg::BarConfig).and_then(i3io::read_msg_and)
}

/// Future to get configs associated with a bar id responds with [BarConfig](../reply/struct.BarConfig.html), sends [BarConfig](../msg/enum.Msg.html#variant.BarConfig)
pub fn get_bar_config(
    stream: UnixStream,
    ids: Vec<String>,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::BarConfig>), Error = io::Error> {
    i3io::write_msg_json(stream, Msg::BarConfig, ids)
        .expect("Serialization of BarIds failed")
        .and_then(i3io::read_msg_and)
}

/// Future to get [BindingModes](../reply/struct.BindingModes.html), sends [BindingModes](../msg/enum.Msg.html#variant.BindingModes)
pub fn get_binding_modes(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::BindingModes>), Error = io::Error> {
    i3io::send_msg(stream, Msg::BindingModes).and_then(i3io::read_msg_and)
}

/// Future for [Config](../reply/struct.Config.html), sends [Config](../msg/enum.Msg.html#variant.Config)
pub fn get_config(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Config>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Config).and_then(i3io::read_msg_and)
}

/// Future sends [Tick](../msg/enum.Msg.html#variant.Tick)
pub fn get_tick(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Success>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Tick).and_then(i3io::read_msg_and)
}

/// Future [Sync](../msg/enum.Msg.html#variant.Sync)
pub fn get_sync(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Success>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Sync).and_then(i3io::read_msg_and)
}
