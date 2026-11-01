# client: dev_tools

Developer helpers driven by environment variables. Compiled for native only (`#[cfg(not(target_arch = "wasm32"))]` in `main.rs`) and inert unless a variable is set. Starting states (position, site, travel, token, view) are applied on leaving `GameView::Connecting`, once the server has said which world it is.

## Files

- `crates/client/src/dev_tools/mod.rs`: `DevToolsPlugin`.

## Public API

Boolean variables are read with `server::option_on`, so `1`, `on` and `yes` work as well as `true`.

- `HOW_DEBUG_POS=x,y`: starting grid position on the world map (sends `DevTeleport`; the server answers with `Teleported`). Sent before `HOW_DEBUG_ENTER` (system sets `Start::Place`, then `Start::Enter`).
- `HOW_DEBUG_ENTER=kind[,seed[,level[,ruin]]]`: start inside a made-up site at the starting position (sends `DevEnterSite`, which only the embedded server accepts): `kind` indexes `PoiKind::ALL` (e.g. 20 castle, 4 rock cave, 29 crypt, 15 stone tower, 38 farm, 39 herbalist's hut), `seed` its map seed (default 1), `level` (default 0; other levels arrive on their first stairs), `ruin` a building's ruin, 0 to 5 (default 0; saved apart from the unruined site, and asking for a level past the top lands on the top one).
- `HOW_DEBUG_CELLS=x,y,kind;...`: on arriving in a site, change cells from where the player stands (`DevSetCell`; `kind`: `floor`, `wall`, `door`, `window`, `water`, `lava`, `hole`, `ground`, `covered`), to look at walls and liquids the generators don't make.
- `HOW_DEBUG_MOVE=dx,dy`: two seconds after starting, step the player this far (`player::TakeStep`s, so not into walls; e.g. off the stairs they arrived on, to look at them).
- `HOW_DEBUG_STAIRS=n`: draw all stairs as this style (an index into `StairsStyle::ALL`: 0 spiral, 1 turning, 2 ladder, 3 log ladder, 4 rope ladder, 5 slat ladder, 6 knotted rope).
- `HOW_DEBUG_FIELDS=n`: draw every field as this stage (its `variety`: for crops 0 ploughed, 1 grass, 2 short, 3 tall, 4 ripe; pasture alternates grass and short).
- `HOW_DEBUG_HERBS=n`: draw every herb tile `n` stages (game days) further on than the clock says (`common::HerbGrowthShift`).
- `HOW_DEBUG_TRAVEL=x,y`: start travelling there (as if clicked), on either map, once the player is at `HOW_DEBUG_POS` if that is set, otherwise two seconds after starting (the server's answer, or putting the player back in their site, would end an earlier journey); combine with `HOW_DEBUG_SCREENSHOT_DELAY` to catch it mid-journey.
- `HOW_DEBUG_WALK=true`: start with `debug_menu::AutoWalk` on (one step east per frame); with a screenshot, the HUD shows the per-step update cost.
- `HOW_DEBUG_SNOW=n`: once connected, ask the server (`DevSnowLine`) for the snow line at `n` (128 normal, 0 frozen), for screenshots of the temperature states.
- `HOW_DEBUG_TIME=n`: once connected, ask the server (`DevSetTime`) to move the clock to `n` game hours since the world began, for looking at another time of day. Saved like any other time, so keep to a scratch data folder.
- `HOW_DEBUG_CELESTIAL=n`: the suns and moons of terrain seed `n`, not this world's. 272 has two suns and two moons, 142 two suns and five moons, 144 one blue sun and no moons.
- `HOW_DEBUG_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as indexes into `TokenShape::ALL` / `Attitude::ALL` (e.g. `3,3` red hexagon, `4,0` white octagon); a third value `1` makes it completed; a fourth is an index into `TokenSize::ALL` (default 2, `Default`).
- `HOW_DEBUG_WINDOW_SIZE=w,h`: resize the window (logical pixels) one second after starting, e.g. to check that pixel-sized things (token outlines, icons) follow resizes.
- `HOW_DEBUG_VIEW=yaw[,pitch[,zoom]]`: start with the camera turned by `yaw` degrees (90 puts the camera in the east looking west, 180 looks south), tilted to `pitch` degrees above the horizon and zoomed in (the distance is divided by `zoom`: 1.6667 is the closest the wheel allows). Sets `OrbitCamera` fields directly, so the limits don't apply.
- `HOW_DEBUG_MAP=dx,dy`: two seconds after starting, open the full map, moved this many grid points from the player, without the limits on dragging (`0,0` centres it on them; far enough off shows the arrow).
- `HOW_DEBUG_MAP_ZOOM=n`: the map `HOW_DEBUG_MAP` opens at this zoom.
- `HOW_DEBUG_FIRST_FLOOR=n`: number floors from this (0 or 1), whatever the locale says.
- `HOW_DEBUG_SHOW_FLOOR=n`: three seconds after starting, look at this known floor (an index, from 0); needs the full map open (`HOW_DEBUG_MAP`), as closed it goes straight back.
- `HOW_DEBUG_CURSOR=x,y`: hold the mouse cursor here (logical pixels), to look at what hovering shows.
- `HOW_DEBUG_PANEL=1`: open the debug menu's Show Widgets panel; `HOW_DEBUG_PANEL=suns`: its Suns and Moons panel; `HOW_DEBUG_PANEL=generate`: Generate POI's submenu.
- `HOW_DEBUG_MENU=text`: start with the debug menu open, filtered by `text`.
- `HOW_DEBUG_LOADING_HOLD=true`: keep the loading screen up (`loading_screen::HoldLoadingScreen`), to look at it.
- `HOW_DEBUG_REVEAL_POIS=true`: show every poi on this connection (sends `DevRevealPois`), not just towns and known ones. Not saved. Changes nothing while the server's `ALL_POIS_KNOWN` is on.
- `HOW_DEBUG_SCREENSHOT=path.png` (+ optional `HOW_DEBUG_SCREENSHOT_DELAY` seconds, default 5): save a screenshot, then quit 2 s later.
- Not here, but also dev tools the embedded server allows: the debug menu's Teleport Mode (`world_map`, `dungeon_map`; on a poi on the full map it goes into it, `minimap`), Change Cell Mode (`dungeon_map`) and Teleport to Start (`debug_menu`).
- `LOG_LEVEL=debug` (read by `server::log_plugin`): log more from our crates; see `docs/context/server/server.md`.
- `HOW_SERVER=wss://host:port/ws`, or `ws://` (read by `connection`): join that server instead of starting single player (a new guest each run; dev tools messages are ignored by such servers).
- `HOW_DATA=folder` (read by the server, not this module): where the embedded server saves. Set it to a scratch folder for test runs, or they move the owner's saved character (the saved position and site win unless `HOW_DEBUG_POS` / `HOW_DEBUG_ENTER` are set).

## Depends on

- `common` (`FirstFloor`, `OtherLevels`, `GameView`, `Token`, `TokenShape`, `Attitude`, `MapPosition`), `connection` (`ToServer`), `shared` (`poi::PoiKind`, `protocol::ClientMessage`), `debug_menu` (`AutoWalk`, `DebugAction`, `DebugMenu`), `player` (`Player`, `TravelTo`), `view3d` (`OrbitCamera`), `minimap` (`FullMap`), `loading_screen` (`HoldLoadingScreen`).

## Used by

- Nothing (only through environment variables).

## Notes

- How Claude checks visual changes natively, e.g. from Git Bash:
  `HOW_DEBUG_SCREENSHOT=C:/Projects/game/target/shots/map.png ./target/debug/hall-of-worlds.exe`
  then read the PNG. Keep screenshots under `target/` (git-ignored).
- Python with Pillow is installed on the owner's machine for analysing screenshots (e.g. mean brightness, cropping the HUD).
- To find which Bevy system is failing in a panic, build with `cargo build -p client --features bevy/debug --target-dir target/debug-names` (system names are hidden otherwise).
