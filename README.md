# tokio-i3ipc (pre-release)

[![Build Status](https://travis-ci.com/leshow/tokio-i3ipc.svg?branch=master)](https://travis-ci.com/leshow/tokio-i3ipc)
[![Crate](https://img.shields.io/crates/v/tokio-i3ipc.svg)](https://crates.io/crates/tokio-i3ipc)
[![API](https://docs.rs/tokio-i3ipc/badge.svg)](https://docs.rs/tokio-i3ipc)

This crate provides types and functions for working with i3's IPC protocol within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

There are many ways you cna interact with this library. You can import an already written future and simply spawn/run it, or you can use the building blocks to construct your own futures.

I expect the most common use case will be to subscribe to some events and listen over a channel:

```rust
use futures::{
    future,
    sink::Sink,
    stream::Stream,
    sync::mpsc::{self, Sender},
    Future,
};
use std::io;
use tokio_i3ipc::{subscribe, event::{self, Subscribe}};

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
use futures::{future::Future, stream::Stream, sink::Sink, sync::mpsc::Sender};
use std::io;

use tokio_i3ipc::{Connect, subscribe_future, event, I3, codec::EventCodec};

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

    rt.spawn(fut);
    Ok(())
}
```

Another example, getting all displays from i3:

```rust
use std::io;
use futures::future::Future;
use tokio_i3ipc::{get, I3, Connect};

pub fn get_displays() -> io::Result<()> {
    let fut = I3::connect()?
        .and_then(get::get_outputs)
        .and_then(|resp| {
            dbg!(resp);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| println!("{}", e));
    Ok(())
}
```

or, you could write `get_outputs` yourself:

```rust
use tokio_uds::UnixStream;
use futures::future::Future;
use std::io;
use tokio_i3ipc::{reply, msg::Msg, MsgResponse, event, io as i3io};

pub fn get_outputs(
    stream: UnixStream,
) -> impl Future<Item = (UnixStream, MsgResponse<reply::Outputs>), Error = io::Error> {
    i3io::send_msg(stream, Msg::Outputs).and_then(i3io::read_msg_and)
}
```

`send_msg`, `write_msg_json` and `write_msg` will handle writing to i3. `read_msg` and `read_msg_and` will handle reading. The latter returns the stream again to continue using it.

## Sending Messages to i3

To [send messages](https://i3wm.org/docs/ipc.html#_sending_messages_to_i3) to i3, there are a number of convenience futures that need only be passed a `UnixStream` and then run in your event loop.

```rust
use futures::future::Future;
use tokio_uds::UnixStream;
use tokio;
use tokio_i3ipc::{I3, Connect, MsgResponse, get, reply};

fn main() {
    let fut = I3::connect()
        .expect("unable to get socket")
        .and_then(get::get_workspaces)
        .and_then(
            |(_stream, reply): (UnixStream, MsgResponse<reply::Workspaces>)| {
                // do something w/ reply::Workspaces
                futures::future::ok(())
            },
        )
        .map(|_| ())
        .map_err(|_| ());
    tokio::run(fut);
}
```
