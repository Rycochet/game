# shared: ground

What can be told of something on the ground from afar, and who can reach a bag, on both sides. The rules for putting it there are the server's (`server/server.md`, `ground.rs`).

## Files

- `crates/shared/src/ground.rs`: `Load`. In `ground/`: `reach.rs` (reach). Tests.

## Public API

- `Load::{ALL, of, of_weight}`: how full a bag looks (`Flat`, `Slack`, `Half`, `Full`, `Bulging`), by the weight of all in it. Limits are first guesses. Saved in what characters remember.
- `ARM_REACH` (150 units), `reach(size)` (radius plus that), `in_reach(size, at, bag)`, `nearest_in_reach(size, at, bags)` (free positions PR 6): bags lie anywhere, in units; someone reaches every bag within `reach` of their middle, and a drop joins the nearest of them. Anyone standing in a cell's middle reaches all of that cell (its corners are 170 away), and no one Medium or smaller reaches the next cell's middle.

## Depends on

- `shared::item`, `shared::creature` (`Size`), `shared::spot`, `serde`.

## Used by

- `protocol`, `server` (`ground`, `storage`), client `ground` and `inventory`.
