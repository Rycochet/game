# shared: world

A world's seeds and global settings. Each world has one secret seed (server only); the public terrain seed, the secret pois seed and the public world id are derived from it one way. Ported from the TypeScript prototype's `IWorld` / `World.tsx` (same ranges; gates not ported yet).

## Files

- `crates/shared/src/world.rs`: `WorldSeed`, `WorldId`, `WORLD_MAP_VERSION`, `WorldParams`, golden tests.

## Public API

- `WorldSeed::new(u64)` (secret: never send it to a client; serde, for the server's save): `.terrain()` (public terrain seed, for `WorldParams`), `.pois()` (secret, for `PoiMap`), `.id() -> WorldId`.
- `WorldId(u64)` (serde): the world's public id; the same seed is the same world on any server.
- `WORLD_MAP_VERSION` (1): the world map generator's version (see "Generator versions" in `architecture.md`); bump only for changes that matter.
- `GAME_HOUR_SECONDS` (60): real seconds in a game hour.
- `WorldParams::from_seed(Seed)` (given the terrain seed): fields `seed` (the terrain seed), `radius` (m), `circumference` (m), `year` (days), `density`, `tilt`, `sea_level` (surface height units, `20..60`), `climate`, `warmth` (`-0.1..0.1`: how warm the planet is on average, used by `shared::climate`).
- `tide(hour) -> f64`, `water_level(hour) -> f64`: sea level plus tide (two tides a day, uses `libm::cos`). `high_water()`: the highest the tide reaches (`sea_level + 4 × tilt`); ground above it is always dry.

## Depends on

- `shared::rng`, `libm`, `serde`.

## Used by

- `shared::surface` (circumference), `shared::poi` (`high_water`), `shared::protocol` (`WorldId`); `server` (`WorldSeed`); client `common` (`CurrentWorld`), `world_map` (water level), `connection` (`WORLD_MAP_VERSION`).

## Notes

- Not ported yet from the prototype: gate positions (needs the old terrain classifier `getTerrain`, which belongs with the roguelike map work). Weather is in `shared::weather`. World map pois are in `shared::poi` instead (one per chunk, not noise).
