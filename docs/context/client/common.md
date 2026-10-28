# client: common

Shared building blocks for the client modules. Keep it small.

## Files

- `crates/client/src/common/mod.rs`: `CommonPlugin`; black `ClearColor`, frame-rate cap (`WinitSettings`), world resource, clock, view state and Tab switching.

## Public API

- `CurrentWorld` (resource): `params: WorldParams`, `surface: Surface`. Fixed seed `12345` until a server supplies one.
- `GameClock` (resource): `hour` `0..24`; 1 real minute = 1 game hour; only changes on the hour (use `is_changed()`).
- `MapView` (state): `Slices` (default) or `Mesh`. Tab toggles. View modules spawn their own camera and entities on `OnEnter` and tag them `DespawnOnExit`.
- `ViewStats` (resource): `summary` line written by the active view, shown by `ui`.
- `Weather` (resource): `snow_line` (`NORMAL_SNOW_LINE` = 128 normally, 0 = frozen). Changed by `demo`; read by both views and `ui`.

## Depends on

- `shared` (`Seed`, `WorldParams`, `Surface`).

## Used by

- All client modules.

## Notes

- There's no shared camera: each view owns its camera, because the slice view is 2D and the mesh view is 3D.
- Frame rate: `WinitSettings` reactive mode, about 30 fps (`FOCUSED_FPS`) while focused and 1 fps unfocused; input wakes it immediately. Not a twitch game, so this saves GPU and power.
- The clock will become server-driven.
