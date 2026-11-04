# client: dev_tools

Developer helpers driven by environment variables. Compiled for native only (`#[cfg(not(target_arch = "wasm32"))]` in `main.rs`) and inert unless a variable is set. Starting states (position, site, travel, token, view) are applied on leaving `GameView::Connecting`, once the server has said which world it is.

## Files

- `crates/client/src/dev_tools/mod.rs`: `DevToolsPlugin`. Each group of `HOW_DEBUG_` variables is read by an `add(app)` in its own file: `start.rs` (position, entering a site, cells), `forced.rs` (`Forced`: stairs, fields, cursor), `menus.rs` (panels, menus, the full map), `inventory.rs`, `moving.rs`, `world.rs` (time, suns and moons, snow), `view.rs` (herbs, token, camera), `window.rs` (`HOW_DEBUG_WINDOW`, with `parse_window`), `title.rs` (`HOW_DEBUG_TITLE`), `screenshot.rs`, `keep_drawing.rs`, `skip_menu.rs`; `parse.rs` reads their values.
- `crates/client/src/dev_tools/fake_input.rs`: only with the `debug-api` feature: fake mouse, touch, wheel, keys and typing for the debug API, queued and played a step a frame. In `fake_input/`: `play.rs`, `mouse.rs`, `keys.rs`, `wait.rs`.
- `crates/client/src/dev_tools/debug_api.rs`: only with the `debug-api` feature: the client's debug API methods (see `docs/context/debug-api.md`), one for most of these variables, going through the same code: `Forced` (stairs style, fields stage and cursor kept as set, applied every frame by `apply_forced`), `open_map`, `turn_camera`, `token_of`, `cell_kind`. In `debug_api/`, by what the methods do: `state.rs` (state, screenshot), `places.rs`, `world.rs`, `items.rs`, `moving.rs`, `view.rs`, `drawing.rs`, `panels.rs`.

## Public API

Boolean variables are read with `server::option_on`, so `1`, `on` and `yes` work as well as `true`.

