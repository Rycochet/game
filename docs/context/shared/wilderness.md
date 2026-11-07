# shared: wilderness

The land between pois at walking scale: one endless map per world, `SCALE` (16) cells to a world map point, wrapping east–west with the planet. Made on demand in 64 × 64 blocks from the public terrain (height, climate, vegetation) and each chunk's plan (its poi's cutout). A block nobody has changed can be made again at will, so only players' changes need saving (owner). The server runs visits to it (exploring, a window of blocks round the player, cutouts' stairs, leaving); the client explores with Shift + . and leaves with Shift + , and draws it as a level (plan in `decisions.md`).

## Files

- `crates/shared/src/wilderness.rs`: `Wilderness` (`new`, `width`, `points`), constants. In `wilderness/`: `scale.rs` (`SCALE`, `BLOCK_SIZE`, `BlockPos`, cells against points, chunks and blocks), `samples.rs` (point samples blended across cells), `water.rs` (open water, sea and rivers, and shallows), `course.rs` (rivers and roads as segments in cells), `gate.rs` (`GateSpot`), `cutout.rs` (`Cutout`), `plan.rs` (`ChunkPlan`), `block.rs` (making cells).

## Public API

- `SCALE` (16), `BLOCK_SIZE` (64), `SHALLOWS_REACH` (8, owner), `MOST_TREES_PERCENT` (35), `CLEARED` (8) and `CLEARED_TO` (24) (owner), `RIVER_CLEARED` (1) and `RIVER_CLEARED_TO` (4), `ROAD_CLEARED` (1) and `ROAD_CLEARED_TO` (6), `ENTRANCE_CLEARING` (2), `WILDERNESS_VERSION` (`1 + FORGET_BEFORE`).
- `Wilderness::new(&WorldParams)`; `.width()` (cells round the planet), `.points()`, `.wrap(x)`, `.point_of(x, y)` (the world map point a cell is in: where `<` leaves to), `.centre_of(px, py)` (the point's middle cell: where `>` arrives), `.columns()`, `.chunk_of(x, y)`, `.chunk_bounds(chunk)`, `.block_columns()`, `.block_of(x, y)`, `.block_bounds(block)`, `.block(block, &ChunkPlan) -> Level` (the plan of the block's own chunk).
- `BlockPos { x, y }` (serde): cells `x * BLOCK_SIZE` on; the last block before the seam is narrower when the width isn't a multiple of 64. Chunks start on multiples of 64, so a block is in one chunk.
- `GateSpot { entrance, cell }` (`new(&Wilderness, entrance, &WorldGate)`, `contains`, `near`): a World Gate in the wilderness, its frame on `cell` (the middle cell of its point), `GATE_PAD` (1) of open `Ground` round it. `gates_of(&Wilderness, &PoiMap)`: the world's, by entrance.
- `ChunkPlan::new(&Wilderness, chunk, Option<(Poi, &Site)>, gates: &[GateSpot], roads: &[&Road])` (the server passes the site as it saved it, the world's gates (those within `CLEARED_TO` are kept, so a gate's clearing carries into the next chunk) and the roads it has worked out near; rivers are traced here, from every chunk within `river::MAX_LENGTH`), `ChunkPlan::with_new_site(&Wilderness, &PoiMap, chunk)` (the site made as new, no roads), `.chunk()`, `.cutout()`, `.gates()`, `.has_rivers()`, `.has_roads()`.
- `Cutout` (`new(&Wilderness, Poi, &SiteLevel)`, `Clone`): `.poi()`, `.offset()` (site level 0 cell + offset = wilderness cell, both multiples of 8 so cell chunks line up; `None` when only the way down is set in), `.level()` (its site level's rectangle, in wilderness cells), `.stairs()` (ways into the site's levels, in wilderness cells), `.contains(x, y)` (the cells set in: for a cave's set-in level, not its outside ground joined to the edge, which stays the land's, so a cave mouth's hill stands in the land itself).

## Depends on

- `shared::surface`, `climate`, `vegetation`, `world`, `rng`, `poi` (`ChunkPos`, `Poi`, `PoiMap`, `CHUNK_SIZE`), `site` (`Site`, `SiteLevel`, `Stairs`, `Generator`), `dungeon` (`Level`, `Cell`), `movement` (`past_the_ice`), `protocol` (`LEVEL_CHUNK_SIZE`).

## Used by

- `server` (`wilderness::WildernessMaps`, `sites::wild`: visits to it).

## Notes

- Cells: past the ice (80°) `Wall`; open water (blended height below the lowest tide, `sea_level - tide_reach`, unless the nearest point's sea is frozen, which is walkable ground) is `Shallows` within `SHALLOWS_REACH` of land (straight line between middles) and `Water` beyond, so a river up to 16 wide can be waded (owner); land is a `Tree` by a hash of the cell with chance cover × `MOST_TREES_PERCENT`, else `Grass` by low growth, else `Ground`. No trees or growth on snow or water points. Tides are left out for now (owner): the sea is fixed at low water.
- Points stand on the corners of their 16 × 16 cells; height and cover are blended between the four round a cell's middle, sampled once per point for a block plus `SHALLOWS_REACH` round it. Each cell depends only on its position and its chunk's plan, whichever block or rectangle is made (tested).
- The biome isn't stored in cells: the client colours ground from the climate at the cell's world position itself.
- A World Gate's cell takes players into the Hall (the server's `follow`); mobs never stand on one (`movement::mob_can_stand`), and the client's routes only cross one that is their goal.
- Trees block walking but not sight (owner: memory stays a seen mask). At most 35% of cells, so woods never wall off more than the odd cell (tested: 99% of a thick wood's walkable cells are joined).
- Cutouts (owner): a site's first level set in whole, centred on the poi's middle cell, unless it is entered from above (stairs to the world map, such as a cave by a hole): then only a 5 × 5 clearing of `Ground` with stairs down to level 0 at the poi's middle cell. A cave mouth's level is a hill (`site`), and only the hill is set in; the rest of its rectangle is land (but trees still thin by the rectangle, and the server's mirrors and "poi known" go by the rectangle too: harmless while nothing can be changed there, to make exact once digging comes). Kept `CLEARED_TO` inside their chunk, so the clearing never reaches another chunk; trees thin from none within `CLEARED` of a cutout to the land's own at `CLEARED_TO`, round hidden pois too (a gap that size is common in woods, so it gives nothing away).
- Rivers and roads (`course.rs`): their world map points as cells (the middle of each point, moved up to 5 cells for rivers, 3 for roads, by a hash of where the point is round the planet, so every chunk and copy agrees), joined into segments with a half width at each end. A river is 1 cell each side at its source and widens a cell each side every 16 steps, to 8.5 (17 across: its middle is then out of the shallows' reach). Its cells are water, so shallow within 8 of a bank. Roads by kind: path dirt track (half width 0.6), track gravel (1.1), road cobbles (1.6), highway flagstones (2.1); over water a `Bridge`. Order per cell: a gate's pad (`WorldGate` on its cell, `Ground` round it, cleared as a cutout is), cutout, past the ice, road (or bridge), water, ground. Trees thin from none at a river's edge to normal by 4, and from a road's by 6.
- `Wilderness::biome_of(block)`: the biome at a block's middle, for who lives there (`mob::wild_spawns`).
- Scent laid ahead: a block's new mobs have lately walked a trail from their den and back (`mob::wild_trail`), into the blocks round it; the server lays it as aged marks.
