# shared: item

Ids, kinds of item, and carrying items in grids: the rules both sides check moves by. The server is the only place ids and new items come from.

## Files

- `crates/shared/src/id.rs`: `Id`, a version 6 UUID (text in saves, 16 bytes in messages). Nothing here makes one: `server::ids::new_id` does.
- `crates/shared/src/item.rs`: `Shape`, `Rotation`, `ItemKind` (the table: `def()`), `ItemDef`, `ContainerDef`, `Quality`, `Drop`, `roll_drop`, `artifact_name`, `MadeAt`, `Item`, `Placed`, `Holder`, `Inventory`, `MoveRefused`. Tests for every rule below.

## Public API

- `Id(Uuid)`: something's id for good, for items, mobs, characters and accounts (the last two wrapped in `CharacterId` and `AccountId`). Prints as the usual UUID text, which also names files. `Id::for_test` is for tests in any crate.
- `ItemKind::{ALL, def()}`, `ItemDef { name, weight (grams), shape, container, wear, has_quality }`, `WearDef` (see `creature.md`), `ContainerDef { width, height, max_total, max_each }`. To add a kind: add it to the enum, `ALL` (alphabetical by name) and `def`. Saved by name.
- `Shape`: up to `MAX_SHAPE_SIDE` (8) a side, as bits of a `u64`, gaps allowed. `rectangle`, `has`, `cells`, `size`, `turned`, `smaller_than`.
- `Rotation::{None, Quarter, Half, ThreeQuarters}`: clockwise; `next`, `turn_cell`.
- `Quality::{ALL, odds, roll}` (Poor to Artifact, odds out of `ODDS_OUT_OF`), `Drop { quality, kind, weight }`, `roll_drop` (both rolls: quality, then the item at it; a table without that quality takes the nearest below, or else above).
- `Item { id, kind, contents, made, quality, made_at, name }` (`name()` is its own name or its kind's; every artifact has one, from `artifact_name`, placeholders so far), `MadeAt { world, minute }` (each world has its own clock), `Made` (who it was made for: `for_body`), `Placed { item, x, y, rotation }` (`shape`, `cells`), `Holder::{Pockets, Inside(id)}`.
- `Inventory { creature, outfitted, pockets, pockets_size, worn, bin }`: `find`, `placed`, `grid`, `grid_size`, `total_weight`, `weight_with_creature`, `check_move`, `move_item`, `first_place_for` (a closed bag: the first place, turning if it must), `give` (a new item, first place in the pockets), `to_bin`, `in_bin` (`bin` keeps what was last thrown away, and the next destroys it).
- `POCKETS` (6 × 4, for saves from before creature kinds and new inventories), `MAX_DEPTH` (3).

## Depends on

- `shared::rng`, `shared::world` (`WorldId`), `serde`, `uuid` (MIT/Apache-2.0, with `serde` and `v5` here, for `WorldId`; the server adds what makes new ids).

## Used by

- `protocol`, `server` (`connections`, `storage`, `ids`), client `inventory`, `debug_menu`, `dev_tools`.

## Notes

- Everything carried is one tree starting from the pockets and what is worn: a worn bag's grid works as any other. `placed` is `None` for something worn. An item's id goes with it through any move.
- What may go in a container, in the order refused: not into itself; only a shape with fewer cells that fits within the container's **outside** (so a 2 × 2 pouch holding a 4 × 4 grid takes rings but not a stone, and each step in is smaller, which alone stops endless nesting); no more than `MAX_DEPTH` grids deep, counting grids inside what is moved; `max_each` and `max_total` for every container it would end up in, counting contents at every level; then room in the grid.
- A container it is already in gains no weight from a move within it. Weights are checked outward: a bag inside another counts with all it holds against the outer one's `max_each`.
- Nothing in the table is small enough to go inside a `SmallBox` and have a grid, so `MAX_DEPTH` is only reached by bags made by hand in tests, for now.
- Rolls are the server's: it is the only maker of items. A roll's odds and colours are first guesses; colours are the client's (`inventory`). Quality gives nothing yet. An item's quality may change, and its id doesn't. Design: `decisions.md`, 2026-11-02, item quality.
- Kinds are placeholders for trying this out, and the weights are guesses.
