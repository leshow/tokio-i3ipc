pub use i3ipc_types::*;

use serde::de::DeserializeOwned;

use std::{
    io::{self, Read, Write},
    os::unix::net::UnixStream,
};

pub struct I3;

#[derive(Debug)]
pub struct I3Stream(UnixStream);

impl Connect for I3 {
    type Stream = I3Stream;
    fn connect() -> io::Result<I3Stream> {
        Ok(I3Stream(UnixStream::connect(socket_path()?)?))
    }
}

impl I3Stream {
    pub fn conn_sub<E>(events: E) -> io::Result<Self>
    where
        E: AsRef<[event::Event]>,
    {
        let mut i3 = I3::connect()?;
        i3.subscribe(events)?;
        Ok(i3)
    }

    pub fn subscribe<E>(&mut self, events: E) -> io::Result<reply::Success>
    where
        E: AsRef<[event::Event]>,
    {
        let sub_json = serde_json::to_string(events.as_ref())?;
        self.send_msg(msg::Msg::Subscribe, &sub_json)?;
        let resp: MsgResponse<reply::Success> = self.receive_msg()?;
        dbg!(&resp.body);
        Ok(resp.body)
    }

    pub fn listen(&'_ mut self) -> I3Iter<'_> {
        I3Iter { stream: self }
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
        dbg!(&msg_type);
        dbg!(&String::from_utf8(payload_bytes.clone()).unwrap());
        Ok(MsgResponse {
            msg_type: msg_type.into(),
            body: serde_json::from_slice(&payload_bytes[..])?,
        })
    }

    pub fn receive_evt(&mut self) -> io::Result<event::Evt> {
        let (evt_type, payload_bytes) = self.decode_msg()?;
        <I3Stream as I3IPC>::decode_evt(evt_type, payload_bytes)
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

    pub fn get_workspaces(&mut self) -> io::Result<reply::Workspaces> {
        let buf = self.encode_msg(msg::Msg::Workspaces);
        self.write_all(&buf[..])?;
        let resp: MsgResponse<Vec<reply::Workspace>> = self.receive_msg()?;
        Ok(resp.body)
    }

    pub fn get_outputs(&mut self) -> io::Result<reply::Outputs> {
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

#[derive(Debug)]
pub struct I3Iter<'a> {
    stream: &'a mut I3Stream,
}

impl<'a> Iterator for I3Iter<'a> {
    type Item = io::Result<event::Evt>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.stream.receive_evt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    #[test]
    fn test_subscribe() -> io::Result<()> {
        let mut i3 = I3::connect()?;
        let resp = i3.subscribe(&[event::Event::Window])?;
        for e in i3.listen() {
            let e = e?;
            println!("{:?}", e);
        }
        Ok(())
    }

    #[test]
    fn test_get_workspaces() -> io::Result<()> {
        let mut i3 = I3::connect()?;
        let workspaces = i3.get_tree()?;
        println!("{:?}", workspaces);
        Ok(())
    }

    #[test]
    fn test_get_outputs() -> io::Result<()> {
        let mut i3 = I3::connect()?;
        let outputs = i3.get_outputs()?;
        println!("{:?}", outputs);
        Ok(())
    }
}
