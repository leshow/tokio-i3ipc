use byteorder::{ByteOrder, LittleEndian};
use bytes::{BufMut, BytesMut};
use futures::prelude::*;
use futures::sync::mpsc::Sender;
use futures::Stream;
use tokio::prelude::*;
use tokio_codec::{Decoder, FramedRead};
use tokio_uds::UnixStream;

use i3ipc_types::{
    event::{self, Event},
    msg::Msg,
    socket_path, I3IPC, MAGIC,
};

use std::io;

fn decode_evt(evt_type: u32, payload: Vec<u8>) -> io::Result<event::Evt> {
    use event::{Event, Evt};
    let evt_type = evt_type & !(1 << 31);
    dbg!(&evt_type);
    let body = match evt_type.into() {
        Event::Workspace => Evt::Workspace(Box::new(
            serde_json::from_slice::<event::WorkspaceData>(&payload[..])?,
        )),
        Event::Output => Evt::Output(serde_json::from_slice::<event::OutputData>(&payload[..])?),
        Event::Mode => Evt::Mode(serde_json::from_slice::<event::ModeData>(&payload[..])?),
        Event::Window => Evt::Window(Box::new(serde_json::from_slice::<event::WindowData>(
            &payload[..],
        )?)),
        Event::BarConfigUpdate => Evt::BarConfig(serde_json::from_slice::<event::BarConfigData>(
            &payload[..],
        )?),
        Event::Binding => Evt::Binding(serde_json::from_slice::<event::BindingData>(&payload[..])?),
        Event::Shutdown => {
            Evt::Shutdown(serde_json::from_slice::<event::ShutdownData>(&payload[..])?)
        }
        Event::Tick => Evt::Tick(serde_json::from_slice::<event::TickData>(&payload[..])?),
    };
    Ok(body)
}

pub struct EvtCodec;

impl Decoder for EvtCodec {
    type Item = event::Evt;
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
                let evt = decode_evt(evt_type, src[14..].as_mut().to_vec())?;
                src.clear();
                Ok(Some(evt))
            }
        } else {
            Ok(None)
        }
    }
}

pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Evt>,
    events: Vec<Event>,
) -> io::Result<()> {
    let fut = UnixStream::connect(socket_path()?)
        .and_then(move |stream| {
            let payload = serde_json::to_string(&events[..]).unwrap();
            let mut buf = BytesMut::with_capacity(14 + payload.len());
            buf.put_slice(MAGIC.as_bytes());
            buf.put_u32_le(payload.len() as u32);
            buf.put_u32_le(Msg::Subscribe.into());
            buf.put_slice(payload.as_bytes());
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            let buf = [0_u8; 30]; // <i3-ipc (6 bytes)><len (4 bytes)><type (4 bytes)><{success:true} 16 bytes>
            tokio::io::read_exact(stream, buf)
        })
        .inspect(|(_stream, buf)| {
            println!("got: {:?}", buf);
        })
        .and_then(|(stream, initial)| {
            if &initial[0..6] != MAGIC.as_bytes() {
                panic!("Magic str not received");
            }
            let payload_len: u32 = LittleEndian::read_u32(&initial[6..10]);
            dbg!(payload_len); // 16
            let msg_type: u32 = LittleEndian::read_u32(&initial[10..14]);
            dbg!(msg_type); // 2
            dbg!(String::from_utf8(initial[14..].to_vec()).unwrap());
            future::ok(stream)
        })
        .and_then(move |stream| {
            let framed = FramedRead::new(stream, EvtCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
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

#[cfg(test)]
mod test {

    use futures::{future, stream::Stream, sync::mpsc};
    use i3ipc_types::event::{self, Event};
    use std::io;

    use super::subscribe;
    #[test]
    fn test_sub() -> io::Result<()> {
        let mut rt =
            tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
        let (tx, rx) = mpsc::channel(5);
        subscribe(rt.handle(), tx, vec![Event::Window])?;
        let fut = rx.for_each(|e: event::Evt| {
            println!("received");
            println!("{:#?}", e);
            future::ok(())
        });
        rt.spawn(fut);
        rt.run().expect("failed runtime");
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
