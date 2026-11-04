# shared: world_gate

World Gates: the ways onto a planet, `WORLD_GATES` (16) on every one, each on land with room to explore round it and well apart from the others. Worked out from the public terrain seed, for any surface generator, so client and server agree. Only where they are: what a gate looks like or does is elsewhere (later).

## Files

- `crates/shared/src/world_gate.rs`: `WorldGates` (`find`, `complete`), `WorldGate`, `WORLD_GATES`, the rounds and spacing. Tests (same seed same gates; on dry land, apart, away from the poles; a planet of water has none and gives up; 120 seeds in a row with every generator find all 16, printing the rounds they needed).
- `world_gate/land.rs`: `CoarseLand`: 32-point cells by the broad height at their middle, land only if most of the 8 neighbours are dry too; worked out as asked for and kept.
- `world_gate/reach.rs`: `has_room`: flood fill over coarse land, stopping once it has found enough.
- `world_gate/spot.rs`: `dry_spot`: the exact spot by the full surface, square rings outward to dry land.

## Public API

- `WORLD_GATES` (16, owner).
- `WorldGates::find(&WorldParams, &Surface)`: `gates` (in the order found) and `tries`; `.complete()`: all 16 placed.
- `WorldGate { x, y, chunk, round }`: on dry land above high tide, `x` in `0..width`; `chunk` as `PoiMap::chunk_at` has it (no other poi stands there); `round` from 0.

## Depends on

- `shared::surface`, `shared::world` (`high_water`, the seed), `shared::poi` (`CHUNK_SIZE`, `ChunkPos`), `shared::rng`, `libm`.

## Used by

- `shared::poi` (`PoiMap` finds the world's gates with its surface and keeps their chunks free of other pois; `world_gates()`), server (`world_map::arrival`: new characters arrive at one; `back_to_start` to the nearest), client `planet_view` (rings on the globe, the bar's line, `client.planet`).

## Notes

- Every seed is a planet (owner: seeds are simple maths, not chosen), so nearly every planet must find all 16. Candidates are spots spread evenly over the area within 60° of the equator, from `seed.derive("world gates")`, 512 new ones a round. Each needs land joined to it (the fill may reach 70°) and spacing from the gates already found. Only gates still missing are looked for in the next round, which asks for less: 2, 1.5, 1, then 0.25 screens of land (a screen is about 400 × 400 grid points), and spacing 1, 0.7, 0.4, then 0.15 of the planet's, which is the smaller of 1,000 points or a twelfth of its width (owner: size-based). A planet of water tries every round and gives up (2,048 tries).
- Measured on 1,000 seeds in a row each (planets are at least `world::SMALLEST_RADIUS`, 1,500 km, raised for this): all 3,000 found all 16. Classic 998 in round 1; Continents 662, 166, 139, 33 by round; Plates 690, 209, 92, 9.
- Land is judged as the walking rules and pois do: above `high_water`, broad height for the coarse cells, the full surface for the spot.
- No floats beyond the surface's own; candidates and fills go in a fixed order (the fill's queue), so the result is the same everywhere.
