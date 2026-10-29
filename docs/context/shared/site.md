# shared: site

Sites: the maps world map features lead to (buildings, caves, crypts), each one or more levels (`dungeon::Level`) joined by stairs. The feature's kind picks the generator and its options; its map seed decides everything else. Each level is generated on its own, so the client and server agree whichever levels were visited first. Shapes only (walls, doors, floors, ground, water, lava); mobs, items and decorations come later.

## Files

- `crates/shared/src/site/mod.rs`: `Site`, `SiteLevel`, `Stairs`, `StairsDirection`, `Exit`, `Generator` (with `for_feature`: the kind → generator table). Tests: only runnable versions make sites, connectivity and matching stairs for every kind, buildings' doors and wall thickness, caves never rectangular, rooms marked, golden fingerprints; an ignored `print_sites` test prints levels as text (`SITE_KINDS=20,4 SITE_SEED=1 cargo test -p shared print_sites -- --ignored --nocapture`).
- `crates/shared/src/site/building.rs`: `BuildingOptions`, `Outline`, the building `Plan` (outlines per floor, stairs) and BSP floor generation.
- `crates/shared/src/site/cave.rs`: `CaveOptions`, `CaveEntrance`, the cave `Plan` and level generation.
- `crates/shared/src/site/grid.rs`: helpers both use: `roll`, `chance`, `Reach` (flood fill over walkable cells), `all_reached`, `door_nearby`, `hide_some_doors` (secret doors only where nothing depends on them), `dig_to_unreached` (fallback join).

## Public API

- `Site::new(kind, seed)` (the generator's current version), `Site::with_version(kind, seed, version) -> Option<Site>` (`None` if that version can't run any more), `.kind()`, `.is_building()` (floors go up; otherwise levels go down), `.levels()`, `.level(index) -> SiteLevel` (0 is the one entered from the world map).
- `leads_out(level, x, y)`: whether arriving there leaves for the world map: `Ground` on the very edge of the level (beside its rectangle's edge).
- `SiteLevel { level, stairs, entrance }`; `arrival(from: Exit)`: where you arrive coming from the world map (the entrance) or another level (the stairs leading back there); `arrive(from)`: the same, falling back to the first stairs or the entrance (dev tools' jumps between levels).
- `Site::stairs_descend(index, level, stairs) -> (dx, dy)`: the way a staircase goes down, agreeing with the other end (generates the other level to look). Only for looks; moved here from the client so the server can send it.
- `Stairs { x, y, direction, to: Exit }`; `Exit::{WorldMap, Level(n)}`. `StairsDirection` is serde-serialisable (sent in `protocol::SeenStairs`).
- `Generator::{Building(BuildingOptions), Cave(CaveOptions)}`, `Generator::for_feature(kind)`: the options per feature kind (wrappers for tuning a generator); `.version()`.
- `BUILDING_VERSION`, `CAVE_VERSION` (both 1): bump when the generator's output changes in a way that matters (not looks), keeping the old version runnable in `with_version` while saved sites use it. The server saves the version each site was first made with.
- `BuildingOptions { width, height, outlines, square, max_floors, size_per_floor }`, `Outline::{Rectangle, CutCorners, RoundCorners, Circle}`.
- `CaveOptions { size, levels, entrance, doors, water_percent, lava_percent }`, `CaveEntrance::{Hole, Opening, Either}`.

## Depends on

- `shared::dungeon` (`Level`, `Cell`, `RoomId`), `shared::rng`, `shared::feature` (`FeatureKind`).

## Used by

- `server` (`sites`: everything about sites), `shared::protocol` (`StairsDirection`); client `site` (`leads_out`), `dungeon_map` (`StairsDirection`).

## Notes

- Buildings (BSP, after RogueBasin's "Basic BSP Dungeon generation"): the outline (cut and rounded corners and circles in integers) gets a 1-cell outer wall (8-neighbour, so diagonals are closed); the inside is split by 1-cell walls, or near the top of the tree by corridors (wall, 1–2 floor, wall); every final part becomes one room filling it to its walls ("rooms expand to fill the space"), following cut corners with a wall round them. Corridors open into the corridor whose wall they end at. Doors are then added (from corridors first) until everything can be reached, plus a few loops; ~8% of doors become secret where nothing depends on them. The ground floor has a 3-cell `Ground` border and outside doors: the first facing south if it can (you arrive on the ground outside it), plus up to one more per 25 cells of width + height (at most 3), on other sides where possible and at least 6 apart. Inside doors only join what the outside doors don't already reach, so a side door may be the only way into part of the building (walk round outside).
- Floors: up to `min(width, height) / size_per_floor` (and `max_floors`), at random. Each floor up has the same outline: half the time the same size, otherwise 1 cell smaller on every side per 10 of the roll above 50 (1–5). Stairs between two floors are on the same cell of both, chosen in the plan well inside the upper floor; splits never put a wall on them, so each floor can be generated alone. Upper floors are wall outside their outline.
- Caves: ellipses filled 60% at random plus a random walk from the centre, smoothed 3 times by the cellular automata rule, keeping what joins the centre (a cave filling its bounding box loses a corner). Joined by a spanning tree of winding passages (2–6 cell stretches, 30% wandering sideways) plus ~25% extra for loops; anything still unreached gets a passage (or, failing that, a dug straight join). Dug dungeons (`doors`) get doors where passages enter caves through 1-wide gaps, ~10% secret. Water (1–3 pools) and lava pools are kept only if everything stays reachable and they don't touch stairs. Every level picks its own size. Way in: a `Hole` (stairs up to the world map in a random cave) or an `Opening` (3-deep `Ground` as wide as the mouth on the south edge, 35% another on the east or west, with 2-wide passages in). Stairs down are in the cave furthest (by walking) from where you arrive.
- Rooms: every building room and cave has a `RoomId` on its cells (water and lava in a cave keep it); corridors, passages, doors and ground have none.
- Every walkable cell is reachable from where you arrive, without secret doors or crossing water/lava (tested for every kind).
- Changing any constant, the order of random draws, or the kind table changes every site: the golden test fails on purpose.
