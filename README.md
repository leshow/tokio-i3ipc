# i3ipc

## tokio-i3ipc

[![Build Status](https://github.com/leshow/tokio-i3ipc/workflows/Actions/badge.svg)](https://github.com/leshow/tokio-i3ipc/actions)
[![Crate](https://img.shields.io/crates/v/tokio-i3ipc.svg)](https://crates.io/crates/tokio-i3ipc)
[![API](https://docs.rs/tokio-i3ipc/badge.svg)](https://docs.rs/tokio-i3ipc)

This crate provides types and functions for working with i3's IPC protocol (and some basic sway support) within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

see [here](https://github.com/leshow/tokio-i3ipc/tree/master/tokio-i3ipc) for tokio runtime specific i3

## async-i3ipc

[![Crate](https://img.shields.io/crates/v/async-i3ipc.svg)](https://crates.io/crates/async-i3ipc)
[![API](https://docs.rs/async-i3ipc/badge.svg)](https://docs.rs/async-i3ipc)

see [here](https://github.com/leshow/tokio-i3ipc/tree/master/async-i3ipc) for async-std specific i3 ipc (and sway-- not all fields supported)

## std synchronous IO i3ipc

[![Crate](https://img.shields.io/crates/v/i3_ipc.svg)](https://crates.io/crates/i3_ipc)
[![API](https://docs.rs/i3_ipc/badge.svg)](https://docs.rs/i3_ipc)

see [here](https://github.com/leshow/tokio-i3ipc/tree/master/i3-ipc) for synchronous specific i3 ipc (and sway-- not all fields supported)

### Using tokio-i3ipc

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

### Contributing

Contributions PRs, issues, comments, are all welcome!
