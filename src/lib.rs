// re-export i3ipc-types so users only have to import 1 thing
pub use i3ipc_types::*;

use bytes::{Buf, BufMut};
use futures::Poll;
use tokio_io::{AsyncRead, AsyncWrite};
use serde::de::DeserializeOwned;
use tokio_core::reactor::Handle;
use tokio_uds::UnixStream;

use std::{env, io::{self, BufReader, Read, Write}, process::Command};

#[derive(Debug)]
pub struct I3(UnixStream);

pub struct MsgResponse<D> {
    msg_type: msg::Msg,
    payload: D,
}

impl Read for I3 {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl AsyncRead for I3 {
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        self.0.prepare_uninitialized_buffer(buf)
    }

    fn read_buf<B>(&mut self, buf: &mut B) -> Poll<usize, io::Error>
    where
        B: BufMut,
    {
        self.0.read_buf(buf)
    }
}


impl AsyncWrite for I3 {
    fn shutdown(&mut self) ->Poll<(), io::Error> {        
        self.0.shutdown(how)
    }

    fn write_buf<B>(&mut self, buf: &mut B) -> Poll<usize, io::Error>
    where
        B: Buf,
    {

            self.0.write_buf(buf)
    }
}

impl Write for I3 {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl I3 {
    const MAGIC: &'static str = "i3-ipc";
    pub fn new(handle: &Handle) -> io::Result<Self> {
        let path = I3::socket_path()?;
        let stream = UnixStream::connect(path);
        Ok(I3(stream))
    }

    fn socket_path() -> io::Result<String> {
        if let Ok(p) = env::var("I3SOCK") {
            return Ok(p);
        }
        let out = Command::new("i3").arg("--get-socketpath").output()?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Unable to get i3 socket path",
            ))
        }
    }

    fn send_msg<P>(&mut self, msg: msg::Msg, payload: P, handle: &Handle) -> io::Result<()>
    where
        P: AsRef<str>,
    {
        unimplemented!()
    }

    fn receive_msg<D: DeserializeOwned>(&mut self, handle: &Handle) -> io::Result<MsgResponse<D>> {
        let magic_str = [0u8; 6];
        self.read_exact(&mut magic_str);
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
