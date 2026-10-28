# client: surface_style

Colours and thresholds for drawing a planet's surface, shared by all views. Swap the resource for other planet types.

## Files

- `crates/client/src/surface_style/mod.rs`: `SurfaceStylePlugin`, `SurfaceStyle`, `SurfaceKind`, `Band`, `hsl_to_rgba`, test.

## Public API

- `SurfaceStyle` (resource, Earth-like default from the prototype): `mountain_height` (90), `hill_height` (80), bands `mountain`, `hill`, `grass`, `sand` (hue 57, fully saturated), `water`, plus `sand_lightness` (0.5, fixed), `shore_jitter` (0.5), `ice_start` (64), `ice_lightness` (0.933 = `#eee`), `earth_face` (front cut-away HSL).
- `SurfaceKind`: `Mountain`, `Hill`, `Grass`, `Sand`, `Water`.
- `kind(sample, water_level, snow_line) -> SurfaceKind`: mountain if `height+jitter > min(snow_line, 90)`, hill if `> 80`, else land if `height + |jitter|·shore_jitter > water` (sand when actually below water: beaches), else water. `band(kind) -> Band`.
- `shade(sample, west_height, water_level, snow_line, slope_strength) -> (Band, lightness)`: **the one colour formula both views use**. Land `(64 - hue/2 - slope·(2000 - sat·2000)·slope_strength)/100` with slope = (height - west)/64; sand fixed; frozen water `ice_lightness` (flat); other underwater `(height/2 + 15)/100 + ice`, ice = `(ice_start - snow_line)/ice_start` clamped `0..1` (icy blue).
- `is_frozen_water(sample, water_level, snow_line)`: underwater with the snow line below the point, so freezing spreads from the shallows.
- `SurfaceStyle::shown_height(sample, water_level)`: the height to draw a point at: `max(height, water)`, so water is a flat surface and shore sand shows at the waterline.
- `Band { hue, saturation }`: lightness is decided by each view.
- `hsl_to_rgba(h, s, l) -> [u8; 4]`: same maths as the prototype's `hsl2rgb`, clamped.

## Depends on

- `shared::surface::SurfaceSample`.

## Used by

- `slice_view`, `mesh_view`.

## Notes

- Purely visual, so client-only. Gameplay terrain types (walkable, water, etc.) will be a separate classification in `shared`.
- Snow line comes from `common::Weather`. Falling, it overtakes hills, grass and sand (their thresholds don't change); below `ice_start` water lightens to icy blue; once the snow line is below a water point it freezes to flat `#eee` (the prototype went grey here, then white only near 0). Shore points just under the water show as a grey ring mid-cycle, as in the prototype.
- Changes from the prototype: `shore_jitter` 0.5 (was effectively 1.0: about half the beaches) and a brighter fixed sand colour, at the owner's request.
