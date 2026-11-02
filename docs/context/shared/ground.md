# shared: ground

What can be told of something on the ground from afar, on both sides. The rules for putting it there are the server's (`server/server.md`, `ground.rs`).

## Files

- `crates/shared/src/ground.rs`: `Load`. Tests.

## Public API

- `Load::{ALL, of, of_weight}`: how full a bag looks (`Flat`, `Slack`, `Half`, `Full`, `Bulging`), by the weight of all in it. Limits are first guesses. Saved in what characters remember.

## Depends on

- `shared::item`, `serde`.

## Used by

- `protocol`, `server` (`ground`, `storage`), client `ground`.
