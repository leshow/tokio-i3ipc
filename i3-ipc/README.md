# I3IPC (synchronous)

This crate is for the synchronous API to i3.

Subscribing to events is easy:

```rust
use i3ipc_sync::{
    event::{Event, Evt},
    I3Stream,
};
use std::io;

fn main() -> io::Result<()> {
    let mut i3 = I3Stream::conn_sub(&[Event::Window, Event::Workspace])?;
    for e in i3.listen() {
        match e? {
            Evt::Workspace(ev) => println!("workspace change event {:?}", ev),
            Evt::Window(ev) => println!("window event {:?}", ev),
            Evt::Output(ev) => println!("output event {:?}", ev),
            Evt::Mode(ev) => println!("mode event {:?}", ev),
            Evt::BarConfig(ev) => println!("bar config update {:?}", ev),
            Evt::Binding(ev) => println!("binding event {:?}", ev),
            Evt::Shutdown(ev) => println!("shutdown event {:?}", ev),
            Evt::Tick(ev) => println!("tick event {:?}", ev),
        }
    }
    Ok(())
}
```

Get all active workspaces & tree of nodes:

```rust
use i3ipc_sync::{Connect, I3};
use std::io;

fn main() -> io::Result<()> {
    let mut i3 = I3::connect()?;
    let workspaces = i3.get_workspaces()?;
    println!("{:?}", workspaces);
    Ok(())
}
```
