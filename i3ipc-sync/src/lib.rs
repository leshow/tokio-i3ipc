pub use i3ipc_types::*;

use serde::de::DeserializeOwned;

use std::{
    env,
    io::{self, Read, Write},
    os::unix::net::UnixStream,
    process::Command,
};

pub struct I3Connect;

// impl I3Connect {
//     pub fn
// }

#[derive(Debug)]
pub struct I3Stream(UnixStream);

impl I3Stream {
    pub const MAGIC: &'static str = "i3-ipc";

    pub fn subscribe<E>(&mut self, events: E) -> io::Result<MsgResponse<reply::Success>>
    where
        E: AsRef<[event::Event]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg(msg::Msg::Subscribe, &sub_json)?;
        let resp: MsgResponse<reply::Success> = self.receive_msg()?;
        Ok(resp)
    }

    pub fn encode_msg<P>(&mut self, msg: msg::Msg, payload: P) -> Vec<u8>
    where
        P: AsRef<str>,
    {
        let payload = payload.as_ref();
        let mut buf = Vec::with_capacity(14 + payload.len());
        buf.extend(I3Stream::MAGIC.as_bytes());
        buf.extend(&(payload.len() as u32).to_ne_bytes());
        buf.extend(&<u32 as From<msg::Msg>>::from(msg).to_ne_bytes());
        buf.extend(payload.as_bytes());
        buf
    }
    pub fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<usize>
    where
        P: AsRef<str>,
    {
        let buf = self.encode_msg(msg, payload);
        self.write(&buf[..])
    }

    pub fn decode_msg(&mut self) -> io::Result<(u32, Vec<u8>)> {
        let mut buf = [0_u8; 6];
        self.read_exact(&mut buf)?;
        if &buf[..] != I3Stream::MAGIC.as_bytes() {
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
        let msgtype = u32::from_be_bytes(msgbuf);
        // get payload
        let mut payload_buf = vec![0_u8; len as usize];
        self.read_exact(&mut payload_buf)?;
        Ok((msgtype, payload_buf))
    }

    pub fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>> {
        let (msg_type, payload_bytes) = self.decode_msg()?;
        Ok(MsgResponse {
            msg_type: msg_type.into(),
            payload: serde_json::from_slice(&payload_bytes[..])?,
        })
    }

    pub fn send_receive<P, D>(&mut self, msg: msg::Msg, payload: P) -> io::Result<MsgResponse<D>>
    where
        P: AsRef<str>,
        D: DeserializeOwned,
    {
        self.send_msg(msg, payload)?;
        self.receive_msg()
    }
}

// pub fn subscribe<E, D>(&mut self, events: E) -> io::Result<EventResponse<D>>
// where
//     E: AsRef<[event::Event]>,
//     D: DeserializeOwned,
// {
//     let sub_json = serde_json::to_string(events.as_ref())?;
//         self.send_msg(msg::Msg::Subscribe, &sub_json)?;
//         let resp: MsgResponse<reply::Success> = self.receive_msg()?;
//         unimplemented!()
// }

impl Read for I3Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for I3Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
