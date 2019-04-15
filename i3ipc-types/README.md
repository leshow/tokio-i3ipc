# i3ipc-types

[![Crate](https://img.shields.io/crates/v/i3ipc-types.svg)](https://crates.io/crates/i3ipc-types)
[![API](https://docs.rs/i3ipc-types/badge.svg)](https://docs.rs/i3ipc-types)

This crate includes all the types for interacting with [i3ipc](https://i3wm.org/docs/ipc.html), along with some undocumented properties I found by browsing i3's source code.

This crate includes the definitions for all i3 ipc message responses, event types, and serialize/deserialize implementations using `serde`. Additionally, I've included traits with default implementations for encoding and decoding for speaking i3's ipc protocol, so long as the type has implemented `io::Read` and `io::Write`.

My goal is to locate all of the type definitions for i3's IPC implementation here, so no one ever has to go through the tedium again. If anything is missing or not working, please fill out an issue or submit a PR, I'm happy to fix things or improve the library in any way I can.

## Advantages over i3ipc-rs

`i3ipc-rs` packs the event responses into a single enum, this is wasteful if you intend to receive responses to events for anything except workspaces & windows, since there is no indirection in the type definiton. Rust's memory layout for enums makes each variant take up to the size of the largest variant. That means that responses with relatively few fields like `OutputData` would take up as much space as `WindowData`. In this crate, that's not the case. There's a layer of indirection for `WorkspaceData` and `WindowData` so that the minimum variant size remains small.
