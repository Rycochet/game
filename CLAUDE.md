# Hall of Worlds: guide for Claude

**Hall of Worlds** is a multiplayer roguelike with an infinite, procedurally generated world, written in Rust with the Bevy engine. Client/server: the client builds natively (Windows/macOS/Linux) and to WebAssembly (browser); the server builds natively only. Design notes are in `README.md` and `designs.txt`.

## Core rules (always apply)

- **Don't trust the user.** The server is authoritative and validates every command against the same rules/data as the client. Clients send intents, never results. The client only predicts.
- **Determinism.** Anything both sides compute (world generation, validation) lives in shared code, uses only the project's seeded, position-hashed RNG, integer game state, and no platform-dependent float functions.
- Full details, including how to decide shared vs. client-only code: `docs/context/architecture.md`. Read it before touching game rules, generation, randomness or networking.

The project owner is new to Rust: explain Rust concepts briefly when they come up, and prefer clear code over clever code.

## Tech stack

Rust stable (pinned in `rust-toolchain.toml`), Bevy 0.20, Trunk for the web build, WebSockets between client and server, postcard for messages. Details: `docs/context/project.md`.

Crates: `crates/shared` (plain Rust, no Bevy: rules and generation), `crates/client` (Bevy app, binary `hall-of-worlds`; native builds embed the server for single player), `crates/server` (native only, binary `hall-of-worlds-server`: the networked server, also serving the web client), `xtask` (helper commands).

## How to load context (read this first)

**Do not read the whole codebase.** Each feature module has a short context file in `docs/context/`. Read only the ones relevant to the task, then open only the source files they point to.

