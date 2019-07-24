use crate::{io as i3io, *};

use futures::prelude::*;
use futures::{channel::mpsc::Sender, Future};
use serde::de::DeserializeOwned;
use std::io as stio;
use tokio::codec::FramedRead;
use tokio::io::{AsyncRead};

/// Convenience function that decodes a single response and passes the type and undecoded buffer to a closure
pub async fn decode_response<F, T, S>(stream: S, f: F) -> stio::Result<(S, T)>
where
    F: Fn(u32, Vec<u8>) -> T,
    S: AsyncRead + Unpin,
{
    let buf = [0; 14];
    stream.read_exact(&mut buf).await?;
 
    if &buf[0..6] != MAGIC.as_bytes() {
        panic!("Magic str not received");
    }
    let payload_len = u32::from_ne_bytes([buf[6], buf[7], buf[8], buf[9]]) as usize;
    let msg_type = u32::from_ne_bytes([buf[10], buf[11], buf[12], buf[13]]);

    let payload = vec![0; payload_len];
    stream.read_exact(&mut payload).await?;
    Ok((stream, f(msg_type, payload)))
}

/// Decode a response into a [MsgResponse](struct.MsgResponse.html)
pub fn decode_msg<D, S>(
    stream: S,
) -> impl Future<Output = stio::Result<(S, stio::Result<MsgResponse<D>>)>>
where
    D: DeserializeOwned,
    S: AsyncRead + Unpin,
{
    decode_response(stream, MsgResponse::new)
}

/// Decode a response into an [Event](event/enum.Event.html)
pub fn decode_event_future<D, S>(
    stream: S,
) -> impl Future<Output = stio::Result<(S, stio::Result<event::Event>)>>
where
    D: DeserializeOwned,
    S: AsyncRead + Unpin,
{
    decode_response(stream, decode_event)
}

/// Function returns a Future that will send a [Subscribe](event/enum.Subscribe.html) to i3 along with the events to listen to.
/// Is bounded by [AsyncI3IPC](trait.AsyncI3IPC.html) but you should almost always use `UnixStream`
pub async fn subscribe_future<S, E>(
    stream: S,
    events: E,
) -> stio::Result<(S, MsgResponse<reply::Success>)>
where
    S: AsyncI3IPC + Unpin,
    E: AsRef<[event::Subscribe]>,
{
    let s = i3io::write_msg_json(stream, msg::Msg::Subscribe, events.as_ref())?.await?;
    Ok(i3io::read_msg_and::<reply::Success, _>(s).await?)
}

/// An easy-to-use subscribe, all you need to do is pass a runtime handle and a `Sender` half of a channel, then listen on
/// the `rx` side for events
pub fn subscribe(
    rt: tokio::runtime::current_thread::Handle,
    tx: Sender<event::Event>,
    events: Vec<event::Subscribe>,
) -> stio::Result<()> {
    let fut = I3::connect()?
        .and_then(|stream| {
            i3io::write_msg_json(stream, msg::Msg::Subscribe, events).expect("Encoding failed")
        })
        .and_then(i3io::read_msg_and::<reply::Success, _>)
        .and_then(|(stream, _r)| {
            let framed = FramedRead::new(stream, codec::EventCodec);
            let sender = framed
                .for_each(move |evt| {
                    let tx = tx.clone();
                    tx.send(evt)
                        .map(|_| ())
                        .map_err(|e| stio::Error::new(stio::ErrorKind::BrokenPipe, e))
                })
                .map_err(|err| eprintln!("{}", err));
            tokio::spawn(sender);
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| eprintln!("{:?}", e));

    rt.spawn(fut).expect("failed to spawn");
    Ok(())
}
