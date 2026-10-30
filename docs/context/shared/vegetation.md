# shared: vegetation

What grows at any world map position, from the public terrain seed, so the client works it out itself. Only something to look at for now: it affects no rules, so it needs no `WORLD_MAP_VERSION` bump. In `shared` because forests will probably matter to the rules later (the same choice as rivers). Drawn by client `world_map`.

## Files

- `crates/shared/src/vegetation.rs`: `Vegetation`, `Cover`, constants. Golden and wrap tests.

## Public API

- `Vegetation::new(&WorldParams)`, `.cover(x, y, height, ClimateSample) -> Cover`.
- `Cover { trees, growth, narrow, lush }`, each `0..=1`: how wooded, how much low growth, how conifer-like the trees and how rainforest-like the canopy.

## Depends on

- `shared::climate` (`ClimateSample`), `shared::world`, `shared::rng` (through `WorldParams::seed`), `fastnoise-lite`, `libm`.

## Used by

- Client `common` (`CurrentWorld::vegetation`), `world_map` (`HeightCache::cover`).

## Notes

- Smooth functions of temperature and moisture rather than a table by `Biome`, so cover has no hard edges at biome borders. Patch noise (`ZOOM` 45) against how wooded the climate is decides which parts are actually woods, so wetter places have more of their area wooded, with clearings left.
- It ignores water, shore and snow, which change with the tide and weather: the caller leaves those out.
- Trees stop by height (`TREE_LINE_FROM` to `TREE_LINE_TO`), a little below where `surface_style` fades to tundra's colours.
- Planned on top (not built): fields and roads come from the server as draped meshes, since working them out here would reveal features, and they clear cover around them as the client already does round rivers. See `decisions.md`.
