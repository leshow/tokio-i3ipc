# tokio-i3ipc

This crate provides types and functions for working with i3's IPC protocol within tokio. It re-exports the subcrate `i3ipc-types` because it is also used for a synchronous version of the code.

There are many ways you cna interact with this library. You can import an already written future and simply spawn/run it, or you can use the building blocks to construct your own futures:

```rust
use tokio::codec::FramedRead;
use tokio_uds::UnixStream;

use tokio_i3ipc::{AsyncConnect, read_msg_and, I3, EventCodec};

fn run_subscribe() -> io::Result<()> {
    let fut = I3::new()?
        .and_then(move |stream| {
            let buf = stream.encode_msg_json(Msg::Subscribe, events).unwrap();
            tokio::io::write_all(stream, buf)
        })
        .and_then(|(stream, _buf)| {
            read_msg_and::<_, reply::Success>(stream)
        })
        .and_then(move |(stream, _)| {
            let framed = FramedRead::new(stream, EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    // do something with the events
                })
                .map_err(|err| println!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    tokio::spawn(fut);
}
```