| Module              | Source                          | Context file                          | Covers                                              |
| ------------------- | ------------------------------- | ------------------------------------- | --------------------------------------------------- |
| (project)           | root files                      | `docs/context/project.md`             | Build, tooling, folder layout, CI                   |
| (design)            | n/a                             | `docs/context/architecture.md`        | Client/server split, trust, determinism, map data flow |
| (performance)       | n/a                             | `docs/context/optimisations.md`       | Measured optimisations tried but not adopted (memory) |
| shared: rng         | `crates/shared/src/rng.rs`      | `docs/context/shared/rng.md`          | Seeds, position hashing, random sequences           |
| shared: world       | `crates/shared/src/world.rs`    | `docs/context/shared/world.md`        | World settings from seed, tides                     |
| shared: surface     | `crates/shared/src/surface.rs`  | `docs/context/shared/surface.md`      | Terrain height/jitter at any grid position (noise)  |
| shared: climate     | `crates/shared/src/climate.rs`  | `docs/context/shared/climate.md`      | Temperature, moisture and Whittaker biomes at any world map position |
| shared: river       | `crates/shared/src/river.rs`    | `docs/context/shared/river.md`        | Rivers running downhill from a chunk's source, more where wet (visual only for now) |
| shared: vegetation  | `crates/shared/src/vegetation.rs` | `docs/context/shared/vegetation.md` | Tree cover and low growth at any world map position, from climate and patch noise (visual only for now) |
| shared: weather     | `crates/shared/src/weather.rs`  | `docs/context/shared/weather.md`      | Cloud cover and rain at any place and time (4D simplex noise) |
| shared: poi     | `crates/shared/src/poi.rs`  | `docs/context/shared/poi.md`      | World map chunks and their pois (kind, position, map seed) |
| shared: showcase    | `crates/shared/src/showcase.rs` | `docs/context/shared/showcase.md`     | Debug plain far south with every kind of poi, 8 seeds and each level of ruin, behind `ENABLED` (P to go, Esc to return) |
| shared: dungeon     | `crates/shared/src/dungeon.rs`  | `docs/context/shared/dungeon.md`      | Cell grid (floor, wall, doors, windows, ground, water, lava, roads, gardens, holes, fields, fences, herb beds), wall shapes, rooms, line of sight / light reach |
| shared: movement    | `crates/shared/src/movement.rs` | `docs/context/shared/movement.md`     | One-cell steps and the rules both sides check them by |
| shared: pathfinding | `crates/shared/src/pathfinding.rs` | `docs/context/shared/pathfinding.md` | Quickest route of legal steps across one level or the world map (A*), spreading diagonals out |
| shared: protocol    | `crates/shared/src/protocol/`   | `docs/context/shared/protocol.md`     | Client/server messages (world map chunks, site levels in 8×8 chunks), in-process `Link` (postcard bytes) |
| shared: site        | `crates/shared/src/site/`       | `docs/context/shared/site.md`         | Sites behind pois: building (BSP) with ruin and basements, cave (with skylights), town, farm and herbalist's hut generators, levels, stairs |
| server              | `crates/server/`                | `docs/context/server/server.md`       | Authoritative server in its own Bevy world: WebSocket networking (and the web client's files) on one port, connections, world map chunks, site visits and level cells, saves (world, sites, characters), embedded single player, standalone binary |
| client: connection  | `crates/client/src/connection/` | `docs/context/client/connection.md`   | Connecting to the server (embedded on native, WebSocket on the web), reconnecting, sending steps and putting refused ones right, `ToServer` / `FromServer` |
| client: common      | `crates/client/src/common/`     | `docs/context/client/common.md`       | Current world (and its pois) and dungeon, the site's other known floors and their numbering, `MapPosition` / `SmoothPosition`, `GameView` state (incl. `Connecting`), `ServerStatus`, `Portal` / `Clickable`, clock, weather, view stats, view facing, frame cap, `Token`, `LightSource`, `SharesSight` |
| client: player      | `crates/client/src/player/`     | `docs/context/client/player.md`       | Player position, keyboard steps, travel to clicks along found routes (announced as `Stepped`) |
| client: surface_style | `crates/client/src/surface_style/` | `docs/context/client/surface_style.md` | Planet colour bands and thresholds, biome colours |
| client: view3d      | `crates/client/src/view3d/`     | `docs/context/client/view3d.md`       | Shared 3D camera (turn, tilt, zoom), entity tokens, token clicks, `ViewGrid` |
| client: world_map   | `crates/client/src/world_map/`  | `docs/context/client/world_map.md`    | 3D terrain window cut to the camera, edge faces, sun, click picking, poi models, rivers, clouds, moving water, precipitation |
| client: dungeon_map | `crates/client/src/dungeon_map/` | `docs/context/client/dungeon_map.md` | Dungeon view: towns (roads, gardens with hedges, buildings with roofs and door signs), farms (fields, crops by stage, fences, gates), herb patches growing by the clock, textured floors and thin 3D walls, doors and windows by theme, 3D stairs in seven styles, liquid channels, holes in the floor, sight, see-through walls, grid light, memory in grey, outside ground and precipitation following the world map |
| client: minimap     | `crates/client/src/minimap/`    | `docs/context/client/minimap.md`      | Round minimap and M full map: known land with poi pictures, or remembered cells as a grid; drag, title, arrow to the player, floor buttons |
| client: site        | `crates/client/src/site/`       | `docs/context/client/site.md`         | Asking the server to enter sites, take stairs, leave; applying its level messages |
| client: debug_menu  | `crates/client/src/debug_menu/`  | `docs/context/client/debug_menu.md`   | Backtick or warning button: full-screen filtered list of debug actions (toggles, Teleport Mode, Cycle Temperature, Stairs/Fields/Herbs panels, Show Widgets) |
| client: touch       | `crates/client/src/touch/`      | `docs/context/client/touch.md`        | Fingers: tap as a left click, `PointerPosition`, `TouchGesture` (drag, pinch, twist, tilt) |
| client: widgets     | `crates/client/src/widgets/`    | `docs/context/client/widgets.md`      | Interface pieces: panels with titles, buttons by state, tooltips, moveable panels; taking clicks on them |
| client: ui          | `crates/client/src/ui/`         | `docs/context/client/ui.md`           | HUD status text, click mode text                    |
| client: loading_screen | `crates/client/src/loading_screen/` | `docs/context/client/loading_screen.md` | Full-screen title and step bar until the game takes input, and while the connection is lost; server refusals |
| client: dev_tools   | `crates/client/src/dev_tools/`  | `docs/context/client/dev_tools.md`    | Env-var helpers: start position or site, stairs style, travel, walk, snow, camera view, screenshots |

Also: `docs/context/decisions.md` is the log of architecture decisions. Check it before proposing a structural change, and follow what it records.

If a task touches a module not listed here, it doesn't exist yet: create it (see "Adding a module").

## Module rules

- **Client:** every feature is a folder `crates/client/src/<module>/` whose `mod.rs` exposes one Bevy `Plugin` (`<Name>Plugin`), registered in `crates/client/src/main.rs`, which contains no game logic.
- **Shared:** plain Rust modules (`crates/shared/src/<module>.rs`, or a folder when it grows) with no Bevy dependency so far. Anything the server will need to agree on goes here (see `architecture.md`).
- A module may use `common` freely. It should not reach into another feature module's internals. Cross-module communication goes through types the owning module deliberately makes `pub` (components, resources, messages/events), and each context file lists them under "Public API".
- If two modules need the same type, move it to `common` rather than having them depend on each other.
- Keep modules small enough that their context file stays short. Split a module (e.g. `world/terrain`, `world/render`) when it grows past what one context file can summarise.

## Keeping context files up to date (required)

Whenever a change affects a module, update its `docs/context/<crate>/<module>.md` **in the same commit / branch / PR** as the code. Update it when you change:

- the module's files (added, removed, renamed),
- its public API (anything `pub` other modules use),
- which modules it depends on, or which depend on it,
- a non-obvious behaviour, gotcha or invariant.

Context files are summaries, not documentation: a few lines per item, enough to decide which file to open. Don't paste code into them. Also update the table above and `docs/context/project.md` when modules or tooling change, and add an entry to `decisions.md` when a structural decision is made (including decisions the owner makes in conversation).

## Before a branch is merged

When the owner says a branch is ready to merge (or asks for a PR):

1. Run `cargo xtask test`; it must pass (CI only runs `cargo check --release` for now, so this local run is the real test).
2. Check every context file the branch touched still matches the code.
3. Tidy `docs/context/decisions.md` following the maintenance rules at its top: fix outdated entries freely, but **ask before removing anything**, listing what and why.

## Adding a module

1. Client: create `crates/client/src/<module>/mod.rs` with a `<Name>Plugin` and register it in `main.rs`. Shared: add `crates/shared/src/<module>.rs` and `pub mod` it in `lib.rs`.
2. Copy `docs/context/_template.md` to `docs/context/<crate>/<module>.md` and fill it in.
3. Add a row to the table above.

## Commands

- Native: `cargo run` · Server: `cargo run -p server` (with `HALL_OF_WORLDS_WEB=dist` after `trunk build`, it serves the web client too, on http://localhost:8080) · Web: `trunk serve` (http://localhost:3000; it has no game server: use the server's port to play) · Format: `cargo fmt`
- Release package: `cargo xtask package` (native client + server + `assets/` in `target/package/`; native releases must always include the assets).
- **Test: `cargo xtask test`**: formatting check, clippy with warnings as errors for native and for wasm (type-check only), then `cargo test --workspace`. Run it before finishing any change. CI no longer runs it (too much work per PR): CI runs only `cargo check --release` on Linux for ready-for-review PRs and `master`, so it must pass locally before merging.
- **Lint: `cargo xtask lint`**: the same without the tests. The pre-commit hook in `.githooks/` runs it (turn on once per clone with `cargo xtask hooks`). It checks the files on disk, not only what's staged. Fix what it reports rather than skipping it with `--no-verify`.

## Development workflow

- Docker isn't installed on the owner's machine, and the Docker workflow only builds the image after merging to `master`: take extra care with `Dockerfile` changes.
- **Develop and test on the native (Windows) build.** Trust Bevy to produce an equivalent web build: `cargo xtask test` type-checks wasm, and the owner tests in the browser occasionally. Don't make browser testing or web performance a task until the owner asks (later in development).
- **Leave the game running for the owner** after finishing a change: start the native build in the background (`./target/debug/hall-of-worlds.exe` with `run_in_background`) and don't close it, so the owner can see and play with the changes. Before rebuilding, close it if it's still running (`taskkill //IM hall-of-worlds.exe //F`): Windows locks the running executable and the build fails with "Access is denied".
- **Checking visuals:** run the native build with `dev_tools` environment variables to set the start position/state and save a screenshot, then read the image (see `docs/context/client/dev_tools.md`). Always set `HALL_OF_WORLDS_SERVER_DATA` to a scratch folder in the same command (shell variables don't carry between commands), or the run saves over the owner's character.
- Bevy compiles slowly the first time (15+ min clean); incremental builds take seconds. Run long builds in the background.

## Conventions

- Bevy version is pinned in `Cargo.toml` (currently 0.20). Bevy changes its API between versions: check the compiler errors and docs.rs for the pinned version rather than assuming older APIs.
- Client and shared code must compile for `wasm32-unknown-unknown` as well as native: no threads, blocking file IO or native-only crates without a `#[cfg(target_arch = "wasm32")]` alternative. Server code is native-only and may use them.
- Dependencies must be open source under an OSI-approved licence; MIT (or MIT/Apache-2.0 dual) preferred. When suggesting a library, give the licence and the reason, and offer alternatives where sensible: the owner cares about results, not specific libraries.
- Git: the owner uses branches and PRs on GitHub (`Rycochet/game`, default branch `master`). Don't push or open PRs unless asked.
