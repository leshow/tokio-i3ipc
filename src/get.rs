
// use futures::prelude::*;
use tokio_uds::UnixStream;

use i3ipc_types::{
    msg::Msg,
    reply, MsgResponse, 
};

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

/// The following are all convenience functions over the `I3Command` future. They are all monomorphized to use
/// `UnixStream`, but the protocol can run on any `AsyncI3IPC` implementor (i3wm only uses unix stream though so,
/// I don't imagine it'll be terribly useful).
/// If you'd rather retain a generic interface and control flow yourself, use `run_msg` or `run_msg_payload`.
pub fn run_command<S: AsRef<str>>(
    stream: UnixStream,
    command: S,
) -> I3Command<Vec<reply::Success>, S, UnixStream> {
    run_msg_payload(stream, Msg::RunCommand, command)
}

pub fn get_workspaces(stream: UnixStream) -> I3Command<reply::Workspaces, String, UnixStream> {
    run_msg(stream, Msg::Workspaces)
}

pub fn get_outputs(stream: UnixStream) -> I3Command<reply::Outputs, String, UnixStream> {
    run_msg(stream, Msg::Outputs)
}

pub fn get_tree(stream: UnixStream) -> I3Command<reply::Node, String, UnixStream> {
    run_msg(stream, Msg::Tree)
}

pub fn get_marks(stream: UnixStream) -> I3Command<reply::Marks, String, UnixStream> {
    run_msg(stream, Msg::Marks)
}

pub fn get_bar_ids(stream: UnixStream) -> I3Command<reply::BarIds, String, UnixStream> {
    run_msg(stream, Msg::Marks)
}

pub fn get_bar_config(stream: UnixStream) -> I3Command<reply::BarConfig, String, UnixStream> {
    run_msg(stream, Msg::BarConfig)
}

pub fn get_binding_modes(stream: UnixStream) -> I3Command<reply::BindingModes, String, UnixStream> {
    run_msg(stream, Msg::BindingModes)
}

pub fn get_config(stream: UnixStream) -> I3Command<reply::Config, String, UnixStream> {
    run_msg(stream, Msg::Config)
}

pub fn get_tick(stream: UnixStream) -> I3Command<reply::Success, String, UnixStream> {
    run_msg(stream, Msg::Tick)
}

pub fn get_sync(stream: UnixStream) -> I3Command<reply::Success, String, UnixStream> {
    run_msg(stream, Msg::Sync)
}
