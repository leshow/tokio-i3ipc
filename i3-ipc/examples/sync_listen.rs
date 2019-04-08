use i3_ipc::{
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
