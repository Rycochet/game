# client: surface_style

Colours and thresholds for drawing a planet's surface. Swap the resource for other planet types.

## Files

- `crates/client/src/surface_style/mod.rs`: `SurfaceStylePlugin`, `SurfaceStyle`, `SurfaceKind`, `Band`, `hsl_to_rgba`, test. Beside it: `band.rs`, `style.rs`, `colour.rs`, `biome.rs`.

## Public API

- `SurfaceStyle` (resource, Earth-like default from the prototype): bands `snow`, `water`, plus `shore_jitter` (0.5), `ice_start` (64), `ice_lightness` (0.933 = `#eee`), `earth_face` (edge face HSL).
- `base_hue_rotation`: degrees added to biome hues, as much as each biome takes (`BiomeColour::rotation`: living colours fully, sand and bare earth little or none). 0 for now; to do: choose per world from the terrain seed, with green most likely.
- `SurfaceKind`: `Snow`, `Land`, `Shore`, `Water`.
- `kind(sample, climate, water_level) -> SurfaceKind`: snow on dry land colder than `shared::climate::SNOW_TEMPERATURE` (with `jitter` roughening the edge); else land or shore if `height + |jitter|·shore_jitter > water` (shore when actually below water, marking the tide), else water. `band(kind) -> Band`.
- `river_is_frozen(temperature, ground, water_level)`: rivers freeze before the land around them is snow, more so the higher up; where one meets the sea, as cold as snow.
- Open land (`Land`) takes its colour from its climate: `living_colour` blends the biome colours around it on a 10×10 cut of the Whittaker diagram, with `jitter` roughening the joins, and its own lightness instead of the hue-based formula below (`SeasonalForest` is the prototype's grass). High ground also fades to tundra's colours whatever the biome (`TUNDRA_FROM` 76 to `TUNDRA_FULL` 84, purely a look: `shared::climate` still says the biome below). There are no hill or mountain bands: high ground goes through that to snow, so how much there is follows the planet's warmth and the local climate.
- `shade(sample, climate, west_height, water_level, floor, slope_strength) -> (Band, lightness)`: **the colour formula** (ported from the prototype). Land `(64 - hue/2 - slope·(2000 - sat·2000)·slope_strength)/100` with slope = (height - west)/64; shore the biome colour with its hue turned `SHORE_HUE_TURN` (30) and lightness times `SHORE_LIGHTNESS` (0.8); frozen water `ice_lightness` (flat); other underwater `(shown/2 + 15)/100`, `shown` the depth between `floor` (`Surface::floor`: 0 for `Desert`, 8 km down for surfaces in metres) and the water squeezed into the water's height, so deep oceans aren't black and `Desert` shades by height as before.
- `ground_colour(sample, climate, water_level, floor) -> [u8; 4]`: `shade` on level ground, as bytes: the land's plain colour, for `dungeon_map`'s outside ground.
- `is_frozen_water(sample, climate, water_level)`: `SurfaceSample::is_frozen`, the same rule walking goes by (sea colder than `climate::SEA_ICE`).
- `SurfaceStyle::shown_height(sample, water_level)`: the height to draw a point at: `max(height, water)`, so water is a flat surface and the shore shows at the waterline.
- `Band { hue, saturation }`: lightness is decided by each view.
- `hsl_to_rgba(h, s, l) -> [u8; 4]`: same maths as the prototype's `hsl2rgb`, clamped.

## Depends on

- `shared::surface::SurfaceSample`, `shared::climate`.

## Used by

- `world_map`, `dungeon_map` (`ground_colour`).

## Notes

- Purely visual, so client-only. Gameplay terrain types (walkable, water, etc.) will be a separate classification in `shared`.
- Snow and ice come only from the climate: the snow line, and Cycle Temperature with it, were removed (2026-11-06). Frozen water is flat `#eee`.
- Changes from the prototype: `shore_jitter` 0.5 (was effectively 1.0: about half the width). The prototype's yellow sand is gone (owner): it distracted from the map, so the shore now takes its colour from the biome.
