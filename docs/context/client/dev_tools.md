# client: dev_tools

Developer helpers driven by environment variables. Compiled for native only (`#[cfg(not(target_arch = "wasm32"))]` in `main.rs`) and inert unless a variable is set.

## Files

- `crates/client/src/dev_tools/mod.rs`: `DevToolsPlugin`.

## Public API

- `GAME_VIEW=mesh|slices`: starting view.
- `GAME_POS=x,y`: starting grid position.
- `GAME_WALK=1`: start with `demo::AutoWalk` on (one step east per frame); with a screenshot, the HUD shows the per-step update cost.
- `GAME_SNOW=n`: start with `Weather::snow_line` at `n` (128 normal, 0 frozen), for screenshots of the temperature states.
- `GAME_SCREENSHOT=path.png` (+ optional `GAME_SCREENSHOT_DELAY` seconds, default 5): save a screenshot, then quit 2 s later.

## Depends on

- `common` (`MapView`, `Weather`), `demo` (`AutoWalk`), `player` (`MapPosition`, `Player`).

## Used by

- Nothing (only through environment variables).

## Notes

- How Claude checks visual changes natively, e.g. from Git Bash:
  `GAME_VIEW=mesh GAME_SCREENSHOT=C:/Projects/game/target/shots/mesh.png ./target/debug/game.exe`
  then read the PNG. Keep screenshots under `target/` (git-ignored).
- Python with Pillow is installed on the owner's machine for analysing screenshots (e.g. mean brightness, cropping the HUD).
- To find which Bevy system is failing in a panic, build with `cargo build -p client --features bevy/debug --target-dir target/debug-names` (system names are hidden otherwise).
