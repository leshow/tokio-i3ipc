use bytes::{BufMut, ByteOrder, BytesMut, LittleEndian};
use futures::prelude::*;
use futures::sync::mpsc::Sender;
use futures::Stream;
use tokio::prelude::*;
use tokio_codec::{Decoder, FramedRead};
use tokio_uds::UnixStream;

use i3ipc_types::{
    decode_event,
    event::{self, Subscribe},
    msg::Msg,
    reply, socket_path, I3IPC, MAGIC,
};

use crate::{AsyncConnect, AsyncI3, I3Msg};

use std::{io, marker::PhantomData};

pub struct EvtCodec;

impl Decoder for EvtCodec {
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
            let payload_len = LittleEndian::read_u32(&src[6..10]) as usize;
            let evt_type = LittleEndian::read_u32(&src[10..14]);
            if src.len() < 14 + payload_len {
                Ok(None)
            } else {
                let evt = decode_event(evt_type, src[14..].as_mut().to_vec())?;
                src.clear();
                Ok(Some(evt))
            }
        } else {
            Ok(None)
        }
    }
}

pub fn get_workspaces(tx: Sender<reply::Workspaces>) -> io::Result<()> {
    let fut = AsyncI3::new()?;
    tokio::run(
        fut.and_then(|stream| {
            let buf = stream.encode_msg(Msg::Workspaces);
            dbg!(&buf[..]);
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| I3Msg::<reply::Workspaces> {
            stream,
            _marker: PhantomData,
        })
        .and_then(|resp| {
            dbg!(resp);
            Ok(())
        })
        // .and_then(|(stream, _buf)| tokio::io::read_exact(stream, [0_u8; 14]))
        // .and_then(|(stream, initial)| {
        //     if &initial[0..6] != MAGIC.as_bytes() {
        //         panic!("Magic str not received");
        //     }
        //     let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
        //     dbg!(payload_len);
        //     let msg_type: u32 = LittleEndian::read_u32(&initial[10..14]);
        //     dbg!(msg_type);
        //     tokio::io::read_exact(stream, vec![0_u8; payload_len])
        // })
        // .and_then(|(stream, buf)| {
        //     let reply = serde_json::from_slice::<reply::Workspaces>(&buf[..]).unwrap();
        //     dbg!(&reply);
        //     future::ok(stream)
        // })
        .map(|_| ())
        .map_err(|e| println!("{}", e)),
    );
    Ok(())
}

pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<Subscribe>,
) -> io::Result<()> {
    let fut = UnixStream::connect(socket_path()?)
        .and_then(move |stream| {
            let buf = subscribe_payload(events);
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            decode_response(stream, |msg_type: u32, buf: Vec<u8>| {
                let s = String::from_utf8(buf.to_vec()).unwrap();
                println!("{:?}", s);
                dbg!(msg_type);
            })
        })
        .and_then(move |stream| {
            let framed = FramedRead::new(stream, EvtCodec);
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

fn subscribe_payload(events: Vec<event::Subscribe>) -> BytesMut {
    let payload = serde_json::to_string(&events[..]).unwrap();
    let mut buf = BytesMut::with_capacity(14 + payload.len());
    buf.put_slice(MAGIC.as_bytes());
    buf.put_u32_le(payload.len() as u32);
    buf.put_u32_le(2);
    buf.put_slice(payload.as_bytes());
    println!("writing {:#?}", buf);
    buf
}

fn decode_response<F>(stream: UnixStream, f: F) -> impl Future<Item = UnixStream, Error = io::Error>
where
    F: Fn(u32, Vec<u8>),
{
    let buf = [0; 14];
    tokio::io::read_exact(stream, buf).and_then(|(stream, initial)| {
        if &initial[0..6] != MAGIC.as_bytes() {
            panic!("Magic str not received");
        }
        let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
        dbg!(payload_len);
        let evt_type = LittleEndian::read_u32(&initial[10..14]);

        let buf = vec![0; payload_len];
        tokio::io::read_exact(stream, buf).and_then(move |(stream, buf)| {
            f(evt_type, buf);
            future::ok(stream)
        })
    })
}

fn read_payload(evt_type: u32, buf: Vec<u8>) {
    let s = String::from_utf8(buf.to_vec()).unwrap();
    println!("{:?}", s);
    dbg!(evt_type);
    let out = decode_event(evt_type, buf).unwrap();
    dbg!(out);
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
        let (tx, rx) = mpsc::channel(5);
        get_workspaces(tx)?;
        Ok(())
    }

}
// let buf = [0; 14];
// tokio::io::read_exact(stream, buf).and_then(|(stream, initial)| {
//     if &initial[0..6] != MAGIC.as_bytes() {
//         panic!("Magic str not received");
//     }
//     let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
//     dbg!(payload_len);
//     let evt_type = LittleEndian::read_u32(&initial[10..14]);
//     let buf = vec![0; payload_len];
//     tokio::io::read_exact(stream, buf).and_then(move |(_stream, buf)| {
//         let s = String::from_utf8(buf.to_vec()).unwrap();
//         println!("{:?}", s);
//         dbg!(evt_type);
//         let out = decode_evt(evt_type, buf).unwrap();
//         dbg!(out);
//         future::ok(())
//     })
// })
