# tokio-i3ipc

This crate provides types and functions for working with i3's IPC protocol within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

There are many ways you cna interact with this library. You can import an already written future and simply spawn/run it, or you can use the building blocks to construct your own futures.

I expect the most common use case will be to subscribe to some events and listen over a channel:

```rust
fn main() -> io::Result<()> {
    let mut rt =
        tokio::runtime::current_thread::Runtime::new().expect("Failed building runtime");
    // create a channel to receive responses
    let (tx, rx) = mpsc::channel(5);
    // pass a handle and `Sender` to `subscribe`
    subscribe(rt.handle(), tx, &[Subscribe::Window])?;
    // handle the events received on the channel
    let fut = rx.for_each(|e: event::Event| {
        println!("received");
        println!("{:#?}", e);
        future::ok(())
    });
    rt.spawn(fut);
    rt.run().expect("failed runtime");
    Ok(())
}
```

But all the tools are exported to build something like this yourself, interact with i3 in any way you like:

```rust
use tokio::codec::FramedRead;
use tokio_uds::UnixStream;

use tokio_i3ipc::{AsyncConnect, read_msg_and, I3, EventCodec};

fn run_subscribe() -> io::Result<()> {
    let fut = I3::new()?
        .and_then(move |stream| {
            let buf = stream.encode_msg_json(Msg::Subscribe, events).unwrap(); // methods available on UnixStream thanks to the `AsyncI3IPC` trait
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            read_msg_and::<reply::Success, _>(stream) // decodes a single msg from i3 and passes along the stream
        })
        .and_then(move |(stream, _)| {
            let framed = FramedRead::new(stream, EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    // do something with the events
                })
                .map_err(|err| println!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    tokio::spawn(fut);
}
```

Another example, getting all outputs from i3:

```rust
pub fn get_outputs() -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| run_msg::<reply::Outputs, _>(stream, Msg::Outputs))
        .and_then(|resp| {
            dbg!(resp);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| println!("{}", e));
    Ok(())
}
```

`run_msg` and `run_msg_payload` implement `Future` and do a send/receive operation, the latter with an accompanying payload. These operations send a message to i3 and decode a response that you specify
