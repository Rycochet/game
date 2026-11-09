# shared: door

Doors as more than a cell (owner's design, 2026-11-09): every `Cell::Door` and `Cell::SecretDoor` of a site level has a `Door`, rolled from its poi's seed where it stands and then changed by whoever opens, closes or breaks it. Which state counts for the rules is kept on the level (`dungeon::Level::swing`); keeping and changing doors is the server's (`sites/doors`), drawing them the client's (`dungeon_map`).

## Files

- `crates/shared/src/door.rs`: `Door { swing, quality, material, style }` (`look`), `DoorStyle` (only `Plain`: styles are a future research task), `door_cost` (opening or closing: a straight step's time). In `door/`: `swing.rs` (`Swing`, `Side`), `start.rs` (`starting_door`, `door_is_outside`, the closed and broken chances), `look.rs` (`DoorLook`).

## Public API

- `Swing::{Closed, Open(Side), Broken(Side)}`, `is_shut`. Open and broken say which side of the wall the leaf lies on.
- `Side::{North, East, South, West}`: `of(runs_east_west, door, from)` (the side someone is on), `opposite`, `fits`, `beside`. A door opens away from its opener (`of(..).opposite()`); a secret door towards them.
- `Door`, `DoorStyle`, `DoorLook { x, y, swing, material, style }` (what a player is told, in `protocol::LevelChunk::doors`), `door_cost()`.
- `starting_door(poi, ruined, index, at, secret, outside, runs_east_west)`: how a door starts, by a hash of the poi's seed, level and cell; `door_is_outside(level, x, y)` (outside ground beside it).

## Depends on

- `item::Quality`, `material::Material`, `poi`, `dungeon::Level`, `energy`.

## Used by

- `dungeon` (`Level` keeps swings), `protocol` (`DoorLook`, `DoorAsk`), server `sites/doors` and `entities/doors`, client `dungeon_map` (leaves, hover), `player` (opening on the way).

## Notes

- Starting chances (`start.rs`): 70% of doors inside start closed, 90% of those leading outside; herbalists' huts 85% and 95%. A ruin's doors are 30% broken, and half as many of the rest closed. Secret doors always start closed. Quality is always Common and material Wood for now (better doors and other materials are for future generators).
- Not built yet: locks and keys (PR 2), Kick and doors taking damage (PR 3), an animated swing.
