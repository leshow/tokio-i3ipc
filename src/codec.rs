use byteorder::{ByteOrder, LittleEndian};
use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use futures::prelude::*;
use tokio::prelude::*;
use tokio_uds::UnixStream;

use i3ipc_types::{
    event::{self, Event},
    msg::Msg,
    reply, socket_path, I3IPC, MAGIC,
};

use std::{
    env,
    io::{self, Read, Write},
    os::unix::net,
    path::{Path, PathBuf},
    process::Command,
};

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

fn subscribe(events: Vec<Event>) -> io::Result<()> {
    let fut = UnixStream::connect(socket_path()?)
        .and_then(move |stream| {
            let payload = serde_json::to_string(&events[..]).unwrap();
            let mut buf = BytesMut::with_capacity(14 + payload.len());
            buf.put_slice(MAGIC.as_bytes());
            buf.put_u32_le(payload.len() as u32);
            buf.put_u32_le(2);
            buf.put_slice(payload.as_bytes());
            println!("writing {:#?}", buf);

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
            dbg!(payload_len);
            let msg_type: u32 = LittleEndian::read_u32(&initial[10..14]);
            dbg!(msg_type);
            dbg!(String::from_utf8(initial[14..].to_vec()).unwrap());
            future::ok(stream)
        })
        .and_then(|stream| {
            let buf = [0; 14];
            tokio::io::read_exact(stream, buf)
        })
        .and_then(|(stream, initial)| {
            if &initial[0..6] != MAGIC.as_bytes() {
                panic!("Magic str not received");
            }
            let payload_len = LittleEndian::read_u32(&initial[6..10]) as usize;
            dbg!(payload_len);
            let evt_type = LittleEndian::read_u32(&initial[10..14]);

            let buf = vec![0; payload_len];
            tokio::io::read_exact(stream, buf).and_then(move |(_stream, buf)| {
                let s = String::from_utf8(buf.to_vec()).unwrap();
                println!("{:?}", s);
                dbg!(evt_type);
                let out = decode_evt(evt_type, buf).unwrap();
                // let out: event::WindowData = serde_json::from_slice(&buf[..]).unwrap();
                dbg!(out);
                future::ok(())
            })
        })
        // .inspect(|node| {
        //     println!("node: {:?}", node);
        // })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    tokio::run(fut);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sub() -> io::Result<()> {
        subscribe(vec![Event::Window])?;
        Ok(())
    }

}
