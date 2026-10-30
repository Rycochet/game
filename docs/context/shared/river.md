# shared: river

Rivers on the world map, from their chunk's position alone. Only something to look at for now: they don't affect walking, routes or features (the owner's choice), so features can stand on them and no `WORLD_MAP_VERSION` bump was needed. In `shared` because they will probably matter to the rules later. Drawn by client `world_map`.

## Files

- `crates/shared/src/river.rs`: `Rivers`, `River`, constants. Tests.

## Public API

- `MAX_LENGTH` (256 steps: how far away a source can matter).
- `Rivers::new(&WorldParams)`, `.columns()`, `.width()`, `.in_chunk(ChunkPos) -> Option<River>` (traces it each call: callers keep the result).
- `River { points, reaches_sea }`: one grid step apart from the source; `x` carries on past the planet's edge without wrapping.

## Depends on

- `shared::feature` (`CHUNK_SIZE`, `ChunkPos`, `EDGE_MARGIN`, `LAND_TRIES`), `shared::climate`, `shared::rng`, `shared::surface`, `shared::world`.

## Used by

- Client `common` (`CurrentWorld::rivers`), `world_map` (`KnownRivers`).

## Notes

- Chance per chunk by the region's biome (not cooled by height): 3% in deserts up to 30% in rainforests (`chance_percent`; the owner asked for more rivers, fewer where dry and more where wet). Source: the highest of `LAND_TRIES` random dry spots in the chunk. Course: downhill on `Surface::broad_height`, one of 8 neighbours each step, scored by slope times a random pull that belongs to the *place* (so courses wander, while rivers that meet stay together). Ends at low tide's water level (`reaches_sea`), at a low point (the river just stops), or at `MAX_LENGTH`.
- Each is traced alone, so order doesn't matter; merging happens only because downhill leads them to the same points.
- On the test world the median river is about 40 points long, the longest about 150: broad height has many dips. To do (owner's idea): small lakes at those low points.
