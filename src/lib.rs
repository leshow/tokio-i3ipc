use serde::de::DeserializeOwned;
use tokio_uds::UnixStream;

use std::{env, io, process::Command};

pub mod event;
pub mod msg;
pub mod reply;

#[derive(Debug)]
pub struct I3 {
    stream: UnixStream,
}

pub struct MsgResponse<D> {
    msg_type: msg::Msg,
    payload: D,
}

trait IPC {
    const MAGIC: &'static str;
    fn conn(&self) -> io::Result<String>;
    fn socket_path() -> io::Result<String>;
    fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<()>
    where
        P: AsRef<str>;
    fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>>;
}

impl IPC for I3 {
    const MAGIC: &'static str = "i3-ipc";
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

    fn conn(&self) -> io::Result<String> {
        unimplemented!()
    }

    fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<()>
    where
        P: AsRef<str>,
    {
        unimplemented!()
    }

    fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>> {
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
