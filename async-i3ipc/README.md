# async-i3ipc

[![Crate](https://img.shields.io/crates/v/async-i3ipc.svg)](https://crates.io/crates/async-i3ipc)
[![API](https://docs.rs/async-i3ipc/badge.svg)](https://docs.rs/async-i3ipc)

This crate provides types and functions for working with i3's IPC protocol within async-std. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

I expect the most common use case will be to subscribe to some events and listen:

```rust
use async_i3ipc::{
    event::{Event, Subscribe},
    I3,
};
use std::io;

// prefer creating the runtime yourself and using only a single core.
// this will hardly need to have a runtime with 1 thread for each core
// on your machine
#[async_std::main]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    let resp = i3.subscribe([Subscribe::Window]).await?;

    println!("{:#?}", resp);
    let mut listener = i3.listen();
    while let Ok(event) = listener.next().await {
        match event {
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
