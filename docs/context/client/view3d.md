# client: view3d

What every 3D view shares: the one camera (orbiting the player: turn, tilt, zoom) and the entity tokens. The views (`world_map`, `dungeon_map`) draw their own scenes and tell this module their grid layout through `ViewGrid`.

## Files

- `crates/client/src/view3d/mod.rs`: `View3dPlugin`. `spawn_camera` (Startup: `Camera3d` with `ViewCamera` and `OrbitCamera`, tonemapping and debanding off; inserts `TokenAssets`). In `PlayerSystems::Input`: `camera::control_with_mouse`, `camera::control_with_touch`; in `View3dSystems::Click` (inside `Input`): `token::click_tokens`. After `PlayerSystems::Travel`, in `View3dSystems::Place`: `token::resize_tokens`, `sync_tokens`, `sync_icons`, then `place_camera_and_tokens` (camera looks at the player's `SmoothPosition` × cell size at floor height, writes `common::ViewFacing`; token views stand on the floor at their owners' `SmoothPosition`; icon anchors face the camera). Constants `CAMERA_FOV_DEGREES` (20), `CAMERA_DISTANCE` (1100) and `CAMERA_PITCH_DEGREES` (34) for the starting view, `LOOK_AHEAD`.
- `crates/client/src/view3d/camera.rs`: `OrbitCamera` (component: pub `yaw`, `pitch`, `distance`; `new(pitch, distance, look_ahead)`, `reset()`, `transform(target)`, `ground_axes()` (right and towards-camera along the ground), `quarter_turns()`, `zoom()` (distance / starting distance), `home_pitch()`), `control_with_mouse` (right-drag turns and tilts, `RADIANS_PER_PIXEL` 0.005; pitch 10°–85°; wheel zooms ×0.9 per notch, between the starting distance (the furthest out) and `MIN_ZOOM` 0.6 of it, both the owner's choice; Home resets all three), `control_with_touch` (from `common::TouchGesture`: a pinch zooms, a twist turns with the fingers, both fingers up or down tilt as right-dragging does, within the same limits). Unit tests.
- `crates/client/src/view3d/token.rs`: how `Token`s are drawn. `TokenAssets` (resource: one body and one outline mesh per shape, the icon quad, one lit body material per attitude and one outline material, each in a normal and a completed version; `icon_offset(camera_rotation, pitch, home_pitch)`), `prism` (`DIAMETER` 16, thickness 15%), `TokenView` (separate root entity per token: body, outline and `IconAnchor` children), `sync_tokens`, `sync_icons` (unlit, alpha-masked icon quads, nearest sampling, one material per icon file), `resize_tokens` (outline meshes rebuilt in place when the window height or zoom changes), `HoveredToken`, `ClickedToken` and `find_token_under_cursor` (every frame, the nearest visible `Clickable` token whose base circle or icon picture is under the cursor, measured on screen; `ClickedToken` if the left button was just pressed). Outline: inverted hull (`cull_mode: Front`) grown by `OUTLINE_PIXELS` (2) in screen pixels.

## Public API

- `View3dPlugin`, `View3dSystems::Click` (systems handling clicks run after it) and `::Place` (views that adjust token heights run after it).
- `ClickedToken(Option<Entity>)` (resource): the `common::Clickable` entity whose token the left button was just pressed on, this frame only. Views skip their ground click when it is set; `site` uses it.
- `HoveredToken(Option<Entity>)` (resource): the `common::Clickable` entity the mouse is over, every frame; `world_map` outlines it and `minimap` names it.
- `ViewCamera` (marker on the camera), `OrbitCamera` (read or set `yaw` (0 looks north, positive moves the camera east), `pitch`, `distance`; used by `dev_tools` `GAME_VIEW`, `world_map` and `dungeon_map`).
- `ViewGrid` (resource, set by the current view on entering): `cell_size` (3D units per grid step) and `floor` (height the camera looks at and tokens stand on).
- `TokenView` (`owner`), `TOKEN_DIAMETER`, `TOKEN_HALF_THICKNESS`: for views that place tokens themselves (the world map rests them on the terrain) or size things to them.

## Depends on

- `common` (`ViewFacing`, `Token`, `Icon`, `SmoothPosition`, `Clickable`), `player` (`Player`, `PlayerSystems`).

## Used by

- `world_map`, `dungeon_map`, `site`, `dev_tools`.

## Notes

- Camera: narrow field of view (20°) from far away (a "long lens"), so perspective is flat. It looks at the floor (sea level on the world map, so it doesn't bob over hills), ahead of the player towards the camera by `LOOK_AHEAD` in the starting view (shrinking as it zooms in or looks down). If the FOV changes, scale `CAMERA_DISTANCE` by `tan(old/2) / tan(new/2)` to keep the framing; `far` is twice the starting distance.
- `ViewFacing` (common) is the camera's yaw rounded to a quarter turn; `player` uses it so arrow keys step towards the top of the screen. Written with `set_if_neq`.
- Token sizes are in 3D units, the same in every view: on the world map a token covers 16 grid points; in a dungeon cell (24 units) it fits inside.
- Token views never rotate (shape turns are baked into the meshes), so an `IconAnchor`'s rotation copied from the camera makes it face the screen. Icons are sized relative to the token, not the screen (owner's choice, for a blockier retro look): the central 32×32 picture is `ICON_ART_SIZE` (1.5 × the diameter) wide at `TokenSize::Default`, scaled by the owner's `TokenSize` (the anchor's transform scale, kept in `IconAnchor::scale`, set by `sync_tokens`; `icon_offset` takes it too). Token bases are the same size whatever the icon size. In the starting view the feet (bottom of the central picture less `ICON_SHADOW_PIXELS`: 3 art pixels) stand `ICON_GAP_PIXELS` (1 art pixel) above the top's centre; tilting down slides them `ICON_SLIDE` (0.5) of the way to `FEET_ABOVE_EDGE_PIXELS` (2 screen pixels) above the near edge, The anchor is also moved towards the camera (which barely moves it on screen) by the token's radius (with outline) × cos(pitch), plus `HALF_THICKNESS`, so the whole quad is in front of the token: standing over the centre, its lower part used to be inside the token and hidden by the near half, cutting pictures off around the middle of the base (most visible at flat angles and on big pictures). The art (Gervais tiles) is planned to be replaced: retune `ICON_FILE_PIXELS`, `ICON_ART_PIXELS`, `ICON_ART_SIZE`, `ICON_GAP_PIXELS` and `ICON_SHADOW_PIXELS` to match it. Polygons have a corner pointing north, then turn: the square to line up with the grid, the hexagon (30°) and octagon (22.5°) to put a flat side towards the camera in the starting view.
- Completed tokens (`fade`): sRGB colour × `COMPLETED_SHADE` (0.7), alpha `COMPLETED_OPACITY` (0.9, owner's choice) with `AlphaMode::AlphaToCoverage` (needs MSAA): ordinary blending showed the outline's inner faces through the body. Coverage comes in coarse steps, so small opacity changes may do nothing.
- Tokens are lit by real Bevy lights: the world map's sun, the dungeon's point lights. Every token is drawn in every view; views hide the ones that don't belong (the world map hides tokens outside its window via the `TokenView`'s `Visibility`, and despawns its pois when left). The dungeon view hides tokens on cells nobody sees.
- Bevy UI draws on this camera (the only one).
