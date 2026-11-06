# shared: world

A world's seeds and global settings. Each world has one secret seed (server only); the public terrain seed, the secret pois seed and the public world id are derived from it one way. Ported from the TypeScript prototype's `IWorld` / `World.tsx` (same ranges; gates not ported yet).

## Files

- `crates/shared/src/world.rs`: `WorldSeed`, `WorldId`, `WORLD_MAP_VERSION`, `WorldParams`, golden tests.
- `crates/shared/src/world/time.rs`: `WorldTime`, `ClockRate`.

## Public API

- `WorldSeed::new(u64)` (secret: never send it to a client; serde, for the server's save): `.terrain()` (public terrain seed, for `WorldParams`), `.pois()` (secret, for `PoiMap`), `.id() -> WorldId`, `.value()` (the seed itself, for the server and `shared::hall`).
- `WorldId(Uuid)` (serde): the world's public id, a version 5 UUID of the seed's bytes under a fixed namespace (no clock or randomness); the same seed is the same world on any server. Its own type rather than an `Id`, which only the server makes.
- `WORLD_MAP_VERSION` (6: oases; 5: generators rolled per world): the world map generator's version (see "Generator versions" in `architecture.md`); bump only for changes that matter.
- `WorldTime { offset, rate }` (serde, `Default`): a world's time, game minutes since it began, worked out from the server's clock (real seconds, stopped only by pausing), which every world on a server goes by: `at(minutes, server_seconds, rate)`, `minutes(server_seconds)`, `with_rate(rate, server_seconds)` (carries on from now at another speed, no jump), `set(minutes, server_seconds)` (not before 0). `ClockRate { game, real }` (whole numbers: `game` minutes for every `real`; `NORMAL` 1 to 1, the default; `new` refuses either being 0; `factor()`). Sent to clients so they can work a world's time out from the server's clock, kept in step by pings.
- `GAME_MINUTE_SECONDS` (1), `MINUTES_PER_HOUR` (60), `GAME_HOUR_SECONDS` (60): real time against the game's clock, which counts minutes.
- `WorldParams::from_seed(Seed)` (given the terrain seed; rolls its generator, or takes `HOW_DEBUG_GENERATOR`'s), `WorldParams::with_generator(Seed, SurfaceGenerator)` (what the seed would be with that generator, for View Planet): fields `seed` (the terrain seed), `radius` (m, `SMALLEST_RADIUS` 1,500 km to 6,500 km since world map version 4, so every planet has room for its World Gates), `circumference` (m), `year` (days), `density`, `tilt`, `sea_level` (surface height units, `20..60`; a desert world's is `DESERT_SEA_BELOW` (0.5) under the lowest of its `Surface::lowest_sampled` ground, so only the deepest hollows hold water), `climate` (wetness; desert `0..0.1`), `warmth` (`-0.1..0.1`, desert `0.25..0.35`, so even its hills are mostly hot desert: how warm the planet is on average, used by `shared::climate`), `generator` (`SurfaceGenerator`, rolled from the seed: `Desert` 1%, `Continents` 70%, `Plates` the rest, owner).
- `tide(&Celestial, hour) -> f64`, `water_level(&Celestial, hour) -> f64`: sea level plus the moons' tide (`Celestial::tide_pull` × `tide_reach()`, `TIDE_REACH` (0.1, 10 m, owner: realistic) × `tilt`; always `sea_level` with no moons). `high_water()`: the highest the tide reaches (`sea_level + tide_reach()`); ground above it is always dry.

## Depends on

- `shared::rng`, `shared::celestial` (the tide), `shared::surface` (`SurfaceGenerator`; a desert's lowest ground), `libm`, `serde`, `uuid`.

## Used by

- `shared::hall` (`WorldSeed`), `shared::surface` (circumference), `shared::poi` (`high_water`), `shared::protocol` (`WorldId`); `server` (`WorldSeed`, `WorldTime`, water level); client `common` (`CurrentWorld`), `world_map` (water level), `connection` (`WORLD_MAP_VERSION`).

## Notes

- Not ported yet from the prototype: gate positions (needs the old terrain classifier `getTerrain`, which belongs with the roguelike map work). Weather is in `shared::weather`, suns and moons in `shared::celestial`. World map pois are in `shared::poi` instead (one per chunk, not noise).
