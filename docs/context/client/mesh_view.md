# client: mesh_view

The surface as a lit 3D terrain mesh rendered by the GPU (Tab switches from `slice_view`). Same composition as the slice view: a 400×400 slab of terrain around the player with a flat front face, but with modern lighting (see the Legend entry in `decisions.md`).

## Files

- `crates/client/src/mesh_view/mod.rs`: `MeshViewPlugin`; on entering `MapView::Mesh` spawns a `Camera3d` (tonemapping off), an east sun with shadows, ambient light, the terrain mesh and the front face, and inserts `HeightCache`. `follow_player` (runs when the player, clock or style changes) moves everything to the player and rebuilds heights/front. Constants `HEIGHT_SCALE`, `HALF_SIZE` (200), `CAMERA_FOV_DEGREES` (20), `CAMERA_DISTANCE`, `CAMERA_PITCH_DEGREES`, `LOOK_SOUTH`, `SLOPE_SHADING`, `SUN_LUX`, `AMBIENT_BRIGHTNESS`; `camera_offset()`.
- `crates/client/src/mesh_view/terrain.rs`: `HeightCache` (wrapping 512×512 grid of samples, so a step only samples the new edge), `new_terrain_mesh` (fixed 401×401 vertex grid centred on the entity), `update_terrain` (heights, smooth normals from neighbours, vertex colours with baked slope shading, all in place), `build_front` (unlit front face: water depth strip over earth down to `FRONT_BASE` × water level).

## Public API

- `MeshViewPlugin` only.

## Depends on

- `common` (`CurrentWorld`, `GameClock`, `MapView`, `ViewStats`), `player` (`MapPosition`), `surface_style`.

## Used by

- Nothing.

## Notes

- World `x` → 3D `+X`, world `y` (south) → `+Z`. The terrain and front face sit at the player's position and move one step at a time with no gliding, like the slice view.
- Camera: narrow field of view (20°) from far away (a "long lens"), so perspective is flat and the slab's sides stay near vertical. It looks at **sea level** (not the ground) `LOOK_SOUTH` units south of the player, so it doesn't bob over hills and the front face fits on screen. If the FOV changes, scale `CAMERA_DISTANCE` by `tan(old/2) / tan(new/2)` to keep the framing; `far` is set from the distance.
- Shading: real lighting alone looked too smooth, so the slice view's slope shading (height minus the western neighbour, very strong) is baked into the vertex colours, scaled by `SLOPE_SHADING`. On top, a harsh sun (from the east) with little ambient fill gives shadows and dark far sides.
- Water is part of the terrain mesh, not a separate plane: points below the water are drawn at the surface (`SurfaceStyle::shown_height`) and coloured by depth, as in the slice view. This keeps shore sand (just below the water by definition) visible at the waterline. A translucent plane showed it tinted green, and lifting only the sand made spikes.
- Colours: `Tonemapping::None` on the camera; Bevy's default tonemapping made the palette pastel. With it off, sunlit faces can exceed full brightness and clip towards lime, so keep `SUN_LUX` low.
- Brightness is tuned to roughly match the slice view (mean luminance ~77 vs ~77.5, measured from `dev_tools` screenshots with Pillow). If colours or lighting change, re-measure.
- Performance (native debug): first fill ~56 ms (161k samples); each step ~12 ms (vs ~38 ms for the slice view). The vertex loops call `HeightCache::get` several times per vertex; a flat height array per step would be the first optimisation if needed.
- Terrain and front face use `NoFrustumCulling` because their heights change after the bounds are computed.
- The view must not need the player in its `OnEnter` setup: when the game starts in this view, setup runs before `Startup` spawns the player. `HeightCache` is added by commands, so `follow_player` takes it as `Option` for the first frame.
- Rebuilds when the player, hour, `Weather` or style changes. Colours come from `SurfaceStyle::shade` (shared with the slice view) with `SLOPE_SHADING`.
- Frozen water vertex colours are multiplied by `ICE_BOOST` (1.41, above white) to cancel the sun's dimming of flat surfaces, so it shows at `#eee` like the slice view. Re-measure if the lighting changes.
- Front face bottom is `FRONT_BASE` (0.3) × the water level rather than 0, making it ~30% shorter than the slice view's at its lowest point; where the ground is below that, the water strip stops at the base.
