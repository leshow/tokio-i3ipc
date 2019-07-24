use futures::{future, stream::Stream, mpsc};
use i3ipc_types::event::{self, Subscribe};
use std::io;

use tokio_i3ipc::subscribe;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("starting");
    let (tx, rx) = mpsc::channel(5);
    subscribe(tx, vec![Subscribe::Window])?.await?;
    while let Some(result) = rx.next().await {
        println!("received");
        println!("{:#?}", e);
    }
    Ok(())
}
