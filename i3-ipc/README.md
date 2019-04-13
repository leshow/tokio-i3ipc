# i3-ipc (synchronous)

A synchronous i3 IPC library. For async see [tokio-i3ipc](https://github.com/leshow/tokio-i3ipc).

## Subscribe

Subscribing to events is easy:

```rust
use i3ipc_sync::{
    event::{Event, Subscribe},
    I3Stream,
};
use std::io;

fn main() -> io::Result<()> {
    let mut i3 = I3Stream::conn_sub(&[Subscribe::Window, Subscribe::Workspace])?;
    for e in i3.listen() {
        match e? {
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

## Get

Getting information is equally easy, use any `get_*` method or `run_command` to send a message to i3:

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
