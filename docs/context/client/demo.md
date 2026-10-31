# client: demo

Demo modes toggled by keys, for showing off and testing the views. Works in native and web builds.

## Files

- `crates/client/src/demo/mod.rs`: `DemoPlugin`; `toggle_modes` (keys), `auto_walk`, `temperature_cycle`, `back_to_start` (**Esc**), `cycle_token` (**K** next shape, **C** next attitude, **X** toggle completed, **Z** next icon size, for the player's `Token`), `cycle_stairs` (**B**: every staircase on the level drawn as the next `StairsStyle`, until the level changes), `cycle_fields` (**F**: every field on the level drawn as its next stage, pasture as crops too so all five show, until the level changes), (The M view switch is gone: the dungeon view is reached by entering sites.) Constant `SNOW_LINE_SPEED` (6 units/s, as the prototype).

## Public API

- **Esc**: asks the server to put the player back where new characters start, out of any site (`DevBackToStart`; only the embedded server allows it).
- `AutoWalk(bool)` (resource): while on, the player walks east one step per frame (a `player::TakeStep`; world map only). **G** toggles.
- `TemperatureCycle` (resource): `Off`, `Cooling`, `Warming`. **T** starts a cycle, asking the server (`DevSnowLine`, dev tools) to move it a whole height at a time: `Weather::snow_line` falls from `NORMAL_SNOW_LINE` (128) to 0 (everything frozen), then rises back and stops. T during a cycle skips to warming.

## Depends on

- `common` (`Weather`, `NORMAL_SNOW_LINE`, `Token`, `GameView`, `CurrentDungeon`), `connection` (`ToServer`), `player` (`Player`, `PlayerSystems`, `TakeStep`: `auto_walk` runs in `Input`), `shared::movement`.

## Used by

- `ui` (shows both modes), `dev_tools` (`GAME_WALK` sets `AutoWalk`).

## Notes

- Ported from the prototype's commented-out snow animation in `WorldMap.tsx`. What the snow line does to colours lives in `surface_style`.
- During a cycle the weather changes every frame, so the world map rebuilds every frame (~13+ ms).
