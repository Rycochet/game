# shared: showcase

A debug area for looking at every kind of poi: from far south of any pole, the world is a flat, bare, dry grassland plain, with normal weather. Behind `ENABLED`, true for now.

## Files

- `crates/shared/src/showcase.rs`: all of it. Tests: every kind has a column of variants, the plain is flat, dry and bare.

## Public API

- `ENABLED`, `VARIANTS` (8), `FLAT_HEIGHT`, `CLIMATE`, `ARRIVAL` (just north of the first poi), `in_plain(y)`, `chunk_in_plain(chunk)`, `poi_at(chunk)`.

## Depends on

- `shared::poi`, `shared::climate`, `shared::rng`.

## Used by

- `shared::surface`, `climate`, `vegetation`, `river` and `poi` (each asks first), `server` (`world_map::to_showcase`, for `DevToShowcase`), client `demo` (P).

## Notes

- One poi in the middle of each chunk: columns east from chunk `x` 0 in `PoiKind::ALL`'s order, so new kinds go on the end by themselves; rows south from chunk `y` 1000. A variant's seed comes from its kind and row, so adding kinds moves none. About 30% of variants come out ruined, as their seeds say.
- Row 1000 is far enough (64,000 points) to be past the poles of any planet by a long way, and no further: drawing is in floats, which lose detail far from the origin.
- All clients and the server agree, as this is in `shared`, but it means any world has the plain, reachable by walking if you went far enough. Turn off `ENABLED` before a real release.
