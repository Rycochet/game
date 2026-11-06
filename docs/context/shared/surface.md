# shared: surface

The planet surface height at any integer grid position (1 unit ≈ 1 km). Ported from the prototype's `WorldMap/Surface.ts`.

## Files

- `crates/shared/src/surface.rs`: `Surface` (picks a generator, the showcase plain, metres), `SurfaceSample`, `SurfaceGenerator`, `PlateSample`, `METRES_PER_UNIT`, the shared noise set-up and cylinder mapping. Golden and wrap tests; `continents_tests.rs` (land share, how much is high, shelves and depths), `plates_tests.rs` (wrapping, land share, ranges, plates only where they should be).
- `surface/desert.rs`: `Desert`, the original generator (7 layers from 256 grid units).
- `surface/continents.rs`: `Continents`, worked out in whole metres, with plates for `Plates`; in `continents/`: `layers.rs` (layered, ridged and warped noise), `shape.rs` (how far into a continent, to metres), `plates.rs` (`Plates`, `Tectonics`: a point's plate and the next, the boundary between them wandering by noise), `plates/cells.rs` (`Cells`, `Plate`: one plate per cell of a grid 10 across the planet, centre, kind (40 % continental) and drift from the seed), `plates/boundary.rs` (`Boundary`: ranges, coastal ranges, trenches, island arcs, ocean ridges and rifts by how plates meet).

## Public API

- `Surface::new(&WorldParams)` (the world's own, by `WorldParams::generator`), `Surface::with_generator(&WorldParams, SurfaceGenerator)`, `.generator()`, `.width()` (grid units around the equator; `x` wraps at this), `.sample(x, y) -> SurfaceSample`, `.broad_height(x, y)` (only the 3 largest layers: hills and mountains without small bumps, what walking counts as climbing).
- `SurfaceGenerator` (`Desert`, the old layered noise, the default; `Continents`, `Plates`; `ALL`, `.name()`, `named(name)`, `in_metres()`, `forced()`: `HOW_DEBUG_GENERATOR`, read here so a native client and its own server agree). Every world uses `Desert` until the generator is rolled per world (planned).
- `Surface::floor()`: the lowest height water is shaded down to: 0 for `Desert`, 8 km below the sea for surfaces in metres.
- `Surface::plate_at(x, y) -> Option<PlateSample>` (`plate`, `continental`, `edge` in plate cells, `closing` `-2..2`): `Plates` only, for showing them.
- `Surface::metres(height) -> i32`: a height on the scale as whole metres from the world's `sea_level`, `METRES_PER_UNIT` (100) to a unit. Generators in metres are turned into the scale the same way, so everything else reads them unchanged (peaks may pass 127.5).
- `SurfaceSample { height, jitter }`: `height` roughly `0.5..127.5` centred on 64; `jitter` `-8..8`, small-scale noise used only to roughen colour-band edges (not physical height). `.is_frozen(water_level, climate)`: whether it is sea ice, under water where the climate over it (`Climate::sample` at its height) is colder than `climate::SEA_ICE`, `jitter` roughening the edge by up to 2 °C (used for both drawing and walking).

## Depends on

- `shared::world`, `shared::rng`, `shared::climate` (`ClimateSample`, `SEA_ICE`, for `is_frozen`), `fastnoise-lite` (OpenSimplex2, frequency 1, f64 coordinates), `libm` (cos/sin).

## Used by

- `shared::movement` (`WorldGround`), client `common` (`CurrentWorld`), `world_map`, `surface_style`.

## Notes

- `Continents` (2026-11-04): noise sized to the planet (largest layer a quarter of its width; layers down to 2 grid units, or 32 for `broad_height`), its point domain-warped so coasts wander. The coast is set at the value leaving the world's land share (`0.2..0.45` from the seed) above it, found from 64 x 48 points spread evenly over the globe's area when it is made. Shape: shelves to -200 m, a slope to -4 km and floors to -8 km; land rising slowly from the coast, raised more or less by plateau noise (up to 1.5 km); mountains (up to 9 km) only in rare long belts where other noise crosses 0, with ridged noise in them. On land about a median 300 m, a tenth above 1 km and 2 % above 3 km, roughly as on Earth.
- `Plates` (2026-11-04): `Continents` with plates. A plate's kind moves the land noise (`PLATE_LAND`), so continents mostly sit on continental plates, blending across a quarter of a cell at their edges; mountains come only from boundaries (a solid range with ridged noise along it), not the noise belts. Plates closing faster than 0.2 (full by 1) make: continent against continent, a wide range on both sides; continent against ocean, a coastal range on the continent and a trench off it; ocean against ocean, a trench on the plate with the lower cell and an island arc on the other. Parting: an ocean ridge (+2.5 km) between ocean plates, a rift (-800 m) otherwise. Ranges also raise the land (`RANGE_LAND`), so they and arcs stand out of the sea. Constants were tuned by eye on seeds 1 and 3.

- Algorithm: point mapped onto a cylinder in noise space (east-west wraps, north-south doesn't), 7 octaves summed (`128 + Σ noise·64/2^i`, halved); jitter is octave 5 × 8.
- Change from the prototype: wraps at an integer `width` (prototype used a fractional circumference, leaving a seam), and uses OpenSimplex2 instead of `simplex-noise`, so exact terrain differs from the old one but has the same character.
- No caching here; the client's `world_map` keeps its own `HeightCache`. Add caching in shared code only if profiling shows a need.
- Heights are floats; when movement validation uses them, compare via quantised/integer values or the same thresholds on both sides (see `architecture.md`).
