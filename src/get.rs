//! Convenience functions for getting data from i3. All of the following functions take a `UnixStream`
//! and return a `Future` that will produce some data. They are mappings of `lib::run_msg` and `Msg::*` to
//! their appropriate output.
//!
//! While the protocol technically can work over any `AsyncRead`+`AsyncWrite`, in reality it's only
//! implemented for `UnixStream`. So all the types are monomorphized here. However, if you need access
//! just use `lib::run_msg` and `lib::run_msg_payload` (sends payload along with message).
use tokio_uds::UnixStream;

use i3ipc_types::{msg::Msg, reply, MsgResponse};

use crate::*;

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
        .and_then(|stream: UnixStream| run_msg_payload(stream, Msg::RunCommand, command))
        .map(|(_stream, resp)| resp)
}

// alternate way of writing it
// pub fn run_command_fut<S>(
//     command: S,
// ) -> impl Future<Item = io::Result<MsgResponse<Vec<reply::Success>>>, Error = io::Error>
// where
//     S: AsRef<str>,
// {
//     I3::connect()
//         .expect("unable to get socket")
//         .and_then(|stream: UnixStream| {
//             let buf = stream.encode_msg_body(Msg::RunCommand, command);
//             tokio::io::write_all(stream, buf)
//         })
//         .and_then(|(stream, _buf)| {
//             decode_msg::<Vec<reply::Success>, _>(stream).map(|(_stream, msg)| msg)
//         })
// }

/// Run an arbitrary command on i3. Response is a `Vec` of success true/false.
pub fn run_command<S: AsRef<str>>(
    stream: UnixStream,
    command: S,
) -> I3Command<Vec<reply::Success>, S, UnixStream> {
    run_msg_payload(stream, Msg::RunCommand, command)
}

/// Future for getting the current workspaces
pub fn get_workspaces(stream: UnixStream) -> I3Command<reply::Workspaces, String, UnixStream> {
    run_msg(stream, Msg::Workspaces)
}

/// Future that gets all outputs (screens)
pub fn get_outputs(stream: UnixStream) -> I3Command<reply::Outputs, String, UnixStream> {
    run_msg(stream, Msg::Outputs)
}

/// Future to get complete node tree
pub fn get_tree(stream: UnixStream) -> I3Command<reply::Node, String, UnixStream> {
    run_msg(stream, Msg::Tree)
}

/// Get all Marks
pub fn get_marks(stream: UnixStream) -> I3Command<reply::Marks, String, UnixStream> {
    run_msg(stream, Msg::Marks)
}

/// Future to get BarIds
pub fn get_bar_ids(stream: UnixStream) -> I3Command<reply::BarIds, String, UnixStream> {
    run_msg(stream, Msg::Marks)
}

/// Future to get Bar Config
pub fn get_bar_config(stream: UnixStream) -> I3Command<reply::BarConfig, String, UnixStream> {
    run_msg(stream, Msg::BarConfig)
}

/// Future to get BindingModes
pub fn get_binding_modes(stream: UnixStream) -> I3Command<reply::BindingModes, String, UnixStream> {
    run_msg(stream, Msg::BindingModes)
}

/// Future for Get Config
pub fn get_config(stream: UnixStream) -> I3Command<reply::Config, String, UnixStream> {
    run_msg(stream, Msg::Config)
}

/// Future for Tick
pub fn get_tick(stream: UnixStream) -> I3Command<reply::Success, String, UnixStream> {
    run_msg(stream, Msg::Tick)
}

/// Future for Get Sync
pub fn get_sync(stream: UnixStream) -> I3Command<reply::Success, String, UnixStream> {
    run_msg(stream, Msg::Sync)
}
