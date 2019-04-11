use serde::{de::DeserializeOwned, Serialize};

use std::{env, io, process::Command};

pub mod event;
pub mod msg;
pub mod reply;

pub trait Connect {
    type Stream: I3IPC;
    fn connect() -> io::Result<Self::Stream>;
}

pub const MAGIC: &str = "i3-ipc";

pub trait I3IPC: io::Read + io::Write {
    const MAGIC: &'static str = MAGIC;

    fn _encode_msg<P>(&self, msg: msg::Msg, payload: Option<P>) -> Vec<u8>
    where
        P: AsRef<str>,
    {
        let mut buf = Vec::with_capacity(14);
        buf.extend(<Self as I3IPC>::MAGIC.as_bytes());
        if let Some(p) = &payload {
            buf.extend(&(p.as_ref().len() as u32).to_ne_bytes());
        } else {
            buf.extend(&(0_u32).to_ne_bytes());
        }
        buf.extend(&<u32 as From<msg::Msg>>::from(msg).to_ne_bytes());
        if let Some(p) = &payload {
            buf.extend(p.as_ref().as_bytes());
        }
        buf
    }

    fn encode_msg(&self, msg: msg::Msg) -> Vec<u8> {
        self._encode_msg::<&str>(msg, None)
    }

    fn encode_msg_body<P>(&self, msg: msg::Msg, payload: P) -> Vec<u8>
    where
        P: AsRef<str>,
    {
        self._encode_msg(msg, Some(payload))
    }

    fn encode_msg_json<P>(&self, msg: msg::Msg, payload: P) -> io::Result<Vec<u8>>
    where
        P: Serialize,
    {
        Ok(self.encode_msg_body(msg, serde_json::to_string(&payload)?))
    }

    fn decode_event(evt_type: u32, payload: Vec<u8>) -> io::Result<event::Event> {
        decode_event(evt_type, payload)
    }

    fn decode_msg(&mut self) -> io::Result<(u32, Vec<u8>)> {
        let mut buf = [0_u8; 6];
        self.read_exact(&mut buf)?;
        if &buf[..] != <Self as I3IPC>::MAGIC.as_bytes() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Expected 'i3-ipc' but received: {:?}", buf),
            ));
        }
        // get payload len
        let mut intbuf = [0_u8; 4];
        self.read_exact(&mut intbuf)?;
        let len = u32::from_ne_bytes(intbuf);
        // get msg type
        let mut msgbuf = [0_u8; 4];
        self.read_exact(&mut msgbuf)?;
        let msgtype = u32::from_ne_bytes(msgbuf);
        // get payload
        let mut payload_buf = vec![0_u8; len as usize];
        self.read_exact(&mut payload_buf)?;
        Ok((msgtype, payload_buf))
    }
}

impl<T: io::Read + io::Write> I3IPC for T {}

#[derive(Debug)]
pub struct MsgResponse<D> {
    pub msg_type: msg::Msg,
    pub body: D,
}

impl<D: DeserializeOwned> MsgResponse<D> {
    pub fn new(msg_type: u32, buf: Vec<u8>) -> io::Result<Self> {
        Ok(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&buf[..])?,
        })
    }
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

pub fn decode_event<P>(evt_type: u32, payload: P) -> io::Result<event::Event>
where
    P: AsRef<[u8]>,
{
    use event::{Event, Subscribe};
    let evt_type = evt_type & !(1 << 31);
    let body = match evt_type.into() {
        Subscribe::Workspace => Event::Workspace(Box::new(serde_json::from_slice::<
            event::WorkspaceData,
        >(payload.as_ref())?)),
        Subscribe::Output => Event::Output(serde_json::from_slice::<event::OutputData>(
            payload.as_ref(),
        )?),
        Subscribe::Mode => {
            Event::Mode(serde_json::from_slice::<event::ModeData>(payload.as_ref())?)
        }

        Subscribe::Window => Event::Window(Box::new(serde_json::from_slice::<event::WindowData>(
            payload.as_ref(),
        )?)),
        Subscribe::BarConfigUpdate => Event::BarConfig(serde_json::from_slice::<
            event::BarConfigData,
        >(payload.as_ref())?),
        Subscribe::Binding => Event::Binding(serde_json::from_slice::<event::BindingData>(
            payload.as_ref(),
        )?),
        Subscribe::Shutdown => Event::Shutdown(serde_json::from_slice::<event::ShutdownData>(
            payload.as_ref(),
        )?),
        Subscribe::Tick => {
            Event::Tick(serde_json::from_slice::<event::TickData>(payload.as_ref())?)
        }
    };
    Ok(body)
}
