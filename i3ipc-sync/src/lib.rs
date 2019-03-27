pub use i3ipc_types::*;

use serde::de::DeserializeOwned;

use std::{
    io::{self, Read, Write},
    os::unix::net::UnixStream,
};

pub struct I3Connect;

impl I3Connect {
    pub fn connect() -> io::Result<I3Stream> {
        Ok(I3Stream(UnixStream::connect(socket_path()?)?))
    }
}

#[derive(Debug)]
pub struct I3Stream(UnixStream);

impl I3Stream {
    pub const MAGIC: &'static str = "i3-ipc";

    fn encode_msg_body<P>(&mut self, msg: msg::Msg, payload: P) -> Vec<u8>
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

    fn encode_msg(&mut self, msg: msg::Msg) -> Vec<u8> {
        let mut buf = Vec::with_capacity(14);
        buf.extend(I3Stream::MAGIC.as_bytes());
        buf.extend(&(0_u32).to_ne_bytes());
        buf.extend(&<u32 as From<msg::Msg>>::from(msg).to_ne_bytes());
        buf
    }

    fn decode_msg(&mut self) -> io::Result<(u32, Vec<u8>)> {
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

    pub fn subscribe<E>(&mut self, events: E) -> io::Result<MsgResponse<reply::Success>>
    where
        E: AsRef<[event::Event]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg(msg::Msg::Subscribe, &sub_json)?;
        let resp: MsgResponse<reply::Success> = self.receive_msg()?;
        Ok(resp)
    }

    pub fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<usize>
    where
        P: AsRef<str>,
    {
        let buf = self.encode_msg_body(msg, payload);
        self.write(&buf[..])
    }

    pub fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>> {
        let (msg_type, payload_bytes) = self.decode_msg()?;
        Ok(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload_bytes[..])?,
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

    pub fn run_command<S: AsRef<str>>(&mut self, payload: S) -> io::Result<Vec<reply::Success>> {
        self.send_msg(msg::Msg::RunCommand, payload)?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_workspaces(&mut self) -> io::Result<Vec<reply::Workspace>> {
        let buf = self.encode_msg(msg::Msg::Workspaces);
        self.write_all(&buf[..])?;
        let resp: MsgResponse<Vec<reply::Workspace>> = self.receive_msg()?;
        Ok(resp.body)
    }

    pub fn get_outputs(&mut self) -> io::Result<Vec<reply::Outputs>> {
        // self.send_msg(msg::Msg::Outputs, "")?;
        let buf = self.encode_msg(msg::Msg::Outputs);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_tree(&mut self) -> io::Result<reply::Node> {
        // self.send_msg(msg::Msg::Tree, "")?;
        let buf = self.encode_msg(msg::Msg::Tree);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_marks(&mut self) -> io::Result<reply::Marks> {
        // self.send_msg(msg::Msg::Marks, "")?;
        let buf = self.encode_msg(msg::Msg::Marks);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_bar_ids(&mut self) -> io::Result<reply::BarIds> {
        // self.send_msg(msg::Msg::BarConfig, "")?;
        let buf = self.encode_msg(msg::Msg::BarConfig);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_bar_config<S: AsRef<str>>(&mut self, bar_id: S) -> io::Result<reply::BarConfig> {
        self.send_msg(msg::Msg::BarConfig, bar_id)?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_version(&mut self) -> io::Result<reply::Version> {
        // self.send_msg(msg::Msg::Version, "")?;
        let buf = self.encode_msg(msg::Msg::Version);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_binding_modes(&mut self) -> io::Result<reply::BindingModes> {
        // self.send_msg(msg::Msg::BindingModes, "")?;
        let buf = self.encode_msg(msg::Msg::BindingModes);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_config(&mut self) -> io::Result<reply::Config> {
        let buf = self.encode_msg(msg::Msg::Config);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_tick(&mut self) -> io::Result<reply::Success> {
        let buf = self.encode_msg(msg::Msg::Tick);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
    }

    pub fn get_sync(&mut self) -> io::Result<reply::Success> {
        let buf = self.encode_msg(msg::Msg::Sync);
        self.write_all(&buf[..])?;
        Ok(self.receive_msg()?.body)
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
