use byteorder::{ByteOrder, LittleEndian};
use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use futures::prelude::*;
use tokio::prelude::*;
use tokio_uds::UnixStream;

use super::event::Event;
use super::msg::Msg;
use super::reply;
use super::{I3Connect, I3Stream};

use std::{
    env,
    io::{self, Read, Write},
    os::unix::net,
    path::{Path, PathBuf},
    process::Command,
};

fn read_socket() -> io::Result<()> {
    let fut = UnixStream::connect(I3Connect::socket_path()?)
        .and_then(|stream| {
            let events = [Event::Window];
            let payload = serde_json::to_string(&events).unwrap();
            let mut buf = BytesMut::with_capacity(14 + payload.len());
            buf.put_slice(I3Stream::MAGIC.as_bytes());
            buf.put_u32_le(payload.len() as u32);
            buf.put_u32_le(2);
            buf.put_slice(payload.as_bytes());
            println!("writing {:#?}", buf);

            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            // let buf = BytesMut::with_capacity(14);
            let buf = [0_u8; 30]; // <i3-ipc (6 bytes)><len (4 bytes)><type (4 bytes)>
            tokio::io::read_exact(stream, buf)
        })
        .inspect(|(_stream, buf)| {
            println!("got: {:?}", buf);
        })
        .and_then(|(stream, initial)| {
            // if &initial[0..6] != I3Stream::MAGIC.as_bytes() {
            // return Err(io::Error::new(io::ErrorKind::Other, ""));
            // }
            let payload_len: u32 = LittleEndian::read_u32(&initial[6..10]);
            dbg!(payload_len);
            let msg_type: u32 = LittleEndian::read_u32(&initial[10..14]);
            dbg!(msg_type);
            dbg!(String::from_utf8(initial[14..].to_vec()).unwrap());
            let payload: Vec<u8> = Vec::with_capacity(payload_len as usize);
            tokio::io::read_exact(stream, payload)
        })
        .and_then(|(_stream, buf)| {
            dbg!(&buf);
            // let s = String::from_utf8(buf).unwrap();
            println!("{:?}", buf);
            let out: reply::Node = serde_json::from_slice(&buf[..]).unwrap();
            dbg!(out);
            future::ok(())
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
    fn test_read_socket() -> io::Result<()> {
        read_socket()?;
        Ok(())
    }

}