- `HOW_DEBUG_POS=x,y`: starting grid position on the world map (sends `DevTeleport`; the server answers with `Teleported`). Sent before `HOW_DEBUG_ENTER` (system sets `Start::Place`, then `Start::Enter`).
- `HOW_DEBUG_ENTER=kind[,seed[,level[,ruin]]]`: start inside a made-up site at the starting position (sends `DevEnterSite`, which only the embedded server accepts): `kind` indexes `PoiKind::ALL` (e.g. 20 castle, 4 rock cave, 29 crypt, 15 stone tower, 38 farm, 39 herbalist's hut), `seed` its map seed (default 1), `level` (default 0; other levels arrive on their first stairs), `ruin` a building's ruin, 0 to 5 (default 0; saved apart from the unruined site, and asking for a level past the top lands on the top one).
- `HOW_DEBUG_CELLS=x,y,kind;...`: on arriving in a site, change cells from where the player stands (`DevSetCell`; `kind`: `floor`, `wall`, `secret`, `door`, `window`, `water`, `lava`, `hole`, `ground`, `covered`), to look at walls and liquids the generators don't make.
- `HOW_DEBUG_MOVE=dx,dy`: two seconds after starting, step the player this far (`player::TakeStep`s, so not into walls; e.g. off the stairs they arrived on, to look at them).
- `HOW_DEBUG_STAIRS=n`: draw all stairs as this style (an index into `StairsStyle::ALL`: 0 spiral, 1 turning, 2 ladder, 3 log ladder, 4 rope ladder, 5 slat ladder, 6 knotted rope).
- `HOW_DEBUG_FIELDS=n`: draw every field as this stage (its `variety`: for crops 0 ploughed, 1 grass, 2 short, 3 tall, 4 ripe; pasture alternates grass and short).
- `HOW_DEBUG_HERBS=n`: draw every herb tile `n` stages (game days) further on than the clock says (`common::HerbGrowthShift`).
- `HOW_DEBUG_TRAVEL=x,y`: start travelling there (as if clicked), on either map, once the player is at `HOW_DEBUG_POS` if that is set, otherwise two seconds after starting (the server's answer, or putting the player back in their site, would end an earlier journey); combine with `HOW_DEBUG_SCREENSHOT_DELAY` to catch it mid-journey.
- `HOW_DEBUG_WALK=true`: start with `debug_menu::AutoWalk` on (one step east per frame); with a screenshot, the HUD shows the per-step update cost.
- `HOW_DEBUG_SNOW=n`: once connected, ask the server (`DevSnowLine`) for the snow line at `n` (128 normal, 0 frozen), for screenshots of the temperature states.
- `HOW_DEBUG_TIME=n`: once connected, ask the server (`DevSetTime`) to move the clock to `n` game hours since the world began, for looking at another time of day. Saved like any other time, so keep to a scratch data folder.
- `HOW_DEBUG_CELESTIAL=n`: the suns and moons of terrain seed `n`, not this world's, for their light only (the tide stays this world's, as the server has it). 272 has two suns and two moons, 142 two suns and five moons, 144 one blue sun and no moons.
- `HOW_DEBUG_TOKEN=shape,attitude[,completed[,size]]`: the player's token, as indexes into `TokenShape::ALL` / `Attitude::ALL` (e.g. `3,3` red hexagon, `4,0` white octagon); a third value `1` makes it completed; a fourth is an index into `TokenSize::ALL` (default 2, `Default`).
- `HOW_DEBUG_WINDOW`: how the window opens, the same every run: `full-screen,WxH` (exclusive full screen at that resolution, physical pixels, or the nearest the screen has, with a warning), `full-screen` (borderless), `maximised`, or `WxH` (windowed, logical pixels, centred on the main monitor). Full screen opens on the main monitor. Sets `common::DisplayInUse` and turns `RememberWindow` off, so nothing about the window is saved over the player's own. It doesn't skip the startup menu. To check that views follow a resize while running, use the debug API's `client.window_size`, which also stops the window being saved.
- `HOW_DEBUG_TITLE=text`: `text` and a space before the window's title (blank: unchanged), so the copies Claude starts read "Dev: Hall of Worlds" (`HOW_DEBUG_TITLE=Dev:`) beside the owner's. It doesn't skip the startup menu.
- `HOW_DEBUG_VIEW=yaw[,pitch[,zoom]]`: start with the camera turned by `yaw` degrees (90 puts the camera in the east looking west, 180 looks south), tilted to `pitch` degrees above the horizon and zoomed in (the distance is divided by `zoom`: 1.6667 is the closest the wheel allows). Sets `OrbitCamera` fields directly, so the limits don't apply; below 1 the map's window stops growing at `world_map::MAX_HALF` and so gets smaller on screen.
- `HOW_DEBUG_MAP=dx,dy`: two seconds after starting, open the full map, moved this many grid points from the player, without the limits on dragging (`0,0` centres it on them; far enough off shows the arrow).
- `HOW_DEBUG_MAP_ZOOM=n`: the map `HOW_DEBUG_MAP` opens at this zoom.
- `HOW_DEBUG_FIRST_FLOOR=n`: number floors from this (0 or 1), whatever the locale says.
- `HOW_DEBUG_SHOW_FLOOR=n`: three seconds after starting, look at this known floor (an index, from 0); needs the full map open (`HOW_DEBUG_MAP`), as closed it goes straight back.
- `HOW_DEBUG_INVENTORY=true`: have a few items and, two seconds after starting, show the inventory, clothes, character panel and any bags. `HOW_DEBUG_HOLD=n`: no bags, but hold the item at this index in the inventory; with `HOW_DEBUG_CURSOR` to place it. `HOW_DEBUG_CREATURE=n`: first become this index in `CreatureKind::ALL`. Items are saved, so keep to a scratch data folder.
- `HOW_DEBUG_CURSOR=x,y`: hold the mouse cursor here (logical pixels), to look at what hovering shows.
- `HOW_DEBUG_PANEL=1`: open the debug menu's Show Widgets panel; `HOW_DEBUG_PANEL=suns`: its Suns and Moons panel; `HOW_DEBUG_PANEL=generate`: Generate POI's submenu.
- `HOW_DEBUG_MENU=text`: start with the debug menu open, filtered by `text`.
- `HOW_DEBUG_SYSTEM_MENU=menu`: start with the system menu open; `quit`: at its requester; `settings`: at its settings.
- `HOW_DEBUG_LOADING_HOLD=true`: keep the loading screen up (`loading_screen::HoldLoadingScreen`), to look at it.
- `HOW_DEBUG_SKIP_MENU=true`: start past the startup menu, as if Play was pressed (`common::Session::Playing`). On by default when any other `HOW_DEBUG_` variable is set, but for the debug API's, `HOW_DEBUG_GUEST`, `HOW_DEBUG_KEEP_DRAWING`, `HOW_DEBUG_TITLE`, `HOW_DEBUG_WINDOW` and the server's; `false` keeps the menu (to look at it with a screenshot).
- `HOW_DEBUG_KEEP_DRAWING=false`: while the debug API (`HOW_DEBUG_API`) or `HOW_DEBUG_SCREENSHOT` is running, frames keep their focused rate (about 30 fps) in the background (`common::KeepDrawing`), so tools driving a window behind others aren't held to one a second; `false` gives the players' rate back.
- `HOW_DEBUG_REVEAL_POIS=true`: show every poi on this connection (sends `DevRevealPois`), not just towns and known ones. Not saved. Changes nothing while the server's `ALL_POIS_KNOWN` is on.
- `HOW_DEBUG_SCREENSHOT=path.png` (+ optional `HOW_DEBUG_SCREENSHOT_DELAY` seconds, default 5): save a screenshot, then quit 2 s later.
- Not here, but also dev tools the embedded server allows: the debug menu's Teleport Mode (`world_map`, `dungeon_map`; on a poi on the full map it goes into it, `minimap`), Change Cell Mode (`dungeon_map`) and Teleport to Start (`debug_menu`).
- `LOG_LEVEL=debug` (read by `server::log_plugin`): log more from our crates; see `docs/context/server/server.md`.
- `HOW_SERVER=wss://host:port/ws`, or `ws://` (read by `connection`): join that server instead of starting single player (the same guest each run, unless `HOW_DEBUG_GUEST=<slot>` names another, for several clients on one computer; dev tools messages are ignored by such servers).
- `HOW_DATA=folder` (read by the server, not this module): where the embedded server saves. Set it to a scratch folder for test runs, or they move the owner's saved character (the saved position and site win unless `HOW_DEBUG_POS` / `HOW_DEBUG_ENTER` are set).

## Depends on

- `common` (`FirstFloor`, `OtherLevels`, `GameView`, `Token`, `TokenShape`, `Attitude`, `MapPosition`), `connection` (`ToServer`), `shared` (`poi::PoiKind`, `protocol::ClientMessage`), `debug_menu` (`AutoWalk`, `DebugAction` incl. `named` and `names`, `DebugMenu`), `inventory` (`InventoryDebug`), `player` (`Player`, `TravelTo`), `view3d` (`OrbitCamera`), `minimap` (`FullMap`), `loading_screen` (`HoldLoadingScreen`), `system_menu` (`SystemMenu`), `server::debug_api` (`AddDebugMethod`, with the feature).

## Used by

- Nothing (only through environment variables).

## Notes

- How Claude checks visual changes natively, e.g. from Git Bash:
  `HOW_DEBUG_TITLE=Dev: HOW_DEBUG_SCREENSHOT=C:/Projects/game/target/shots/map.png <target>/debug/hall-of-worlds.exe`, where `<target>` is cargo's target folder (`target/` unless moved)
  then read the PNG. Keep screenshots under `target/` (git-ignored).
- Python with Pillow is installed on the owner's machine for analysing screenshots (e.g. mean brightness, cropping the HUD).
- To find which Bevy system is failing in a panic, build with `cargo build -p client --features bevy/debug --target-dir <target>/debug-names` (inside cargo's target folder, wherever that is) (system names are hidden otherwise).
- `HOW_DEBUG_BAGS=true`: a row of every look of bag east of the player, only drawn, to look at.
- `HOW_DEBUG_SECRETS=true`, with `HOW_DEBUG_INVENTORY`: truesight on, and rings, hats and socks that are magical, blessed and cursed, to look at how they glow.
- `HOW_DEBUG_DROP=true`, with `HOW_DEBUG_INVENTORY`: all but the first of what is in the inventory goes on the ground underfoot, to look at the ground section.
