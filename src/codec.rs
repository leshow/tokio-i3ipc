use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use futures::prelude::*;
use tokio::prelude::*;
use tokio_uds::UnixStream;

use super::event::Event;
use super::msg::Msg;
use super::reply;
use super::{I3Connect, I3Stream};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    env,
    io::{self, Read, Write},
    os::unix::net,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_socket() -> io::Result<()> {
        let fut = UnixStream::connect(I3Connect::socket_path()?)
            .and_then(|stream| {
                // let (tx, rx) = s.split();

                let events = [Event::Window];
                let payload = serde_json::to_string(&events).unwrap();
                // let mut buf = BytesMut::with_capacity(14 + payload.len());
                // buf.put_slice(I3Stream::MAGIC.as_bytes());
                // buf.put_u32_le(payload.len() as u32);
                // buf.put_u32_le(2);
                // buf.put_slice(payload.as_bytes());
                // println!("{:#?}", buf);
                let mut buf = Vec::with_capacity(14 + payload.len());
                buf.extend("i3-ipc".bytes()); // 6 bytes
                buf.write_u32::<LittleEndian>(payload.len() as u32).unwrap(); // 4 bytes
                buf.write_u32::<LittleEndian>(2).unwrap(); // 4 bytes
                buf.extend(payload.bytes()); // payload.len() bytes

                println!("{:#?}", buf);
                tokio::io::write_all(stream, buf)
            })
            .and_then(|(stream, _buf)| {
                let mut buf: Vec<u8> = vec![0; 30];
                tokio::io::read_exact(stream, buf)
            })
            .inspect(|(_stream, buf)| {
                println!("buf: {:?}", buf);
            })
            .map(|(_stream, buf)| {
                let s = String::from_utf8(buf).unwrap();
                println!("{:?}", s);
                let out: reply::Node = serde_json::from_slice(s.as_bytes()).unwrap();
                out
            })
            .inspect(|node| {
                println!("node: {:?}", node);
            })
            .map(|_| ())
            .map_err(|e| eprintln!("{:?}", e));

        tokio::run(fut);
        Ok(())
    }

}
