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

    fn encode_msg(&self, msg: msg::Msg) -> Vec<u8> {
        let mut buf = Vec::with_capacity(14);
        buf.extend(<Self as I3IPC>::MAGIC.as_bytes());
        buf.extend(&(0_u32).to_ne_bytes());
        buf.extend(&<u32 as From<msg::Msg>>::from(msg).to_ne_bytes());
        buf
    }

    fn encode_msg_body<P>(&self, msg: msg::Msg, payload: P) -> Vec<u8>
    where
        P: AsRef<str>,
    {
        let payload = payload.as_ref();
        let mut buf = Vec::with_capacity(14 + payload.len());
        buf.extend(<Self as I3IPC>::MAGIC.as_bytes());
        buf.extend(&(payload.len() as u32).to_ne_bytes());
        buf.extend(&<u32 as From<msg::Msg>>::from(msg).to_ne_bytes());
        buf.extend(payload.as_bytes());
        buf
    }

    fn decode_evt(evt_type: u32, payload: Vec<u8>) -> io::Result<event::Evt> {
        use event::{Event, Evt};
        let evt_type = evt_type & !(1 << 31);
        dbg!(&evt_type);
        let body = match evt_type.into() {
            Event::Workspace => Evt::Workspace(Box::new(serde_json::from_slice::<
                event::WorkspaceData,
            >(&payload[..])?)),
            Event::Output => {
                Evt::Output(serde_json::from_slice::<event::OutputData>(&payload[..])?)
            }
            Event::Mode => Evt::Mode(serde_json::from_slice::<event::ModeData>(&payload[..])?),
            Event::Window => Evt::Window(Box::new(serde_json::from_slice::<event::WindowData>(
                &payload[..],
            )?)),
            Event::BarConfigUpdate => Evt::BarConfig(
                serde_json::from_slice::<event::BarConfigData>(&payload[..])?,
            ),
            Event::Binding => {
                Evt::Binding(serde_json::from_slice::<event::BindingData>(&payload[..])?)
            }
            Event::Shutdown => {
                Evt::Shutdown(serde_json::from_slice::<event::ShutdownData>(&payload[..])?)
            }
            Event::Tick => Evt::Tick(serde_json::from_slice::<event::TickData>(&payload[..])?),
        };
        Ok(body)
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
