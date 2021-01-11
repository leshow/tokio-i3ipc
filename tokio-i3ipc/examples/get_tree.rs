use std::io;

use tokio_i3ipc::{reply, I3};

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    let mut i3 = I3::connect().await?;
    // this type can be inferred, here is written explicitly:
    let tree: reply::Node = i3.get_tree().await?;
    println!("{:#?}", tree);

    Ok(())
}
