use futures::{future, stream::Stream, mpsc};
use i3ipc_types::event::{self, Subscribe};
use std::io;

use tokio_i3ipc::subscribe;

fn main() -> io::Result<()> {
    println!("starting");
    let mut rt = tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
    let (tx, rx) = mpsc::channel(5);
    subscribe(rt.handle(), tx, vec![Subscribe::Window])?;
    let fut = rx.for_each(|e: event::Event| {
        println!("received");
        println!("{:#?}", e);
        future::ok(())
    });
    rt.spawn(fut);
    rt.run().expect("failed runtime");
    Ok(())
}
