# tokio-i3ipc

[![Build Status](https://travis-ci.com/leshow/tokio-i3ipc.svg?branch=master)](https://travis-ci.com/leshow/tokio-i3ipc?branch=master)
[![Crate](https://img.shields.io/crates/v/tokio-i3ipc.svg)](https://crates.io/crates/tokio-i3ipc)
[![API](https://docs.rs/tokio-i3ipc/badge.svg)](https://docs.rs/tokio-i3ipc)

This crate provides types and functions for working with i3's IPC protocol within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

I expect the most common use case will be to subscribe to some events and listen:

```rust
use std::io;
use tokio::stream::StreamExt;
use tokio_i3ipc::{
    event::{Event, Subscribe},
    I3,
};

#[tokio::main(basic_scheduler)]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    let resp = i3.subscribe([Subscribe::Window]).await?;

    println!("{:#?}", resp);
    let mut listener = i3.listen();
    while let Some(event) = listener.next().await {
        match event? {
            Event::Workspace(ev) => println!("workspace change event {:?}", ev),
            Event::Window(ev) => println!("window event {:?}", ev),
            Event::Output(ev) => println!("output event {:?}", ev),
            Event::Mode(ev) => println!("mode event {:?}", ev),
            Event::BarConfig(ev) => println!("bar config update {:?}", ev),
            Event::Binding(ev) => println!("binding event {:?}", ev),
            Event::Shutdown(ev) => println!("shutdown event {:?}", ev),
            Event::Tick(ev) => println!("tick event {:?}", ev),
        }
    }
    Ok(())
}
```

Another example, getting all workspaces from i3:

```rust
use std::io;
use tokio_i3ipc::{reply, I3};

#[tokio::main(basic_scheduler)]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // this type can be inferred, here is written explicitly:
    let worksp: reply::Workspaces = i3.get_workspaces().await?;
    println!("{:#?}", worksp);

    Ok(())
}
```

or, you could write any `get_*` yourself using the same methods it does:

```rust
use std::io;
use tokio_i3ipc::{msg, reply, MsgResponse, I3};

#[tokio::main(basic_scheduler)]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // send msg RunCommand with a payload
    let payload = "some_command";
    i3.send_msg_body(msg::Msg::RunCommand, payload).await?;
    let resp: MsgResponse<Vec<reply::Success>> = i3.read_msg().await?;
    Ok(())
}
```

`send_msg`, will handle writing to i3. `read_msg` and will handle reading.

## Sending Messages to i3

To [send messages](https://i3wm.org/docs/ipc.html#_sending_messages_to_i3) to i3, there are a number of convenience methods on `I3`.

```rust
use std::io;
use tokio_i3ipc::{reply, I3};

#[tokio::main(basic_scheduler)]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // this type can be inferred, here is written explicitly:
    let outputs = i3.get_outputs().await?;
    println!("{:#?}", worksp);

    Ok(())
}
```

### "Real world" example

I have a fork of an i3 window-logging application that uses `tokio-i3ipc` (https://github.com/leshow/i3-tracker-rs/). The tracker subscribes to window events and logs how much time is spent on each node.
