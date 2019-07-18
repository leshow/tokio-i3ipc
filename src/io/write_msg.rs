use crate::AsyncI3IPC;
use i3ipc_types::*;

use futures::{ready, Future, Poll};
use std::{pin::Pin, task::Context};
use serde::Serialize;
use std::io as stio;
use tokio::io as tio;
use tokio_uds::UnixStream;

#[derive(Debug)]
pub struct I3WriteMsg<T, S = UnixStream> {
    msg: msg::Msg,
    state: StateW<Option<T>, S>,
}

#[derive(Debug)]
enum StateW<T, S> {
    Writing { stream: S, buf: T },
    Empty,
}

fn _write_msg<T, S>(stream: S, msg: msg::Msg, buf: Option<T>) -> I3WriteMsg<T, S>
where
    S: AsyncI3IPC,
    T: AsRef<str>,
{
    I3WriteMsg {
        msg,
        state: StateW::Writing { stream, buf },
    }
}

/// A future which can be used to write a [Msg](msg/enum.Msg.html) from i3 -- sending a buffer
pub fn write_msg<T, S>(stream: S, msg: msg::Msg, buf: T) -> I3WriteMsg<T, S>
where
    S: AsyncI3IPC,
    T: AsRef<str>,
{
    _write_msg(stream, msg, Some(buf))
}

/// A future which can be used to write a [Msg](msg/enum.Msg.html) from i3 (that sends no data)
pub fn send_msg<S>(stream: S, msg: msg::Msg) -> I3WriteMsg<String, S>
where
    S: AsyncI3IPC,
{
    _write_msg::<String, S>(stream, msg, None)
}

/// A future which can be used to write a [Msg](msg/enum.Msg.html) from i3 -- sending some json data
pub fn write_msg_json<T, S>(
    stream: S,
    msg: msg::Msg,
    payload: T,
) -> stio::Result<I3WriteMsg<String, S>>
where
    S: AsyncI3IPC,
    T: Serialize,
{
    Ok(write_msg(stream, msg, serde_json::to_string(&payload)?))
}

impl<T, S> Future for I3WriteMsg<T, S>
where
    S: AsyncI3IPC,
    T: AsRef<str>,
{
    type Output = stio::Result<S>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.state {
            StateW::Writing {
                ref mut stream,
                ref buf,
            } => {
                let buf = buf.as_ref();
                let send = stream._encode_msg(self.msg, buf);
                let (_strm, _size) = ready!(tio::write_all(stream, send).poll());
            }
            StateW::Empty => panic!("poll a WriteAll after it's done"),
        }

        match std::mem::replace(&mut self.state, StateW::Empty) {
            StateW::Writing { stream, .. } => Poll::Ready(Ok(stream).into()),
            StateW::Empty => panic!(),
        }
    }
}
