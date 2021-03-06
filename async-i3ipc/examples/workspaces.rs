use std::io;

use async_i3ipc::{reply, I3};

#[async_std::main]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // this type can be inferred, here is written explicitly:
    let worksp: reply::Workspaces = i3.get_workspaces().await?;
    println!("{:#?}", worksp);

    Ok(())
}
