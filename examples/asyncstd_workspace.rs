use async_std::{io, net::TcpStream, prelude::*, task};
use std::io;

use tokio_i3ipc::{
    event::{Event, Subscribe},
    I3,
};

fn main() -> io::Result<()> {
    task::block_on(async {
        let mut i3 = I3::connect().await?;
        // this type can be inferred, here is written explicitly:
        let worksp: reply::Workspaces = i3.get_workspaces().await?;
        println!("{:#?}", worksp);

        Ok(())
    })
}
