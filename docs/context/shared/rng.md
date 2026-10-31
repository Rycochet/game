# shared: rng

Deterministic randomness. The only source of randomness allowed in shared code.

## Files

- `crates/shared/src/rng.rs`: `Seed`, `Rng`; private `mix` (SplitMix64 finaliser) and `fnv1a` (purpose names to numbers). Golden-value tests.

## Public API

- `Seed::new(u64)`, `.value()`. Serde-serialisable (sent in `protocol::WorldInfo`).
- `Seed::derive("purpose") -> Seed`: independent seed per purpose (e.g. `"land"`, `"radius"`). Always derive rather than reusing a seed for two things.
- `Seed::at(&[i64]) -> u64`: hash of seed + coordinates, for position-based randomness in the infinite world.
- `Seed::rng() -> Rng`: SplitMix64 sequence: `next_u64`, `next_f64` (`0..1`), `range_f64(min, max)`, `range_u64(min, max)` (max excluded).
- `Seed::noise_seed() -> i32`: seed for `fastnoise-lite`.

## Depends on

- `serde` (derive only).

## Used by

- `shared::world` (world settings), `shared::surface` (noise seed), `shared::poi` (per-chunk pois), `shared::site` (site plans and levels).

## Notes

- Changing any constant or algorithm here changes every world: the golden tests will fail on purpose. Only do it as a deliberate, recorded decision.
- No `rand` crate: SplitMix64 is enough for game generation. Add `rand_pcg` only if better statistical quality is needed.
