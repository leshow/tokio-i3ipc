use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use futures::prelude::*;
use tokio::prelude::*;
use tokio_uds::UnixStream;

use std::{
    env,
    io::{self, Read, Write},
    os::unix::net,
    path::{Path, PathBuf},
    process::Command,
};

use super::event::Event;
use super::msg::Msg;
use super::{I3Connect, I3Stream};

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_socket() -> io::Result<()> {
        let fut = UnixStream::connect(I3Connect::socket_path()?)
            .and_then(|s| {
                // let (tx, rx) = s.split();

                let sub_json = "[ \"window\" ]";
                let mut buf = BytesMut::with_capacity(14 + sub_json.len());
                buf.put_slice(b"i3-ipc");
                buf.put_u32_le(sub_json.len() as u32);
                buf.put_u32_le(2);
                buf.put_slice(sub_json.as_bytes());
                println!("{:?}", buf);

                tokio::io::write_all(s, buf)
            })
            .and_then(|(stream, _buf)| {
                let mut buf: Vec<u8> = vec![0; 256];
                tokio::io::read_exact(stream, buf)
            })
            .inspect(|(_stream, buf)| {
                println!("buf: {:?}", buf);
            })
            .map(|_| ())
            .map_err(|e| eprintln!("{:?}", e));

        tokio::run(fut);
        Ok(())
    }

}
