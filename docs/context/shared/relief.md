# shared: relief

The ground's shape on a level: a height at every grid corner, in whole units (`UNITS_PER_CELL`, 240 a cell, as positions will be), with the surface between them four flat triangles per cell meeting at its middle (the average of its corners). Which ground a level has is one rule both sides follow (`LevelGround`), worked out by the server and sent with the level. Moves on levels are priced on it (`movement::LevelWay`: slopes along a move, sampled every quarter cell), on both sides: the client on `common::DrawnGround`'s, the server on each population's (`Population::relief`).

## Files

- `crates/shared/src/relief.rs`: `Relief` (`flat`, `from_fn`, `of_land`, `bounds`, `covers`, `corner`, `corners_of`, `centre`, `at`, `level_stairs`), `UNITS_PER_CELL`, `UNITS_PER_16TH`. In `relief/`: `level_ground.rs` (`LevelGround`), `cave.rs` (a cave's swell). Tests: corners and middles on the surface, no steps in it, stairs level, a cave's swell and pools, and a site's first level agreeing with its wilderness cutout corner by corner.

## Public API

- `Relief`: corner `(x, y)` is cell `(x, y)`'s north-west corner; `corner(x, y)` (0 beyond it), `corners_of(x, y)` (north-west, north-east, south-west, south-east), `centre(x, y)` (their average, rounded), `at(ux, uy)` (a point in units, cell `(x, y)`'s middle at `(x, y) × 240`, on the triangle towards the nearest side), `covers(x, y)` (a cell). `of_land(&WildHeights, bounds, offset, stairs)`: each corner from `WildHeights::corner` at the cell plus `offset`, stairs cells levelled. `level_stairs(cells)`: each cell's corners set to its middle.
- `UNITS_PER_16TH` (30): a 16th of an elevation is an eighth of a cell (owner, 2026-11-08).
- `LevelGround` (serde, in `LevelInfo::ground`): `Flat` (buildings, the Hall, the default), `Wilderness` (the client's window: `Relief::of_land` with no offset), `Land { offset, flats }` (a site's first level set in whole: the land where it is set in, with its cutout's flats), `Cave { seed }`. `of_site(&Wilderness, &WorldGround, (poi, &site), index, &SiteLevel)`: the rule (first level set in whole: `Land`; a cave's (`Site::is_cave`) other levels: `Cave`; else `Flat`). `relief(&WorldGround, &Level, stairs)`: the heights, `None` for `Flat` and `Wilderness`.
- `CAVE_SWELL` (60, a quarter of a cell), `ROOM_SWELL` (12): how far a cave's floor rises or falls, outside and inside its caves (`RoomId`).

## Depends on

- `movement` (`WildHeights`, `FlatGround`, `WorldGround`), `wilderness` (`Cutout`, `Wilderness`), `site` (`Site`, `SiteLevel`), `dungeon` (`Level`), `poi` (`Poi`), `rng` (`Seed`), `serde`.

## Used by

- `protocol` (`LevelInfo::ground`), `movement::LevelWay` (move costs), `server` (`sites::levels`: each site level's ground; the wilderness's is `Wilderness`; populations' `relief` for move costs and reach), client `player` (`keep_drawn_ground`), `common` (`DrawnGround`), `dungeon_map` (draping), `movement` (`UNITS_PER_16TH`).

## Notes

- Land corners blend the world map points round them exactly (points stand on the corners of their 16 × 16 cells), so a cell's corners average to its middle's blend. A corner beside a levelled cell (a flat) takes the flat's height, the highest if several, so open ground slopes up to buildings and walls without a step.
- A cave's swell is hashed from a constant and the poi's place and level, never the poi's map seed (that is secret, and SplitMix's mixing can be undone). It is blended between points 4 cells apart, cut to a fifth inside caves and 0 round liquid, so pools lie flat.
- Positions (PR 2 of `free-positions.md`) can stand on `at` in whole units; the client draws the same triangles in floats (`common::DrawnGround::drawn_between`).
- Client and server may build slightly different ground, harmlessly: the client levels the stairs it has seen and, inside a flat, the cells it hasn't (an unknown cell counts as a wall), where the server knows them all. Only those cells' corners differ, so a move beside unseen stairs may cost the server a little more or less than the client predicted; the server's energy is authoritative and only paces moves (one taken a few ticks later, within the client's quarter second ahead), never refuses one. Once seen, they agree.
