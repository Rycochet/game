# client: dungeon_map

The dungeon (and building) view: `shared::dungeon` cells drawn in 3D through `view3d`'s camera, with the same tokens as the world map. Shown while `GameView::Dungeon` is current (the starting view for now). Draws only what the player, or anything with `SharesSight`, can see; lights it from `LightSource`s. Left-click a seen floor to travel there.

## Files

- `crates/client/src/dungeon_map/mod.rs`: `DungeonMapPlugin`. `spawn_scene` (Startup: a `DungeonScene` root with a solid mesh and a see-through mesh, unlit, sharing a generated tile texture (white with a 1-pixel black border on 32×32, nearest sampling)). `pick_on_click` (in `PlayerSystems::Input`: sends `TravelTo` for a seen floor cell, using the private `Shown` resource: last combined sight and solid wall sides). `show_scene` / `hide_scene` on entering/leaving the view (also sets `ViewGrid` to `CELL_SIZE` with the floor at 0, and the dim ambient light for tokens). `update_meshes` (when a viewer or light moves, a `LightSource` or the level changes, or the camera's position changes (turning, zooming, following the player): one `Sight` per viewer plus the combined one, light, rebuild, store `Shown`; writes `ViewStats`). `sync_lights` (one Bevy `PointLight` per `LightSource`, to light tokens). Constants `CELL_SIZE` (1.5 × token diameter = 24, about the token's picture), `WALL_HEIGHT`, `FLOOR_COLOUR` (#404040), `WALL_COLOUR` (#8b5a2b), `AMBIENT_LIGHT` (0.2), `SEE_THROUGH_ALPHA` (0.5), token light constants.
- `crates/client/src/dungeon_map/mesh.rs`: `LightGrid` (light per cell: `add_light` uses `shared::dungeon::Sight` with the radius, so walls block it; strength `1 − (d / (radius + 1))²`; corner light = average over the open cells round a grid corner). `View` (a viewer's cell and its own `Sight`), `WallFace`. `build` (the two meshes: a floor quad per seen open cell; for each wall only its `visible_sides`, each solid or see-through by `faces_camera`; returns the solid sides for picking). `visible_sides` (a viewer sees a side when it sees the wall and the open cell in front of it, and stands on that side: at most two per viewer, never the top). `faces_camera(centre, normal, camera)` (the camera's position is in front of the side's plane). Unit tests.
- `crates/client/src/dungeon_map/picking.rs`: `pick(ray, level, sight, solid_faces)`: the ray to the floor (height 0) gives a cell, if it is open and seen and no solid wall side is hit first (see-through sides don't block). Unit tests.

## Public API

- `DungeonMapPlugin`, `CELL_SIZE`.

## Depends on

- `common` (`CurrentDungeon`, `GameView`, `LightSource`, `SharesSight`, `ViewStats`, `MapPosition`, `SmoothPosition`), `player` (`Player`, `PlayerSystems`, `TravelTo`), `view3d` (`ViewCamera`, `ViewGrid`, `View3dSystems`, `TOKEN_DIAMETER`), `shared::dungeon`.

## Used by

- Nothing.

## Notes

- Grid cell `(x, y)` is centred on `(x × CELL_SIZE, 0, y × CELL_SIZE)`; walls are `WALL_HEIGHT` tall, but only their seen sides are drawn.
- Wall rule (owner's): only sides someone sees are drawn (tops and bottoms never). A seen side facing the camera is solid; one facing away (the wall stands between the camera and what is seen) is drawn 50% see-through, from behind (the see-through material doesn't cull back faces). Facing is measured from the camera's position, not its direction, so as the camera turns a side flips only when the camera passes its plane; this is the same test as back-face culling, so solid sides are always front faces. Out of sight: not drawn at all (no remembered map yet).
- Light is worked out on the grid and baked into vertex colours (unlit materials): `colour × (ambient + (1 − ambient) × light)`, so full light shows the tile colour as authored. Floors and wall sides take corner-averaged light (smooth across floors, no leaking out of walls). Light colours add up (clamped), so coloured lights work already; all are white for now.
- Tokens can't use baked light, so they get real lights: a `PointLight` (no shadows) `TOKEN_LIGHT_HEIGHT` above each light source, range (radius + 1) cells, plus a dim ambient. Tuned by eye.
- Sight and light use `MapPosition` (whole cells).
- The see-through mesh is one mesh with `AlphaMode::Blend`, so its faces don't sort against each other; with one colour at 50% this hasn't shown.
- Clicks: travel uses the world map's straight-line rules (through walls) until pathfinding (planned as a separate task); `player` travels at 12 cells/s here.
- Rebuilding everything is ~0.1 ms for the test room; it happens every frame while the camera moves (dragging, travelling). Bigger levels may need to rebuild only what changed.
