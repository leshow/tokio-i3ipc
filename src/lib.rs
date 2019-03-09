use serde::de::DeserializeOwned;
use tokio_uds::UnixStream;

use std::io;

pub mod event;
pub mod msg;
pub mod reply;

#[derive(Debug)]
pub struct I3IPC {
    stream: UnixStream,
}

pub struct MsgResponse<D> {
    msg_type: msg::Msg,
    payload: D,
}

trait I3 {
    fn conn(&self) -> io::Result<String>;
    fn send_msg<P>(&mut self, msg: msg::Msg, payload: P) -> io::Result<()>
    where
        P: AsRef<str>;
    fn receive_msg<D: DeserializeOwned>(&mut self) -> io::Result<MsgResponse<D>>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
