use futures::{
    future,
    sink::Sink,
    stream::Stream,
    mpsc::{self, Sender},
    Future,
};
use std::io;
use tokio::codec::FramedRead;
use tokio_uds::UnixStream;

use tokio_i3ipc::{
    codec::EventCodec,
    event::{self, Subscribe},
    subscribe_future, Connect, I3,
};

pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<event::Subscribe>,
) -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| subscribe_future(stream, events))
        .and_then(|(stream, _)| {
            let framed = FramedRead::new(stream, EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
                })
                .map_err(|err| println!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    rt.spawn(fut).expect("Failed to spawn subscribe future");
    Ok(())
}

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
