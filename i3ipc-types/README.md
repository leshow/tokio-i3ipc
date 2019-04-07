# i3ipc-types

This crate includes all the types for interacting with [i3ipc](https://i3wm.org/docs/ipc.html).

I wanted to have a single crate other packages could depend on for the types used in i3 so that no one had to re-implement the same (tedious) work over again.

## Advantages over i3ipc-rs

`i3ipc-rs` packs the event responses into a single enum, this is wasteful if you intend to receive responses to events for anything except workspaces & windows, since there is no indirection in the type definiton. Rust's memory layout for enums makes each variant take up to the size of the largest variant. That means that responses with relatively few fields like `OutputData` would take up as much space as `WindowData`. In this crate, that's not the case. There's a layer of indirection for `WorkspaceData` and `WindowData` so that the minimum variant size remains small.
