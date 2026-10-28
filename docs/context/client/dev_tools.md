# client: dev_tools

Developer helpers driven by environment variables. Compiled for native only (`#[cfg(not(target_arch = "wasm32"))]` in `main.rs`) and inert unless a variable is set.

## Files

- `crates/client/src/dev_tools/mod.rs`: `DevToolsPlugin`.

## Public API

- `GAME_POS=x,y`: starting grid position.
- `GAME_TRAVEL=x,y`: start travelling there (as if clicked); combine with `GAME_SCREENSHOT_DELAY` to catch it mid-journey.
- `GAME_WALK=1`: start with `demo::AutoWalk` on (one step east per frame); with a screenshot, the HUD shows the per-step update cost.
- `GAME_SNOW=n`: start with `Weather::snow_line` at `n` (128 normal, 0 frozen), for screenshots of the temperature states.
- `GAME_TOKEN=shape,attitude[,1]`: the player's token, as indexes into `TokenShape::ALL` / `Attitude::ALL` (e.g. `3,3` red hexagon, `4,0` white octagon); a third value `1` makes it completed.
- `GAME_WINDOW_SIZE=w,h`: resize the window (logical pixels) one second after starting, e.g. to check that pixel-sized things (token outlines, icons) follow resizes.
- `GAME_YAW=degrees`: start with the world map turned (`OrbitCamera::yaw`; 90 puts the camera in the east looking west, 180 looks south).
- `GAME_SCREENSHOT=path.png` (+ optional `GAME_SCREENSHOT_DELAY` seconds, default 5): save a screenshot, then quit 2 s later.

## Depends on

- `common` (`Weather`, `Token`, `TokenShape`, `Attitude`), `demo` (`AutoWalk`), `player` (`MapPosition`, `Player`, `TravelTo`), `world_map` (`OrbitCamera`).

## Used by

- Nothing (only through environment variables).

## Notes

- How Claude checks visual changes natively, e.g. from Git Bash:
  `GAME_SCREENSHOT=C:/Projects/game/target/shots/map.png ./target/debug/hall-of-worlds.exe`
  then read the PNG. Keep screenshots under `target/` (git-ignored).
- Python with Pillow is installed on the owner's machine for analysing screenshots (e.g. mean brightness, cropping the HUD).
- To find which Bevy system is failing in a panic, build with `cargo build -p client --features bevy/debug --target-dir target/debug-names` (system names are hidden otherwise).
