# shared: wilderness

The land between pois at walking scale: one endless map per world, `SCALE` (16) cells to a world map point, wrapping east–west with the planet. Made on demand in 64 × 64 blocks from the public terrain (height, climate, vegetation) and each chunk's plan (its poi's cutout). A block nobody has changed can be made again at will, so only players' changes need saving (owner). The server runs visits to it (exploring, a window of blocks round the player, cutouts' stairs, leaving); the client explores with Shift + . and leaves with Shift + , and draws it as a level (plan in `decisions.md`).

## Files

- `crates/shared/src/wilderness.rs`: `Wilderness` (`new`, `width`, `points`), constants. In `wilderness/`: `scale.rs` (`SCALE`, `BLOCK_SIZE`, `BlockPos`, cells against points, chunks and blocks), `samples.rs` (point samples blended across cells), `water.rs` (open water and shallows), `cutout.rs` (`Cutout`), `plan.rs` (`ChunkPlan`), `block.rs` (making cells).

## Public API

- `SCALE` (16), `BLOCK_SIZE` (64), `SHALLOWS_REACH` (8, owner), `MOST_TREES_PERCENT` (35), `CLEARED` (8) and `CLEARED_TO` (24) (owner), `ENTRANCE_CLEARING` (2), `WILDERNESS_VERSION` (`1 + FORGET_BEFORE`).
- `Wilderness::new(&WorldParams)`; `.width()` (cells round the planet), `.points()`, `.wrap(x)`, `.point_of(x, y)` (the world map point a cell is in: where `<` leaves to), `.centre_of(px, py)` (the point's middle cell: where `>` arrives), `.columns()`, `.chunk_of(x, y)`, `.chunk_bounds(chunk)`, `.block_columns()`, `.block_of(x, y)`, `.block_bounds(block)`, `.block(block, &ChunkPlan) -> Level` (the plan of the block's own chunk).
- `BlockPos { x, y }` (serde): cells `x * BLOCK_SIZE` on; the last block before the seam is narrower when the width isn't a multiple of 64. Chunks start on multiples of 64, so a block is in one chunk.
- `ChunkPlan::new(&Wilderness, chunk, Option<(Poi, &Site)>)` (the server passes the site as it saved it), `ChunkPlan::with_new_site(&Wilderness, &PoiMap, chunk)` (the site made as new), `.chunk()`, `.cutout()`.
- `Cutout` (`new(&Wilderness, Poi, &SiteLevel)`): `.poi()`, `.offset()` (site level 0 cell + offset = wilderness cell, both multiples of 8 so cell chunks line up; `None` when only the way down is set in), `.level()` (the cells set in, in wilderness cells), `.stairs()` (ways into the site's levels, in wilderness cells), `.contains(x, y)`.

## Depends on

- `shared::surface`, `climate`, `vegetation`, `world`, `rng`, `poi` (`ChunkPos`, `Poi`, `PoiMap`, `CHUNK_SIZE`), `site` (`Site`, `SiteLevel`, `Stairs`, `Generator`), `dungeon` (`Level`, `Cell`), `movement` (`past_the_ice`), `protocol` (`LEVEL_CHUNK_SIZE`).

## Used by

- `server` (`wilderness::WildernessMaps`, `sites::wild`: visits to it).

## Notes

- Cells: past the ice (80°) `Wall`; open water (blended height below the lowest tide, `sea_level - tide_reach`, unless the nearest point's sea is frozen, which is walkable ground) is `Shallows` within `SHALLOWS_REACH` of land (straight line between middles) and `Water` beyond, so a river up to 16 wide can be waded (owner); land is a `Tree` by a hash of the cell with chance cover × `MOST_TREES_PERCENT`, else `Grass` by low growth, else `Ground`. No trees or growth on snow or water points. Tides are left out for now (owner): the sea is fixed at low water.
- Points stand on the corners of their 16 × 16 cells; height and cover are blended between the four round a cell's middle, sampled once per point for a block plus `SHALLOWS_REACH` round it. Each cell depends only on its position and its chunk's plan, whichever block or rectangle is made (tested).
- The biome isn't stored in cells: the client colours ground from the climate at the cell's world position itself.
- Trees block walking but not sight (owner: memory stays a seen mask). At most 35% of cells, so woods never wall off more than the odd cell (tested: 99% of a thick wood's walkable cells are joined).
- Cutouts (owner): a site's first level set in whole, centred on the poi's middle cell, unless it is entered from above (stairs to the world map, such as a cave by a hole): then only a 5 × 5 clearing of `Ground` with stairs down to level 0 at the poi's middle cell. Kept `CLEARED_TO` inside their chunk, so the clearing never reaches another chunk; trees thin from none within `CLEARED` of a cutout to the land's own at `CLEARED_TO`, round hidden pois too (a gap that size is common in woods, so it gives nothing away).
- To come: rivers (shallows banks, widening downstream), roads and bridges; mobs per block with scent trails; players' changes kept per block.
