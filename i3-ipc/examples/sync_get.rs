use i3ipc_sync::{Connect, I3};

use std::io;

fn main() -> io::Result<()> {
    let mut i3 = I3::connect()?;
    let workspaces = i3.get_workspaces()?;
    println!("{:?}", workspaces);
    Ok(())
}
