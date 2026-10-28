# shared: world

A world's global settings, all generated from its seed. Ported from the TypeScript prototype's `IWorld` / `World.tsx` (same ranges; gates not ported yet).

## Files

- `crates/shared/src/world.rs`: `WorldParams`, golden test.

## Public API

- `WorldParams::from_seed(Seed)`: fields `seed`, `radius` (m), `circumference` (m), `year` (days), `density`, `tilt`, `sea_level` (surface height units, `20..60`), `climate`.
- `tide(hour) -> f64`, `water_level(hour) -> f64`: sea level plus tide (two tides a day, uses `libm::cos`). `high_water()`: the highest the tide reaches (`sea_level + 4 × tilt`); ground above it is always dry.

## Depends on

- `shared::rng`, `libm`.

## Used by

- `shared::surface` (circumference), `shared::feature` (`high_water`); client `common` (`CurrentWorld`), `world_map` (water level).

## Notes

- Not ported yet from the prototype: gate positions (needs the old terrain classifier `getTerrain`, which belongs with the roguelike map work), weather noise. World map features are in `shared::feature` instead (one per chunk, not noise).
