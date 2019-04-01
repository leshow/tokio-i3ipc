# I3IPC (synchronous)

This crate is for the synchronous API to i3.

Subscribing to events is easy:

```rust
fn main() -> io::Result<()> {
    let mut i3 = I3::connect()?;
    let resp = i3.subscribe(&[Event::Window])?;
    // alternatively, let i3 = I3Stream::conn_sub(&[Event::Window])?;
    for e in i3.listen() {
        match e? { // each item is a io::Result
            Evt::Workspace(ev) => println!("workspace change event {:?}", ev),
            Evt::Window(ev) => println!("window event {:?}", ev),
            Evt::Output(ev)=> println!("output event {:?}", ev),
            Evt::Mode(ev)=> println!("mode event {:?}", ev),
            Evt::BarConfig(ev)=> println!("bar config update {:?}", ev),
            Evt::Binding(ev)=> println!("binding event {:?}", ev),
            Evt::Shutdown(ev)=> println!("shutdown event {:?}", ev),
            Evt::Tick(ev) => println!("tick event {:?}", ev),
        }
    }
}
```

Get all active workspaces & tree of nodes:

```rust
fn test_get_workspaces() -> io::Result<()> {
    let mut i3 = I3::connect()?;
    let workspaces = i3.get_workspaces()?;
    println!("{:?}", workspaces);

    let nodes = i3.get_tree()?;
    println!("{:?}", nodes);

    Ok(())
}
```
