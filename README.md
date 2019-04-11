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
    subscribe(rt.handle(), tx, vec![Subscribe::Window])?;
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
use tokio_i3ipc::{AsyncConnect, Subscribe, I3, EventCodec};

pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<Subscribe>,
) -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream: UnixStream| send_sub(stream, events).expect("failed to subscribe"))
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

    rt.spawn(fut);
    Ok(())
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

## Sending Messages to i3

To [send messages](https://i3wm.org/docs/ipc.html#_sending_messages_to_i3) to i3, there are a number of convenience futures that need only be passed a `UnixStream` and then run in your event loop.

```rust
use tokio_i3ipc::{I3, Connect, get, reply};

  I3::connect()
        .expect("unable to get socket")
        .and_then(|stream: UnixStream| get::get_workspaces(stream))
        .and_then(|(_stream, reply: reply::Workspaces)| {
            // do something w/ reply::Workspaces
        })
```

The definition of `get_workspaces` is literally just:

```rust
run_msg::<S, D>(stream, Msg::Workspaces) where S: AsyncI3IPC, D: DeserializeOwned
```

So you could write this yourself very easily.
