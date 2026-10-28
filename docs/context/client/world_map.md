# client: world_map

The world map: the surface as a lit 3D terrain mesh rendered by the GPU. A 400×400 slab of terrain around the player with a flat front face (the prototype's composition, see the Legend entry in `decisions.md`) with modern lighting. Left-click travels to the clicked point; a flat white cylinder marks the player.

## Files

- `crates/client/src/world_map/mod.rs`: `WorldMapPlugin`. `spawn_view` (Startup) spawns a `Camera3d` (tonemapping and debanding off), an east sun with shadows, ambient light, the terrain mesh, the front face and the marker, and inserts `HeightCache`. `follow_player` (runs when `MapPosition`, clock, weather or style changes) moves the terrain to the player's grid point and rebuilds heights/front; `place_camera_and_marker` (every frame) puts the camera and marker at `SmoothPosition`; `pick_on_click` (in `PlayerSystems::Input`) sends `TravelTo`. Constants `HEIGHT_SCALE`, `HALF_SIZE` (200), `CAMERA_FOV_DEGREES` (20), `CAMERA_DISTANCE`, `CAMERA_PITCH_DEGREES`, `LOOK_SOUTH`, `SLOPE_SHADING`, `SUN_LUX`, `AMBIENT_BRIGHTNESS`; `camera_offset()`.
- `crates/client/src/world_map/picking.rs`: `pick(ray, centre, ...) -> Option<(x, y)>`: walks the camera ray in 0.5-unit steps until it is below the drawn surface; misses if it passes over or comes in through the front face. Unit tests.
- `crates/client/src/world_map/marker.rs`: `Marker`, `spawn(...)`: white lit flat cylinder (`DIAMETER` 16, thickness 15%, straight edges; lit so top and side shade differently) with a black outline made from a slightly bigger cylinder, `cull_mode: Front` (inverted hull, no custom shader). Outline width is `OUTLINE_PIXELS` (4) converted to grid units from the camera distance, FOV and window height at spawn.
- `crates/client/src/world_map/terrain.rs`: `HeightCache` (wrapping 512×512 grid of samples, so a step only samples the new edge), `new_terrain_mesh` (fixed 401×401 vertex grid centred on the entity), `update_terrain` (heights, smooth normals from neighbours, vertex colours with baked slope shading, all in place), `build_front` (unlit front face: water depth strip over earth down to `FRONT_BASE` × water level), `highest_shown_height` (max drawn height within a radius, for resting the marker).

## Public API

- `WorldMapPlugin` only.

## Depends on

- `common` (`CurrentWorld`, `GameClock`, `ViewStats`, `Weather`), `player` (`MapPosition`, `SmoothPosition`, `TravelTo`, `PlayerSystems`), `surface_style`.

## Used by

- Nothing.

## Notes

- World `x` → 3D `+X`, world `y` (south) → `+Z`. The terrain and front face sit at the player's grid point (`MapPosition`) and move a whole step at a time; the camera and marker follow `SmoothPosition`, so travel looks smooth (the slab edges shift by under one unit relative to the camera).
- Camera: narrow field of view (20°) from far away (a "long lens"), so perspective is flat and the slab's sides stay near vertical. It looks at **sea level** (not the ground) `LOOK_SOUTH` units south of the player, so it doesn't bob over hills and the front face fits on screen. If the FOV changes, scale `CAMERA_DISTANCE` by `tan(old/2) / tan(new/2)` to keep the framing; `far` is set from the distance.
- Shading: real lighting alone looked too smooth, so the prototype's slope shading (height minus the western neighbour, very strong) is baked into the vertex colours, scaled by `SLOPE_SHADING`. On top, a harsh sun (from the east) with little ambient fill gives shadows and dark far sides.
- Water is part of the terrain mesh, not a separate plane: points below the water are drawn at the surface (`SurfaceStyle::shown_height`) and coloured by depth, as the prototype did. This keeps shore sand (just below the water by definition) visible at the waterline. A translucent plane showed it tinted green, and lifting only the sand made spikes.
- Colours: `Tonemapping::None` on the camera; Bevy's default tonemapping made the palette pastel. With it off, sunlit faces can exceed full brightness and clip towards lime, so keep `SUN_LUX` low.
- Brightness was tuned to roughly match the prototype's renderer (mean luminance ~77, measured from `dev_tools` screenshots with Pillow). If colours or lighting change, re-measure.
- Frozen water vertex colours are multiplied by `ICE_BOOST` (1.41, above white) to cancel the sun's dimming of flat surfaces, so it shows at `#eee`. Re-measure if the lighting changes.
- Front face bottom is `FRONT_BASE` (0.3) × the water level rather than 0 (shorter than the prototype's); where the ground is below that, the water strip stops at the base.
- The marker rests on the highest drawn ground within its radius, so it never sinks into slopes (it floats a little over the low side). If the window is resized, the outline's on-screen width drifts from 4 px (it's computed at spawn).
- Performance (native debug): first fill ~56 ms (161k samples); each grid step ~12 ms (the prototype's CPU renderer took ~38 ms). The vertex loops call `HeightCache::get` several times per vertex; a flat height array per step would be the first optimisation if needed.
- Terrain and front face use `NoFrustumCulling` because their heights change after the bounds are computed. `HeightCache` is inserted by commands at startup, so systems take it as `Option` for the first frame.
