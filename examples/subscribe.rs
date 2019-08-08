#![feature(async_await)]
use futures::{future, stream::Stream, channel::mpsc};
use i3ipc_types::event::{self, Subscribe};
use std::io;

use tokio_i3ipc::I3;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("starting");
    let i3 = I3::connect().await?;
    i3.subscribe([Subscribe::Window]);
    while let Some(result) = rx.next().await {
        println!("received");
        println!("{:#?}", e);
    }
    Ok(())
}
