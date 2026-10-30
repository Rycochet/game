# client: surface_style

Colours and thresholds for drawing a planet's surface. Swap the resource for other planet types.

## Files

- `crates/client/src/surface_style/mod.rs`: `SurfaceStylePlugin`, `SurfaceStyle`, `SurfaceKind`, `Band`, `hsl_to_rgba`, test.

## Public API

- `SurfaceStyle` (resource, Earth-like default from the prototype): bands `snow`, `water`, plus `shore_jitter` (0.5), `ice_start` (64), `ice_lightness` (0.933 = `#eee`), `earth_face` (edge face HSL).
- `base_hue_rotation`: degrees added to biome hues, as much as each biome takes (`BiomeColour::rotation`: living colours fully, sand and bare earth little or none). 0 for now; to do: choose per world from the terrain seed, with green most likely.
- `SurfaceKind`: `Snow`, `Land`, `Shore`, `Water`.
- `kind(sample, climate, water_level, snow_line) -> SurfaceKind`: snow if above the snow line, or on dry land colder than `shared::climate::SNOW_TEMPERATURE` (with `jitter` roughening the edge); else land or shore if `height + |jitter|·shore_jitter > water` (shore when actually below water, marking the tide), else water. `band(kind) -> Band`.
- `river_is_frozen(temperature, ground, water_level, snow_line)`: rivers freeze before the land around them is snow, more so the higher up; where one meets the sea, about when the shallows do.
- Open land (`Land`) takes its colour from its climate: `living_colour` blends the biome colours around it on a 10×10 cut of the Whittaker diagram, with `jitter` roughening the joins, and its own lightness instead of the hue-based formula below (`SeasonalForest` is the prototype's grass). High ground also fades to tundra's colours whatever the biome (`TUNDRA_FROM` 76 to `TUNDRA_FULL` 84, purely a look: `shared::climate` still says the biome below). There are no hill or mountain bands: high ground goes through that to snow, so how much there is follows the planet's warmth and the local climate.
- `shade(sample, climate, west_height, water_level, snow_line, slope_strength) -> (Band, lightness)`: **the colour formula** (ported from the prototype). Land `(64 - hue/2 - slopeÂ·(2000 - satÂ·2000)Â·slope_strength)/100` with slope = (height - west)/64; shore the biome colour with its hue turned `SHORE_HUE_TURN` (30) and lightness times `SHORE_LIGHTNESS` (0.8); frozen water `ice_lightness` (flat); other underwater `(height/2 + 15)/100 + ice`, ice = `(ice_start - snow_line)/ice_start` clamped `0..1` (icy blue).
- `is_frozen_water(sample, water_level, snow_line)`: underwater with the snow line below the point, so freezing spreads from the shallows.
- `SurfaceStyle::shown_height(sample, water_level)`: the height to draw a point at: `max(height, water)`, so water is a flat surface and the shore shows at the waterline.
- `Band { hue, saturation }`: lightness is decided by each view.
- `hsl_to_rgba(h, s, l) -> [u8; 4]`: same maths as the prototype's `hsl2rgb`, clamped.

## Depends on

- `shared::surface::SurfaceSample`, `shared::climate`.

## Used by

- `world_map`.

## Notes

- Purely visual, so client-only. Gameplay terrain types (walkable, water, etc.) will be a separate classification in `shared`.
- Snow line comes from `common::Weather`. Falling, it overtakes land and shore; below `ice_start` water lightens to icy blue; once the snow line is below a water point it freezes to flat `#eee` (the prototype went grey here, then white only near 0). Shore points just under the water show as a grey ring mid-cycle, as in the prototype.
- Changes from the prototype: `shore_jitter` 0.5 (was effectively 1.0: about half the width). The prototype's yellow sand is gone (owner): it distracted from the map, so the shore now takes its colour from the biome.
