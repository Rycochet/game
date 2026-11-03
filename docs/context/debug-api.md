# (debug api)

Bevy's remote protocol (`bevy_remote`: JSON-RPC over HTTP, localhost only) in the native client, its embedded server and the standalone server, for looking at and driving a running game without restarting it. For Claude and other tools; never in releases or the web build. Not a module of its own: the setup is `server::debug_api`, and each module adds its own methods.

## Turning it on

- Build with `--features debug-api` (`cargo build -p client --features debug-api`; the client's feature turns on the server's). Off by default. `compile_error!` in `server::debug_api` refuses the feature in any release build, so nothing that ships (`cargo xtask package`, the Docker image, CI's `cargo check --release`) can have it. The handlers live in files compiled only with the feature, so their code never reaches other builds.
- Start with `HOW_DEBUG_API=true` (port 15702) or `HOW_DEBUG_API=<port>`. The client and a standalone server take that port; the client's embedded server the next (15703). Without the feature, setting it only logs a warning.
- A port already in use (another copy running) is logged as a warning and the program starts without the API: only callers are stuck. `client.state` / `server.state` give the process id and data folder, so a caller can tell whose game answered.
- Switching the feature on or off recompiles part of Bevy the first time (bevy_remote turns on some Bevy features); cargo then keeps both builds.

## Calling it

- `cargo xtask api [--server | --port N] <method> [key=value ... | '{json}']`: posts the request, prints the answer indented, fails on an error. `--server` is the embedded server's port. A value that isn't JSON is sent as a string (`path=target/shots/a.png`). If the answer has `wait_for_file`, it waits (up to 15 s) for that file to exist and stop growing.
- Or any HTTP client: `POST http://127.0.0.1:15702` with `{"jsonrpc":"2.0","id":1,"method":"client.state"}`.
- `rpc.discover` lists every method. Bevy's own (`world.query`, `world.get_resources`, `world.mutate_components`, ...) work on types that derive `Reflect` and are registered, which few of ours are yet.

## Methods

- `client.state` (`client/src/dev_tools/debug_api.rs`): program, pid, data folder, `view` (`GameView`), player `position`, `level` (the `LevelInfo` in a site), clock minutes and paused, window size.
- `client.screenshot {path}` (same file): saves the window at the end of the frame to `path` (absolute or relative to the game's folder; folders made, an old file removed first); answers with the full path and `wait_for_file`. The game keeps running.
- `server.state` (`server/src/debug_api/state.rs`): program (`server` or `embedded server`), pid, data folder, dev tools on, clock minutes and paused, `players` (character id, phase, world position, `visit`: site id, level, position).

## Adding a method

- In the module that owns what it shows or does: `app.add_debug_method("<area>.<what>", handler)` in its plugin (`server::debug_api::AddDebugMethod`), the handler a system taking `In<Option<Value>>` (the params) and returning `BrpResult` (`Ok(json!(...))`). `params::<T>(request)?` reads the params into a `Deserialize` type (missing params read as `{}`). Methods added before or after `DebugApiPlugin` both work (`register_methods` puts them in place in `finish`); without the API they do nothing.
- Put handlers in a `debug_api.rs` (or `debug_api/`) file declared `#[cfg(feature = "debug-api")] mod debug_api;`, and the `add` call behind the same `cfg`. On the client the server crate is native only, so a module that also builds for the web needs `#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]`.
- Only add `#[cfg(feature = "debug-api")]` accessors to other code (as `Connections::clients`, `Visit::site_id`), never change what a normal build does.
- Changes go through the same paths as the game (messages to the server, the client's own systems): the API drives the client, it doesn't get round the server's checks.
- **Temporary methods** are named `tmp.<what>`: fine to commit on a branch, but `cargo xtask test` fails while any `"tmp.` string is in `crates/`. Before merging, delete them or rename them to keep them, and list kept ones here.

## Files

- `crates/server/src/debug_api/mod.rs`: always built: `DebugApiPlugin { port_offset }` (with the feature it starts the API; without it, warns if `HOW_DEBUG_API` is set), `VARIABLE`, `DEFAULT_PORT`, `port_from`, the release guard. Re-exports for method files: `AddDebugMethod`, `BrpError`, `BrpResult`, `Value`, `json`, `params`.
- `crates/server/src/debug_api/remote.rs`: feature only: `start` (tries the port, then `RemotePlugin` + `RemoteHttpPlugin` on 127.0.0.1), `PendingMethods`, `register_methods`, `AddDebugMethod`, `params`. Tests.
- `crates/server/src/debug_api/state.rs`: `server.state`, added by `ServerPlugin`.
- `crates/client/src/dev_tools/debug_api.rs`: `client.state`, `client.screenshot`.
- `xtask/src/api.rs`: `cargo xtask api`, and the `tmp.` check (`temporary_methods`). Tests.
- Where it is added: `main.rs` of the client (port offset 0) and of the server (after the log plugin), `embedded.rs` (offset 1).

## Planned

- PR 2: server methods (mobs on a level, cells, clocks, loaded sites, save and wait), client methods for every `dev_tools` variable (sharing their code, the variables kept) and running debug menu entries by name.
- PR 3: fake input (click, drag, key, type) and waiting for frames or until idle.
