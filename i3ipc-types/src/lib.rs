use std::{env, io, process::Command};

pub mod event;
pub mod msg;
pub mod reply;

#[derive(Debug)]
pub struct MsgResponse<D> {
    pub msg_type: msg::Msg,
    pub body: D,
}

#[derive(Debug)]
pub struct EventResponse<D> {
    pub evt_type: event::Event,
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
