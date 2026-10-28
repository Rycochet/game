# shared: surface

The planet surface height at any integer grid position (1 unit ≈ 1 km). Ported from the prototype's `WorldMap/Surface.ts`.

## Files

- `crates/shared/src/surface.rs`: `Surface`, `SurfaceSample`, golden and wrap tests.

## Public API

- `Surface::new(&WorldParams)`, `.width()` (grid units around the equator; `x` wraps at this), `.sample(x, y) -> SurfaceSample`.
- `SurfaceSample { height, jitter }`: `height` roughly `0.5..127.5` centred on 64; `jitter` `-8..8`, small-scale noise used only to roughen colour-band edges (not physical height).

## Depends on

- `shared::world`, `shared::rng`, `fastnoise-lite` (OpenSimplex2, frequency 1, f64 coordinates), `libm` (cos/sin).

## Used by

- Client `common` (`CurrentWorld`), `slice_view`, `mesh_view`.

## Notes

- Algorithm: point mapped onto a cylinder in noise space (east-west wraps, north-south doesn't), 7 octaves summed (`128 + Σ noise·64/2^i`, halved); jitter is octave 5 × 8.
- Change from the prototype: wraps at an integer `width` (prototype used a fractional circumference, leaving a seam), and uses OpenSimplex2 instead of `simplex-noise`, so exact terrain differs from the old one but has the same character.
- No caching. ~37 ms for a full slice-view redraw natively (debug build); add caching only if profiling shows a need.
- Heights are floats; when movement validation uses them, compare via quantised/integer values or the same thresholds on both sides (see `architecture.md`).
