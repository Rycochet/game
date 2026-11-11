# (debug api)

Bevy's remote protocol (`bevy_remote`: JSON-RPC over HTTP, localhost only) in the native client, its embedded server and the standalone server, for looking at and driving a running game without restarting it. For Claude and other tools; never in releases or the web build. Not a module of its own: the setup is `server::debug_api`, and each module adds its own methods.

## Turning it on

- Build with `--features debug-api` (`cargo build -p client --features debug-api`; the client's feature turns on the server's). Off by default. `compile_error!` in `server::debug_api` refuses the feature in any release build, so nothing that ships (`cargo xtask package`, the Docker image, CI's `cargo check --release`) can have it. The handlers live in files compiled only with the feature, so their code never reaches other builds.
- Start with `HOW_DEBUG_API=true` (port 15702) or `HOW_DEBUG_API=<port>`. The client and a standalone server take that port; the client's embedded server the next (15703). Without the feature, setting it only logs a warning.
- A port already in use (another copy running) is logged as a warning and the program starts without the API: only callers are stuck. `client.state` / `server.state` give the process id and data folder, so a caller can tell whose game answered. The owner's copy from `cargo xtask latest` (`play.cmd`) uses port 15712 (`--port 15712`), so development copies on the default port never reach it.
- Switching the feature on or off recompiles part of Bevy the first time (bevy_remote turns on some Bevy features, and the client's feature turns on `bevy/serialize` for key and button names); cargo then keeps both builds.

## Calling it

- `cargo xtask api [--server | --port N] <method> [key=value ... | '{json}']`: posts the request, prints the answer indented, fails on an error. `--server` is the embedded server's port. A value that isn't JSON is sent as a string (`path=target/shots/a.png`). If the answer has `wait_for_file`, it waits (up to 15 s) for that file to exist and stop growing; with `wait_for_input`, until the fake input queued has been played (up to 120 s).
- Or any HTTP client: `POST http://127.0.0.1:15702` with `{"jsonrpc":"2.0","id":1,"method":"client.state"}`.
- `rpc.discover` lists every method. Bevy's own (`world.query`, `world.get_resources`, `world.mutate_components`, ...) work on types that derive `Reflect` and are registered, which few of ours are yet.

## Methods

Kinds of things (`PoiKind`, `ItemKind`, `Quality`, `Favour`, `CreatureKind`, `Cell`) are given by the names they are saved by (`"Castle"`, `"Ring"`, `"Rare"`, `"SecretDoor"`); a wrong one is answered with the list of right ones. Methods that only do something answer `"ok"`; what they ask of the server arrives a tick or so later, so check with a state method.

Client (`client/src/dev_tools/debug_api.rs`; each says which `HOW_DEBUG_` variable it matches, and shares its code):

- `client.state`: program, pid, data folder, `view` (`GameView`), player `position`, `target` (`common::Target`), `level` (the `LevelInfo` in a site), `showing_floor`, clock minutes and paused, window size and scale factor, `ui_scale` (the interface's size, 1 for 100%), `full_map`, `debug_menu`, `system_menu`.
- `client.screenshot {path}`: saves the window at the end of the frame to `path` (absolute or relative to the game's folder; folders made, an old file removed first); answers with the full path and `wait_for_file`. The game keeps running.
- Asking the server: `client.teleport {x, y}`, `client.enter_site {kind, seed?, level?, ruin?}`, `client.create_mob {kind, x, y, attitude?, gear?, still?, seed?}` (`debug_api/mobs.rs`: a mob made to order on a cell, as a click in Mob Mode; `attitude` by name, the kind's own if missing; gear on by default), `client.inspect_mob {id}` (asks the server about a mob in sight, as Examine Mob does for the target) and `client.inspected` (`mob`, `lines`: the last answer), `client.dev_area {size?, generator?, mobs?, count?, seed?}` (a Dev Area as the panel makes, by names: `"Moat"`, `"Wolf"`; a seed of words is hashed; answers the size and seed used), `client.set_cells {cells: [{dx, dy, cell}]}` (from the player, in a site), `client.set_time {hours | minutes}`, `client.go_to_world {seed?, gate?}` (no seed: this world; `gate` from 0), `client.reveal_pois`, `client.give_item {kind, quality?, magical?, set?, favour?}`, `client.wear_item {kind}` (puts on the first of that kind in the inventory), `client.set_creature {kind, how?}` (`how`: `Permanent`, the default, or `Temporary`).
- Moving: `client.step {dx, dy}` (keyboard steps), `client.travel {x, y}` (as a click), `client.walk {on}` (Auto Walk).
- Looks: `client.view {yaw?, pitch?, zoom?}` (degrees; zoom divides the present distance), `client.window_size {width, height}` (windowed, un-maximised; the window's place and size are then no longer saved), `client.interface {size?, touch?}` (`"auto"` or a percentage, and "Enable Touch", as Settings sets them, so saved: use `HOW_PREFERENCES` elsewhere), `client.cursor {x, y}` (held until `{}`), `client.stairs_style {style}` / `client.fields_stage {stage}` (kept until `null`), `client.herbs_shift {stages}`, `client.celestial {seed}`, `client.token {shape, attitude, completed?, size?}` (indexes).
- What is open: `client.map {open?, dx?, dy?, zoom?}`, `client.show_floor {floor}` (`null` for the player's own), `client.debug_action {name}` (any debug menu entry, by its name in any case; not checked for being available), `client.debug_menu {open?, filter?}`, `client.system_menu {show: closed | menu | quit | settings}`, `client.inventory {show: inventory | bags | hold | drop_some, index?}`.

- View Planet (`planet_view/debug_api.rs`): `client.planet {generator?, seed?, layer?, path?}`, with the view open: choose its generator (`Desert`, `Continents`, `Plates`), seed and layer (`Surface`, `Elevation`, `Plates`, `Temperature`, `Moisture`) as the bar would (the bar's widgets don't follow), and save its picture of the surface to `path` as a PNG, as far as it is filled; answers `filled`, `seed`, `world_gates` (`x`, `y`, `round` each) and `gate_tries`.
- Item pictures (`item_art/debug_api.rs`): `client.time_pictures {count?}` (300 if not given) times a second of frames, then asks for `count` pictures of items (the same looks every time, from `item_art::samples`) and times them until all are drawn; `client.picture_times` answers `running` or `pictures` (how many were new), `total_s`, `first_picture_s`, `pictures_per_s`, `frames`, `average_frame_ms`, `worst_frame_ms`, `idle_frame_ms`. Start it in a fresh game: pictures already drawn are kept.

Widgets by id (`client/src/widgets/debug_api.rs`; ids are `widgets::UiId`, `form/name`, see `client/widgets.md`). **Prefer these to clicks and screenshots** for anything with an id: they don't need the window's layout, the mouse, or a picture read.

- `client.ui.list {form?, all?}`: every widget on show (with `all`, hidden ones too), sorted by id: `id`, `kind` (`text_box`, `key_box`, `dropdown`, `colour_picker`, `slider`, `tabs`, `check`, `switch`, `hold_button`, `panel`, `scroll`, `button`, `other`), `label`, `state`, `shown`, `disabled`, `covered` (a full-screen layer is in front), `value` (with `index`, `options`, `greyed` for dropdowns and tabs; `error`, `focused`, `caret` and `selection` (`[from, to]` in places: characters, and an item written into a box that takes them as one, or `null`) for text boxes; for key boxes `value` is the binding's saved name (`Shift+KeyG`, `null` when empty) with `shows`, `warned`, `waiting`), and `at` / `size` in the window's logical pixels for `client.click`; and `duplicates`, ids more than one widget on show has. `form` is a form's name (all alike, as `bag`) or with which one (`bag#<id>`).
- `client.ui.get {id}`: one, the same way.
- `client.ui.set {id, value}`: a text box's words (a number is written out), a dropdown's, colour picker's or tab row's option by index or words (any case), a slider 0 to 1, a check button or switch true or false, a hold button true (held, from the next frame, until set false) or false, a scroll area's place in interface pixels. Sends the widget's own message (`TextEdited`, `DropdownChosen`, `SliderMoved`, a `ButtonPressed` on the tab, check or switch, which flips it and sends `SwitchFlipped`), as a player's change would.
- `client.ui.press {id}`: a `ButtonPressed` on a button (a dropdown opens its list, a switch flips). `client.ui.close {id}`: closes a moveable panel (`<form>/panel`), as its close button does.
- `client.ui.at {x?, y?}` (`client/src/dom/debug_api.rs`): the thing in front at a point in the window's logical pixels (or where the pointer is now): `target`, and `chain`, the target and everything it is in going outwards, each with its `id`, its `element` (markup's tag) and whether it is `solid`. What would be pointed at and pressed there, by the same search the game uses (`dom::Hit`); nothing in front is `null`.
- `client.ui.html`: every document of markup as it is now (`name`, `shown`, `html`), ids written whole (`form/name`), for reading a screen without a picture.
- All of them refuse an id not on show, disabled, behind a full-screen layer it isn't in (as when a dialog is open), or given to more than one widget on show, saying which. Changes land a frame or so later: `client.wait` before reading back.

Fake input (`client/src/dev_tools/fake_input.rs`): Bevy's own input messages (`MouseButtonInput`, `MouseWheel`, `TouchInput`, `KeyboardInput`) and the window's cursor, so the game takes them as a person's. Each request is queued and played one step a frame, before Bevy reads input (`PreUpdate`, before `InputSystems`), so a press and its release land on separate frames. Positions are logical pixels (`client.state`'s window size; screenshots are physical, so divide by the scale). They answer `wait_for_input`, and `cargo xtask api` waits (asking `client.input`) until all is played.

- `client.move {x, y}`, `client.click {x, y, button?, hold_frames?, touch?}` (`touch: true` taps with a finger instead), `client.drag {from: [x, y], to: [x, y], steps?, button?, touch?}`, `client.scroll {lines, x?, y?}` (positive scrolls up, zooming in).
- `client.key {key, hold_frames?, with?}` (Bevy's `KeyCode` names: `Escape`, `Backquote`, `KeyM`, `Enter`, `ArrowUp`, ...; `with` holds others round it, as `["ControlLeft"]` for Ctrl+V), `client.type {text}` (a key a frame, as `Key::Character`, into whatever has the keyboard).
- `client.wait {frames}`: answers once that many frames have passed after everything queued. `client.input`: `pending` steps and the `frame` count.
- A real mouse moved over the window at the same time fights the fake cursor.

Server (`server/src/debug_api/methods.rs`; `--server` for the embedded one):

- `server.state`: program (`server` or `embedded server`), pid, data folder, dev tools on, `home_world`, `server_clock` (seconds), `clock_minutes` (the home world's time), paused, action ticks, `players` (character id, phase, `world`, world position, `visit`: site id, level, position).
- `server.worlds`: every open world: id, whether it is the home, its time in minutes, `rate` (`[game, real]`), players in it, sites loaded and levels in use.
- `server.mobs {world?}`: every level in use (site, level) of a world (the home unless given its id) and its mobs: id, kind, position, home, `doing`, `target` (who it has in its sights as an enemy, if anyone).
- `server.cells {player?, radius?}`: a player's level around them as the server has it (secret doors included), rows from the north, `@` the player, blank outside the level, with a `legend` of the letters used, and `from` (the top-left cell).
- `server.sites {world?}`: a world's sites loaded (the home unless given its id), their generator version and the levels with saves.
- `server.tick {hold?, count?}` (`methods/tick.rs`): holds the action clock (`clock::TickHold`, a resource only the debug API adds) so it no longer follows real time, and moves it on by `count` ticks, one a frame; `hold: false` lets it run again. Answers `ticks`, `held`, `steps` still to come. For watching fights and mobs a step at a time.
- `server.save`: saves everything changed and answers once it is written (`written`: whether all of it was, within 8 s).

## Scenarios

`cargo xtask scenario <file> [--start]` (`xtask/src/scenario.rs`) plays a file of calls in order, each line what would follow `cargo xtask api` (`--server`, `key=value` pairs with `"..."` for spaces, or JSON from a `{`), `#` for comments, waiting for files and input as `api` does and stopping at the first that fails. With `--start` it builds the client with the debug API, starts it with its own scratch data folder (in the system's temp folder) and `HOW_DEBUG_TITLE=Dev:`, plays the file and closes it. Kept in `scenarios/` (`*.txt`, each saying how to run it at the top); a test checks every line names a method.

## Adding a method

- In the module that owns what it shows or does: `app.add_debug_method("<area>.<what>", handler)` in its plugin (`server::debug_api::AddDebugMethod`), the handler a system taking `In<Option<Value>>` (the params) and returning `BrpResult` (`Ok(json!(...))`). `params::<T>(request)?` reads the params into a `Deserialize` type (missing params read as `{}`). Methods added before or after `DebugApiPlugin` both work (`register_methods` puts them in place in `finish`); without the API they do nothing.
- Put handlers in a `debug_api.rs` (or `debug_api/`) file declared `#[cfg(feature = "debug-api")] mod debug_api;`, and the `add` call behind the same `cfg`. On the client the server crate is native only, so a module that also builds for the web needs `#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]`.
- Only add `#[cfg(feature = "debug-api")]` accessors to other code (as `Connections::clients`, `Visit::site_id`), never change what a normal build does.
- Changes go through the same paths as the game (messages to the server, the client's own systems): the API drives the client, it doesn't get round the server's checks.
- **Temporary methods** are named `tmp.<what>`: fine to commit on a branch, but `cargo xtask test` fails while any `"tmp.` string is in `crates/`. Before merging, delete them or rename them to keep them, and list kept ones here.

## Files

- `crates/server/src/debug_api/mod.rs`: always built: `DebugApiPlugin { port_offset }` (with the feature it starts the API; without it, warns if `HOW_DEBUG_API` is set), `VARIABLE`, `DEFAULT_PORT`, `port_from`, the release guard. Re-exports for method files: `AddDebugMethod`, `BrpError`, `BrpResult`, `Value`, `json`, `params`, `refused` (an answer saying what was wrong).
- `crates/server/src/debug_api/remote.rs`: feature only: `start` (tries the port, then `RemotePlugin` + `RemoteHttpPlugin` on 127.0.0.1), `PendingMethods`, `register_methods`, `AddDebugMethod`, `params`, `refused`. Tests.
- `crates/server/src/debug_api/methods.rs`: the `server.` methods, added by `ServerPlugin`. Uses `connections::save_all` and feature-only accessors `Connections::clients`, `Visit::site_id`, `Visit::cells`, `Storage::flush`. Test of the cells picture.
- `crates/client/src/dev_tools/fake_input.rs`: the fake input methods, `FakeInput` (the queue) and `play`. Tests (keys, drags, a step a frame).
- `crates/client/src/dev_tools/debug_api.rs`: the other `client.` methods. Uses `dev_tools`' `Forced`, `open_map`, `turn_camera` and `token_of`, which the variables use too, and `DebugAction::named` / `names`.
- `xtask/src/api.rs`: `cargo xtask api` (waiting for files and for input), and the `tmp.` check (`temporary_methods`). Tests. In `api/`: `request.rs`, `http.rs`, `json.rs`, `wait.rs`, `temporary.rs`.
- Where it is added: `main.rs` of the client (port offset 0) and of the server (after the log plugin), `embedded.rs` (offset 1).
