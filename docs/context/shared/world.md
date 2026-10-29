# shared: world

A world's seeds and global settings. Each world has one secret seed (server only); the public terrain seed, the secret features seed and the public world id are derived from it one way. Ported from the TypeScript prototype's `IWorld` / `World.tsx` (same ranges; gates not ported yet).

## Files

- `crates/shared/src/world.rs`: `WorldSeed`, `WorldId`, `WORLD_MAP_VERSION`, `WorldParams`, golden tests.

## Public API

- `WorldSeed::new(u64)` (secret: never send it to a client): `.terrain()` (public terrain seed, for `WorldParams`), `.features()` (secret, for `FeatureMap`), `.id() -> WorldId`.
- `WorldId(u64)` (serde): the world's public id; the same seed is the same world on any server.
- `WORLD_MAP_VERSION` (1): the world map generator's version (see "Generator versions" in `architecture.md`); bump only for changes that matter.
- `WorldParams::from_seed(Seed)` (given the terrain seed): fields `seed` (the terrain seed), `radius` (m), `circumference` (m), `year` (days), `density`, `tilt`, `sea_level` (surface height units, `20..60`), `climate`.
- `tide(hour) -> f64`, `water_level(hour) -> f64`: sea level plus tide (two tides a day, uses `libm::cos`). `high_water()`: the highest the tide reaches (`sea_level + 4 × tilt`); ground above it is always dry.

## Depends on

- `shared::rng`, `libm`, `serde`.

## Used by

- `shared::surface` (circumference), `shared::feature` (`high_water`), `shared::protocol` (`WorldId`); `server` (`WorldSeed`); client `common` (`CurrentWorld`), `world_map` (water level), `connection` (`WORLD_MAP_VERSION`).

## Notes

- Not ported yet from the prototype: gate positions (needs the old terrain classifier `getTerrain`, which belongs with the roguelike map work), weather noise. World map features are in `shared::feature` instead (one per chunk, not noise).
