# client: demo

Demo modes toggled by keys, for showing off and testing the views. Works in native and web builds.

## Files

- `crates/client/src/demo/mod.rs`: `DemoPlugin`; `toggle_modes` (keys), `auto_walk`, `temperature_cycle`. Constant `SNOW_LINE_SPEED` (6 units/s, as the prototype).

## Public API

- `AutoWalk(bool)` (resource): while on, the player walks east one step per frame. **G** toggles.
- `TemperatureCycle` (resource): `Off`, `Cooling`, `Warming`. **T** starts a cycle: `Weather::snow_line` falls from `NORMAL_SNOW_LINE` (128) to 0 (everything frozen), then rises back and stops. T during a cycle skips to warming.

## Depends on

- `common` (`Weather`, `NORMAL_SNOW_LINE`), `player` (`MapPosition`, `Player`, `PlayerSystems`: `auto_walk` runs in `Input`).

## Used by

- `ui` (shows both modes), `dev_tools` (`GAME_WALK` sets `AutoWalk`).

## Notes

- Ported from the prototype's commented-out snow animation in `WorldMap.tsx`. What the snow line does to colours lives in `surface_style`.
- During a cycle the weather changes every frame, so the world map rebuilds every frame (~13+ ms).
