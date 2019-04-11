use bytes::BytesMut;
use futures::prelude::*;
use futures::sync::mpsc::Sender;
use futures::{future, Stream};
use serde::de::DeserializeOwned;
use tokio::codec::{Decoder, FramedRead};
use tokio_uds::UnixStream;

use i3ipc_types::{
    decode_event,
    event::{self, Subscribe},
    msg::Msg,
    reply, MsgResponse, I3IPC, MAGIC,
};

use crate::{read_msg, read_msg_and, run_msg, Connect, I3};

use std::io;

/// This codec only impls `Decoder` because it's only job is to read messages from i3 and turn
/// them into frames of Events. All other interactions with i3 over the IPC are simple send/receive
/// operations. Events received will be relative to what was subscribed.
pub struct EventCodec;

impl Decoder for EventCodec {
    type Item = event::Event;
    type Error = io::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        if src.len() > 14 {
            if &src[0..6] != MAGIC.as_bytes() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Expected 'i3-ipc' but received: {:?}", &src[0..6]),
                ));
            }
            let payload_len = u32::from_ne_bytes([src[6], src[7], src[8], src[9]]) as usize;
            let evt_type = u32::from_ne_bytes([src[10], src[11], src[12], src[13]]);
            // ends at payload + original 14 bytes
            let end_len = 14 + payload_len;
            if src.len() < end_len {
                Ok(None)
            } else {
                let evt = decode_event(evt_type, &src[14..end_len])?;
                src.advance(end_len);
                Ok(Some(evt))
            }
        } else {
            Ok(None)
        }
    }
}

/// Run an arbitrary command for i3 and decode the responses, represented as vector of success true/false
pub fn run_command<S>(
    command: S,
) -> impl Future<Item = io::Result<Vec<reply::Success>>, Error = io::Error>
where
    S: AsRef<str>,
{
    I3::connect()
        .expect("unable to get socket")
        .and_then(|stream: UnixStream| {
            let buf = stream.encode_msg_body(Msg::RunCommand, command);
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            decode_msg::<Vec<reply::Success>>(stream).map(|(_stream, msg)| msg.map(|m| m.body))
        })
}

pub fn get_workspaces() -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| {
            let buf = stream.encode_msg(Msg::Workspaces);
            dbg!(&buf[..]);
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| read_msg::<reply::Workspaces, _>(stream))
        .and_then(|resp| {
            dbg!(resp);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| println!("{}", e));
    Ok(())
}

pub fn get_outputs() -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| run_msg::<reply::Outputs, _>(stream, Msg::Outputs))
        .and_then(|resp| {
            dbg!(resp);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| println!("{}", e));
    Ok(())
}

pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<Subscribe>,
) -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(move |stream: UnixStream| {
            let buf = stream.encode_msg_json(Msg::Subscribe, events).unwrap();
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| read_msg_and::<reply::Success, _>(stream))
        .and_then(move |(stream, _)| {
            let framed = FramedRead::new(stream, EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
                })
                .map_err(|err| println!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    rt.spawn(fut);
    Ok(())
}

pub fn decode_response<F, T>(
    stream: UnixStream,
    f: F,
) -> impl Future<Item = (UnixStream, T), Error = io::Error>
where
    F: Fn(u32, Vec<u8>) -> T,
{
    let buf = [0; 14];
    tokio::io::read_exact(stream, buf).and_then(|(stream, init)| {
        if &init[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = u32::from_ne_bytes([init[6], init[7], init[8], init[9]]) as usize;
        dbg!(payload_len);
        let msg_type = u32::from_ne_bytes([init[10], init[11], init[12], init[13]]);

        let buf = vec![0; payload_len];
        tokio::io::read_exact(stream, buf)
            .and_then(move |(stream, buf)| future::ok((stream, f(msg_type, buf))))
    })
}

pub fn decode_msg<D>(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, io::Result<MsgResponse<D>>), Error = io::Error>
where
    D: DeserializeOwned,
{
    decode_response(stream, MsgResponse::new)
}

pub fn decode_event_fut<D>(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, io::Result<event::Event>), Error = io::Error>
where
    D: DeserializeOwned,
{
    decode_response(stream, decode_event)
}

#[cfg(test)]
mod test {

    use futures::{future, stream::Stream, sync::mpsc};
    use i3ipc_types::event::{self, Subscribe};
    use std::io;

    use super::{get_workspaces, subscribe};
    #[test]
    fn test_sub() -> io::Result<()> {
        let mut rt =
            tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
        let (tx, rx) = mpsc::channel(5);
        subscribe(rt.handle(), tx, vec![Subscribe::Window])?;
        let fut = rx.for_each(|e: event::Event| {
            println!("received");
            println!("{:#?}", e);
            future::ok(())
        });
        rt.spawn(fut);
        rt.run().expect("failed runtime");
        Ok(())
    }

    #[test]
    fn test_workspaces() -> io::Result<()> {
        get_workspaces()?;
        Ok(())
    }

}
