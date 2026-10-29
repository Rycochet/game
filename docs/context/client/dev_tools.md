# client: dev_tools

Developer helpers driven by environment variables. Compiled for native only (`#[cfg(not(target_arch = "wasm32"))]` in `main.rs`) and inert unless a variable is set.

## Files

- `crates/client/src/dev_tools/mod.rs`: `DevToolsPlugin`.

## Public API

- `GAME_POS=x,y`: starting grid position on the world map.
- `GAME_ENTER=kind[,seed[,level]]`: start inside a site, as if the feature at the starting position were entered: `kind` indexes `FeatureKind::ALL` (e.g. 20 castle, 4 rock cave, 29 crypt, 15 stone tower), `seed` its map seed (default 1), `level` (default 0; other levels arrive on their first stairs).
- `GAME_MOVE=dx,dy`: a second after starting, move the player by this much (e.g. off the stairs they arrived on, to look at them).
- `GAME_TRAVEL=x,y`: start travelling there (as if clicked); combine with `GAME_SCREENSHOT_DELAY` to catch it mid-journey.
- `GAME_WALK=1`: start with `demo::AutoWalk` on (one step east per frame); with a screenshot, the HUD shows the per-step update cost.
- `GAME_SNOW=n`: start with `Weather::snow_line` at `n` (128 normal, 0 frozen), for screenshots of the temperature states.
- `GAME_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as indexes into `TokenShape::ALL` / `Attitude::ALL` (e.g. `3,3` red hexagon, `4,0` white octagon); a third value `1` makes it completed; a fourth is an index into `TokenSize::ALL` (default 2, `Default`).
- `GAME_WINDOW_SIZE=w,h`: resize the window (logical pixels) one second after starting, e.g. to check that pixel-sized things (token outlines, icons) follow resizes.
- `GAME_VIEW=yaw[,pitch[,zoom]]`: start with the camera turned by `yaw` degrees (90 puts the camera in the east looking west, 180 looks south), tilted to `pitch` degrees above the horizon and zoomed in (the distance is divided by `zoom`: 1.6667 is the closest the wheel allows). Sets `OrbitCamera` fields directly, so the limits don't apply.
- `GAME_SCREENSHOT=path.png` (+ optional `GAME_SCREENSHOT_DELAY` seconds, default 5): save a screenshot, then quit 2 s later.

## Depends on

- `common` (`Weather`, `Token`, `TokenShape`, `Attitude`, `MapPosition`), `site` (`EnterFeature`, `TakeExit`), `shared` (`feature`, `rng`, `site::Exit`), `demo` (`AutoWalk`), `player` (`Player`, `TravelTo`), `view3d` (`OrbitCamera`).

## Used by

- Nothing (only through environment variables).

## Notes

- How Claude checks visual changes natively, e.g. from Git Bash:
  `GAME_SCREENSHOT=C:/Projects/game/target/shots/map.png ./target/debug/hall-of-worlds.exe`
  then read the PNG. Keep screenshots under `target/` (git-ignored).
- Python with Pillow is installed on the owner's machine for analysing screenshots (e.g. mean brightness, cropping the HUD).
- To find which Bevy system is failing in a panic, build with `cargo build -p client --features bevy/debug --target-dir target/debug-names` (system names are hidden otherwise).
