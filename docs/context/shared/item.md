# shared: item

Ids, kinds of item, and carrying items in grids: the rules both sides check moves by. The server is the only place ids and new items come from.

## Files

- `crates/shared/src/id.rs`: `Id`, a version 6 UUID (text in saves, 16 bytes in messages). Nothing here makes one: `server::ids::new_id` does.
- `crates/shared/src/item.rs`: `Shape`, `Rotation`, `ItemKind` (the table: `def()`), `ItemDef`, `ContainerDef`, `Quality`, `Drop`, `roll_drop`, `artifact_name`, `MadeAt`, `Item`, `Placed`, `Holder`, `Inventory`, `MoveRefused`. Tests for every rule below.

- `crates/shared/src/item_form.rs`: `ItemForm`, `Limits`, `GemForm`, `Cut`, `give_form`, `give_blade_within`, `give_commission`. Tests.
- `crates/shared/src/material.rs`: `Material`, `Category`. Placeholder table, first guesses at tiers. Tests.

## Public API

- `Id(Uuid)`: something's id for good, for items, mobs, characters and accounts (the last two wrapped in `CharacterId` and `AccountId`). Prints as the usual UUID text, which also names files. `Id::for_test` is for tests in any crate. `Id::NOBODY` is nil, no real id.
- `ItemKind::{ALL, def()}`, `ItemDef { name, weight (grams), shape, container, wear, has_quality, make_percent }`, `WearDef` (see `creature.md`), `ContainerDef { width, height, max_total, max_each }`. To add a kind: add it to the enum, `ALL` (alphabetical by name) and `def`. Saved by name.
- `Shape`: up to `MAX_SHAPE_SIDE` (8) a side, as bits of a `u64`, gaps allowed. `rectangle`, `has`, `cells`, `size`, `turned`, `scaled` (for a `Size`), `smaller_than`.
- `Rotation::{None, Quarter, Half, ThreeQuarters}`: clockwise; `next`, `turn_cell`.
- `Quality::{ALL, odds, roll}` (Poor to Artifact, odds out of `ODDS_OUT_OF`), `Drop { quality, kind, weight }`, `roll_drop` (both rolls: quality, then the item at it; a table without that quality takes the nearest below, or else above).
- `Item { id, kind, contents, made, quality, made_at, name, set }` (`name()` is its own name or its kind's with its size unless `Medium`; every artifact has its own, from `artifact_name`, placeholders so far; `shape()` is its kind's at the size it was made: use it, not `def().shape`), `MadeAt { world, minute }` (each world has its own clock), `Made` (who it was made for: `for_body`), `Placed { item, x, y, rotation }` (`shape`, `cells`), `Holder::{Pockets, Inside(id)}`.
- `Inventory { creature, outfitted, pockets, pockets_size, worn, bin }`: `find`, `placed`, `grid`, `grid_size`, `total_weight`, `weight_with_creature`, `check_move`, `move_item`, `check_add`, `add`, `first_place_for_new` (the same for something not carried), `first_place_for` (a closed bag: the first place, turning if it must), `give` (a new item, first place in the pockets), `to_bin`, `to_bin_new`, `in_bin` (`bin` keeps what was last thrown away, and the next destroys it), `move_cost`, `put_down_cost` (what moves take, both sides).
- `give_form(item, size, new_id)`: rolls a new item's form, for kinds with one (gems, rings), going by its quality and its id's seed, and sets its size to what it is told or what it rolled. Everything making items calls it. `Item.form` holds the result, `None` for other kinds and old saves. `Item::set_gems`. `ItemForm::{name, gems}`, `GemForm { material, cut, shape_seed }`, `Cut::{ALL, name}`, `RingForm { metal, profile, width, pattern, settings }`, `Setting { angle, size, mount, gem }` (`takes`), `Profile`, `BandWidth`, `Pattern`, `Mount`.
- `Limits { fits, at_least_long, most_grams, least_grams, wielder }`, `Limits::{NONE, allows}`: what a request needs of an item, so far only used by blades (`give_blade_within`, `give_commission`, which may change an item's kind). `ItemKind::blade_family`: `Axe`, `Dagger`, `Knife`, `Longsword`, `Polearm`, `ShortSword`, `Spear`, `Sword` are rolled as blades (see `blade.md`).
- `Material::{ALL, GEMS, name, category, tier, density}`, `Category::{Bone, Gem, Metal, Wood}`.
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
- Size changes only the outside of an item: each side is multiplied by `Size::quarters` (0.5 to 2), rounding down when enlarging and up when reducing, to at most 8. A blade's weight and cells come from its form, and its weight scales with its size cubed. A bag's grid, its limits and every other weight stay as they are, so a Tiny pouch is 1 × 1 and can hold nothing, and bags aren't rolled at other sizes yet. Saved items may now overlap: harmless, as they can still be moved apart, until the item wipe.
- A form is rolled once by the server and saved, never worked out again, as it will affect value: changing the rolls leaves existing items alone. The client only draws what it is sent. Each kind rolls its choices in order, each seeing those before: changing weights to lean on one (`Rng::pick_weighted`), or a weight of 0 to rule something out. Gems: quality, material, cut, size. Rings: quality, metal, band, number of settings, then each gem: first guesses for all of it. The finest gems never come Giant, by whole-number division.
- A gem set in a ring is a whole item with its own id, inside the ring's form, so taking one out later needs only moving it, and the setting stays. Rolled: about 5% of settings empty, 90% of gems after the first the same as it, 3% one size too big for the setting (owner's request, though they find it ugly), no uncut stones. A ring's quality is how it was made, and a gem has its own. `Inventory::find` doesn't look in settings, as nothing can move them yet: taking them out and putting them in is to come, by `Setting::takes`. Nothing but `holds_or_is` and weights needs to see them so far.
- Kinds are placeholders for trying this out, and the weights are guesses.
