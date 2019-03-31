use std::{env, io, process::Command};

pub mod event;
pub mod msg;
pub mod reply;

pub trait Connect {
    type Stream: I3IPC;
    fn connect() -> io::Result<Self::Stream>;
}

pub trait I3IPC {
    const MAGIC: &'static str = "i3-ipc";
    fn encode_msg_body<P>(&self, msg: msg::Msg, payload: P) -> Vec<u8>
    where
        P: AsRef<str>;
    fn encode_msg(&self, msg: msg::Msg) -> Vec<u8>;
    fn decode_msg(&mut self) -> io::Result<(u32, Vec<u8>)>;
}

#[derive(Debug)]
pub struct MsgResponse<D> {
    pub msg_type: msg::Msg,
    pub body: D,
}

pub fn socket_path() -> io::Result<String> {
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
