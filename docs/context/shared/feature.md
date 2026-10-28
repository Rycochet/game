# shared: feature

World map features: the world grid is cut into chunks, and each chunk has at most one feature (a cave, house, castle, crypt...), each leading to its own map (dungeon, building, town) later. Everything about a feature comes from its chunk's position alone, so client and server agree and it doesn't matter which chunks were looked at first. Features stand only on dry land (ground above high tide). Generating the feature's map is not here yet.

## Files

- `crates/shared/src/feature.rs`: `FeatureMap`, `Feature`, `FeatureKind`, `ChunkPos`, constants. Golden and invariant tests.

## Public API

- Constants: `CHUNK_SIZE` (64 grid points), `EDGE_MARGIN` (8: features stand at least this far inside their chunk, so two are at least 16 apart, a token's width), `FEATURE_CHANCE_PERCENT` (80, the owner's choice), `LAND_TRIES` (8 random spots tried for dry land; a chunk with none has no feature, so seas and coasts have fewer).
- `FeatureKind`: 34 kinds (`Boulders` … `Mausoleum`), named after the tiles `l2_terrain066`–`099` and in their order; `ALL`. All equally likely for now.
- `ChunkPos { x, y }`: chunk `x` is always `0..columns()` (wraps), `y` unbounded.
- `Feature { kind, chunk, x, y, seed }`: `x, y` its exact world grid position; `seed` is what its map is generated from.
- `FeatureMap::new(&WorldParams)` (keeps its own `Surface`), `.columns()`, `.chunk_at(x, y)`, `.in_chunk(chunk) -> Option<Feature>` (`x` in `0..width`), `.in_area(min, max) -> Vec<Feature>` (inclusive, any planet-width frame; results in the caller's frame, sorted by `(y, x)`).

## Depends on

- `shared::rng`, `shared::surface` (ground height), `shared::world` (`high_water`).

## Used by

- Client `common` (`CurrentWorld::features`), `world_map` (feature tokens).

## Notes

- Per chunk: `Seed::new(world.derive("features").at(&[cx, cy])).rng()`, then in order: the chance roll, the kind, the map seed, then up to `LAND_TRIES` `x`/`y` spots until one is above `WorldParams::high_water()`. Changing that order or any constant changes every world (golden test).
- East–west the world wraps at `width`, rarely a whole number of chunks, so the last column takes the remainder (wider). North–south there's no wrap.
- Land is judged by the float surface height against `high_water`, as the server will; a spot exactly at the threshold could in theory differ across platforms if the noise isn't bit-identical (the surface golden tests guard that).
