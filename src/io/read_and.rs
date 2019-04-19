use crate::io;
use i3ipc_types::*;

use futures::{try_ready, Async, Future, Poll};
use serde::de::DeserializeOwned;
use std::{io as stio, marker::PhantomData};
use tokio::io::AsyncRead;
use tokio_uds::UnixStream;

#[derive(Debug)]
pub struct I3MsgAnd<D, S = UnixStream> {
    state: State<S, Option<MsgResponse<D>>>,
    _marker: PhantomData<D>,
}

impl<D, S> I3MsgAnd<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    pub fn new(stream: S) -> Self {
        Self {
            state: State::Reading { stream, resp: None },
            _marker: PhantomData,
        }
    }
}

impl<D, S> Future for I3MsgAnd<D, S>
where
    D: DeserializeOwned,
    S: AsyncRead,
{
    type Item = (S, MsgResponse<D>);
    type Error = stio::Error;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.state {
            State::Reading {
                ref mut stream,
                ref mut resp,
            } => {
                let msg = try_ready!(io::read_msg::<D, _>(stream).poll());
                *resp = Some(msg);
            }
            State::Empty => panic!("poll a ReadExact after it's done"),
        }

        match std::mem::replace(&mut self.state, State::Empty) {
            State::Reading { stream, resp, .. } => Ok(Async::Ready((
                stream,
                resp.expect("Should always contains something after read"),
            ))),
            State::Empty => panic!(),
        }
    }
}

pub fn read_msg_and<D, S>(stream: S) -> I3MsgAnd<D, S>
where
    S: AsyncRead,
    D: DeserializeOwned,
{
    I3MsgAnd::new(stream)
}

#[derive(Debug)]
enum State<S, D> {
    Reading { stream: S, resp: D },
    Empty,
}
