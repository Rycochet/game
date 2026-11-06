# Hall of Worlds: guide for Claude

**Hall of Worlds** is a multiplayer roguelike with an infinite, procedurally generated world, written in Rust with the Bevy engine. Client/server: the client builds natively (Windows/macOS/Linux) and to WebAssembly (browser); the server builds natively only. Design notes are in `README.md` and `designs.txt`.

## Core rules (always apply)

- **Don't trust the user.** The server is authoritative and validates every command against the same rules/data as the client. Clients send intents, never results. The client only predicts.
- **Determinism.** Anything both sides compute (world generation, validation) lives in shared code, uses only the project's seeded, position-hashed RNG, integer game state, and no platform-dependent float functions.
- Full details, including how to decide shared vs. client-only code: `docs/context/architecture.md`. Read it before touching game rules, generation, randomness or networking.

The project owner is new to Rust: explain Rust concepts briefly when they come up, and prefer clear code over clever code.

## Tech stack

Rust stable (pinned in `rust-toolchain.toml`), Bevy 0.20, Trunk for the web build, WebSockets between client and server, postcard for messages. Details: `docs/context/project.md`.

Crates: `crates/shared` (plain Rust, no Bevy: rules and generation), `crates/client` (Bevy app, binary `hall-of-worlds`; native builds embed the server for single player), `crates/server` (native only, binary `hall-of-worlds-server`: the networked server, also serving the web client), `crates/web_delta` (native only: delta downloads of the web client, for the server and xtask), `xtask` (helper commands).

## How to load context (read this first)

**Do not read the whole codebase.** Each feature module has a short context file in `docs/context/`. Read only the ones relevant to the task, then open only the source files they point to.

| Module              | Source                          | Context file                          | Covers                                              |
| ------------------- | ------------------------------- | ------------------------------------- | --------------------------------------------------- |
| (project)           | root files                      | `docs/context/project.md`             | Build, tooling, folder layout, CI                   |
| (design)            | n/a                             | `docs/context/architecture.md`        | Client/server split, trust, determinism, map data flow |
| (performance)       | n/a                             | `docs/context/optimisations.md`       | Measured optimisations tried but not adopted (memory) |
| (wiki style)        | n/a                             | `docs/context/wiki-style.md`          | How the players' wiki pages are laid out and written, as a checklist the owner signs off; read before writing any page in `docs/` |
| (ui design)         | n/a                             | `docs/context/ui-design.md`           | The palette of interface pieces: how each should look and behave here, and which we have |
| (debug api)         | `crates/server/src/debug_api/`, each module's `debug_api.rs` | `docs/context/debug-api.md` | Localhost JSON-RPC into a running client or server (`--features debug-api`, `HOW_DEBUG_API=true`, `cargo xtask api`): its methods and how to add one, temporary or for good |
| (postgres plan)     | n/a                             | `docs/context/postgres-migration.md`  | Not built: optional relational Postgres saves and a two-way migration tool, for when the data schema has matured |
| (abilities plan)    | n/a                             | `docs/context/abilities-skills.md`    | Nine attributes chosen with triangles (built), and not built yet: the skill tree and how practice, checks, specialising and rust work, abilities across branches; for players and mobs alike |
| shared: rng         | `crates/shared/src/rng.rs`      | `docs/context/shared/rng.md`          | Seeds, position hashing, random sequences           |
| shared: world       | `crates/shared/src/world.rs`    | `docs/context/shared/world.md`        | World settings from seed, tides                     |
| shared: celestial   | `crates/shared/src/celestial.rs` | `docs/context/shared/celestial.md`   | Suns and moons from the seed: colours and names, where they are, the light they give and the moons' pull on the tide at any time |
| shared: surface     | `crates/shared/src/surface.rs`  | `docs/context/shared/surface.md`      | Terrain height/jitter at any grid position, by generator, rolled per world (Desert, the old noise, for dry desert worlds; Continents in metres; Plates, Continents shaped by drifting plates) |
| shared: climate     | `crates/shared/src/climate.rs`  | `docs/context/shared/climate.md`      | Temperature, moisture and Whittaker biomes at any world map position; for surfaces in metres, from the planet's tilt and suns, with belts of rain and ice caps |
| shared: river       | `crates/shared/src/river.rs`    | `docs/context/shared/river.md`        | Rivers running downhill from a chunk's source, more where wet (visual only for now) |
| shared: road        | `crates/shared/src/road.rs`     | `docs/context/shared/road.md`         | Roads between towns within range (paths, tracks, stone roads, highways by the smaller town), routed by the walking rules; worked out by the server; steps along them are faster |
| shared: vegetation  | `crates/shared/src/vegetation.rs` | `docs/context/shared/vegetation.md` | Tree cover and low growth at any world map position, from climate and patch noise (visual only for now) |
| shared: weather     | `crates/shared/src/weather.rs`  | `docs/context/shared/weather.md`      | Cloud cover and rain at any place and time (4D simplex noise) |
| shared: world_gate | `crates/shared/src/world_gate.rs` | `docs/context/shared/world_gate.md` | World Gates: 16 ways onto every planet, on land with room round them and spaced apart, found in easing rounds from the terrain seed |
| shared: hall        | `crates/shared/src/hall/`       | `docs/context/shared/hall.md`         | The Hall of Worlds: a wrapping marble walkway of 2^33 tiles with gates every 4 on both sides; where each gate leads (planet seed and World Gate, both ways, shuffled per sixteenth), gate designs and rune codes |
| shared: wilderness | `crates/shared/src/wilderness.rs` | `docs/context/shared/wilderness.md` | The land between pois at 16 cells a world map point, made on demand in 64 × 64 blocks: ground, grass and trees by cover, sea with shallows near land, each chunk's poi set in as a cutout in a clearing |
| shared: poi     | `crates/shared/src/poi.rs`  | `docs/context/shared/poi.md`      | World map chunks and their pois (kind, position, map seed) |
| shared: showcase    | `crates/shared/src/showcase.rs` | `docs/context/shared/showcase.md`     | Debug plain far south with every kind of poi, 8 seeds and each level of ruin, behind `ENABLED` (P to go, Esc to return) |
| shared: dungeon     | `crates/shared/src/dungeon.rs`  | `docs/context/shared/dungeon.md`      | Cell grid (floor, wall, doors, windows, ground, covered ground, water, lava, roads, gardens, holes, fields, fences, herb beds, grass, trees), wall shapes, rooms, line of sight / light reach |
| shared: item        | `crates/shared/src/item.rs`, `id.rs` | `docs/context/shared/item.md`       | UUID v6 `Id`s, kinds of item, shapes and grid inventories with nested bags: the rules moves are checked by |
| shared: item_form   | `crates/shared/src/item_form.rs`, `material.rs` | `docs/context/shared/item.md` | What one item is physically like, rolled once and saved: forms, gem cuts, materials |
| shared: blade       | `crates/shared/src/blade/`      | `docs/context/shared/blade.md`        | Knives to polearms: size grids of what may be made, rolled properties, specific and generic names (research: `docs/context/research-bladed-weapons.md`) |
| shared: body        | `crates/shared/src/body.rs`, `body/` | `docs/context/shared/body.md`    | Any body's layout (tilting spine, limb slots with roles, head, parts) and its two-legged view, the frame worn things are built on: levels, widths and joints in millimetres, worked out from a kind's proportions (research: `docs/context/research-creatures.md`) |
| shared: armour      | `crates/shared/src/armour/`     | `docs/context/shared/armour.md`       | What all armour shares (constructions, styles) and pieces by location: hands (gauntlets, demi-gauntlets, padded gloves), shoulders (pauldrons, spaulders, sode, capes, discs, ailettes), arms (vambraces, couters, rerebraces, arm harness, bracers, sleeves, manicae, kote, bazubands) and torso (cuirasses, leather, mail, ring mail, scale, lamellar, laminar, brigandines, plated mail, mirrors, slats, rattan, padding, arming vest) and head (helmets of every kind, coifs, arming caps) and neck (collars, mail standards, gorgets, colletins, nodowa, bevors, aventails, mantles) and legs (cuisses, poleyns, greaves, leg harness, chausses, suneate, haidate, padded chausses) and waist (faulds, tassets, cuirassier tassets, tonlets, skirts, kusazuri, pteruges, war girdles, mail breeches) and feet (sabatons, armoured boots, mail shoes, kusari tabi, kogake, foot guards) so far; dyes and finishes; sets (armour coats, corslets, half and full armour) made of those pieces |
| shared: footwear    | `crates/shared/src/footwear/`   | `docs/context/shared/footwear.md`     | Shoes, boots, sandals, clogs and slippers (Middle on the feet): builds, heights, toes, heels, fastenings, tops, decoration, rolled from allowed grids, with names, cells and weights |
| shared: creature    | `crates/shared/src/creature.rs`, `wear.rs`, `sheet.rs` | `docs/context/shared/creature.md` | Kinds of creature and their bodies, sizes, the nine attributes (chosen with triangles, kind bonuses), skills, looks, what they can carry, layers of clothes and the rules for wearing them |
| shared: energy      | `crates/shared/src/energy.rs` | `docs/context/shared/energy.md` | Time for actions: energy by speed and ticks, worked out by formula; action costs |
| shared: ground      | `crates/shared/src/ground.rs`   | `docs/context/shared/ground.md`     | How full a bag on the ground looks, from its weight |
| shared: shop        | `crates/shared/src/shop.rs`     | `docs/context/shared/shop.md`       | What town shops sell, make times, biased rolls, and catching stock up on opening |
| shared: mob         | `crates/shared/src/mob.rs`    | `docs/context/shared/mob.md`        | Who is on a site level from its seed, and what a player learns of what is on the map |
| shared: movement    | `crates/shared/src/movement.rs` | `docs/context/shared/movement.md`     | One-cell steps and the rules both sides check them by |
| shared: pathfinding | `crates/shared/src/pathfinding.rs` | `docs/context/shared/pathfinding.md` | Quickest route of legal steps across one level or the world map (A*), spreading diagonals out |
| shared: protocol    | `crates/shared/src/protocol/`   | `docs/context/shared/protocol.md`     | Client/server messages (world map chunks, site levels in 8×8 chunks), in-process `Link` (postcard bytes), server addresses as typed |
| shared: site        | `crates/shared/src/site/`       | `docs/context/shared/site.md`         | Sites behind pois: building (BSP) with ruin and basements, cave (with skylights), town, farm, herbalist's hut and oasis generators, levels, stairs |
| web_delta           | `crates/web_delta/`             | `docs/context/web_delta/web_delta.md` | Delta downloads of the web client (`dcz`): a new wasm compressed against an old build a browser has; hashes and file names the server and xtask share |
| server              | `crates/server/`                | `docs/context/server/server.md`       | Authoritative server in its own Bevy world: WebSocket networking (and the web client's files) on one port, connections, world map chunks, site visits and level cells, wilderness visits (a window of blocks round the player), accounts and logging in, saves (world, sites, characters), embedded single player (which its player can open to others), standalone binary |
| client: connection  | `crates/client/src/connection/` | `docs/context/client/connection.md`   | Connecting to the server (embedded on native, WebSocket on the web), reconnecting, sending steps and putting refused ones right, `ToServer` / `FromServer` |
| client: common      | `crates/client/src/common/`     | `docs/context/client/common.md`       | Current world (and its pois) and dungeon, the site's other known floors and their numbering, `MapPosition` / `SmoothPosition`, `GameView` state (incl. `Connecting`), the window as left (place, size, maximised, monitor, full screen) and `DisplayInUse`, `ServerStatus`, key bindings (`Controls`, `Action`), `Portal` / `Clickable`, clock, weather, view stats, view facing, frame cap, `Token`, `LightSource`, `SharesSight` |
| client: player      | `crates/client/src/player/`     | `docs/context/client/player.md`       | Player position, keyboard steps, travel to clicks along found routes (announced as `Stepped`) |
| client: surface_style | `crates/client/src/surface_style/` | `docs/context/client/surface_style.md` | Planet colour bands and thresholds, biome colours |
| client: view3d      | `crates/client/src/view3d/`     | `docs/context/client/view3d.md`       | Shared 3D camera (turn, tilt, zoom), entity tokens, token clicks, `ViewGrid` |
| client: world_map   | `crates/client/src/world_map/`  | `docs/context/client/world_map.md`    | 3D terrain window cut to the camera, edge faces, sun, click picking, poi models, rivers, roads with bridges, clouds, moving water, precipitation |
| client: dungeon_map | `crates/client/src/dungeon_map/` | `docs/context/client/dungeon_map.md` | Dungeon view: towns (roads, gardens with hedges, buildings with roofs and door signs), farms (fields, crops by stage, fences, gates), herb patches growing by the clock, textured floors and thin 3D walls, doors and windows by theme, 3D stairs in seven styles, liquid channels, holes in the floor, sight, see-through walls, grid light, memory in grey, outside ground and precipitation following the world map |
| client: minimap     | `crates/client/src/minimap/`    | `docs/context/client/minimap.md`      | Round minimap and M full map: known land with poi pictures, or remembered cells as a grid; drag, title, arrow to the player, floor buttons |
| client: site        | `crates/client/src/site/`       | `docs/context/client/site.md`         | Asking the server to enter sites, take stairs, leave; applying its level messages |
| client: debug_menu  | `crates/client/src/debug_menu/`  | `docs/context/client/debug_menu.md`   | Backtick or warning button: full-screen filtered list of debug actions (toggles, Teleport Mode, World Time, Suns and Moons, View Planet, Stairs/Fields/Herbs panels, Show Widgets) |
| client: system_menu | `crates/client/src/system_menu/` | `docs/context/client/system_menu.md` | Esc or gear button: small menu in sections (Continue, Online, Connect...; Settings, Credits; Logout, Quit), the same as the startup menu with Play, Online with Server Options and its requesters, Connect (address, remembered servers, pasted certificates), refusals, quit requester and web ended page, Settings layer with tabs (Interface: size, Enable Touch; Video: display mode, resolution, Apply and keep requester), Alt+Enter, Credits scrolling by itself; asks a single-player server to pause while open |
| client: inventory   | `crates/client/src/inventory/` | `docs/context/client/inventory.md`  | Grids of what is carried and the clothes panel, in moveable panels; picking items up, holding, turning and dropping or wearing them |
| client: item_art    | `crates/client/src/item_art/`   | `docs/context/client/item_art.md`   | Pictures of items: drawn once in 3D off screen and kept as images; gem, ring, weapon, gauntlet, shoulder, arm, torso, helmet, neck, leg, waist, foot armour and footwear models (built in place on the body frame) |
| client: trade       | `crates/client/src/trade/`      | `docs/context/client/trade.md`      | Trading panels with someone's stock in a grid; for now a town shop's, opened by clicking its door, near or from afar |
| client: ground      | `crates/client/src/ground/`     | `docs/context/client/ground.md`     | Bags on the ground: 3D models by how full, sparkle, black outlines, remembered in grey; clicking one walks there and opens the Character panel |
| client: character   | `crates/client/src/character/` | `docs/context/client/character.md`  | Character panel: attributes, skills, abilities, looks and the most that can be carried |
| client: character_create | `crates/client/src/character_create/` | `docs/context/client/character_create.md` | Making a new character when the server asks: name, kind, looks, and the three turning triangles with two-layer radar charts (the kind's bonus inside) |
| client: entities    | `crates/client/src/entities/` | `docs/context/client/entities.md`   | Mobs and other players as tokens, as long as they can be seen |
| client: hall_view   | `crates/client/src/hall_view/`  | `docs/context/client/hall_view.md`    | The Hall of Worlds' gates: models by design with their rune codes (shown when zoomed in; also for world map gate pois), placed along the stretch, faded when in the player's way |
| client: touch       | `crates/client/src/touch/`      | `docs/context/client/touch.md`        | Fingers: tap as a left click, `PointerPosition`, `TouchGesture` (drag, pinch, twist, tilt) |
| client: widgets     | `crates/client/src/widgets/`    | `docs/context/client/widgets.md`      | Interface pieces: panels with titles, buttons by state, tooltips, moveable panels; taking clicks on them; interface size (`UiScale`); widget ids (`UiId`, `form/name`) and the `client.ui.*` debug methods |
| client: ui          | `crates/client/src/ui/`         | `docs/context/client/ui.md`           | HUD status text, click mode text                    |
| client: fonts       | `crates/client/src/fonts/`      | `docs/context/client/fonts.md`        | The default font for all text: Lexend, built into the program |
| client: planet_view | `crates/client/src/planet_view/` | `docs/context/client/planet_view.md` | View Planet: the whole world as a turning globe from the surface generator, a day a minute, with drifting clouds (shader) and textured, spinning moons orbiting it; relief from a normal map, clouds gathering on highlands; a bar picks the generator, seed (or a random one) and layer (surface, elevation, plates, temperature or moisture), with the World Gates ringed; any other click, tap or Esc closes |
| client: loading_screen | `crates/client/src/loading_screen/` | `docs/context/client/loading_screen.md` | Full-screen title and step bar until the game takes input, and while the connection is lost; server refusals; only the title behind the startup menu |
| client: dev_tools   | `crates/client/src/dev_tools/`  | `docs/context/client/dev_tools.md`    | Env-var helpers: start position or site, stairs style, travel, walk, generator, camera view, screenshots |

Also: `docs/context/decisions.md` is the log of architecture decisions. Check it before proposing a structural change, and follow what it records.

If a task touches a module not listed here, it doesn't exist yet: create it (see "Adding a module").

## Module rules

- **Client:** every feature is a folder `crates/client/src/<module>/` whose `mod.rs` exposes one Bevy `Plugin` (`<Name>Plugin`), registered in `crates/client/src/main.rs`, which contains no game logic.
- **Shared:** plain Rust modules (`crates/shared/src/<module>.rs`, or a folder when it grows) with no Bevy dependency so far. Anything the server will need to agree on goes here (see `architecture.md`).
- A module may use `common` freely. It should not reach into another feature module's internals. Cross-module communication goes through types the owning module deliberately makes `pub` (components, resources, messages/events), and each context file lists them under "Public API".
- If two modules need the same type, move it to `common` rather than having them depend on each other.
- Keep modules small enough that their context file stays short. Split a module (e.g. `world/terrain`, `world/render`) when it grows past what one context file can summarise.

## File size (owner's rule)

Small files, each about one thing, are easier to understand and to find things in, for the owner and for Claude alike.

- **One idea per file:** a type and its `impl`, one Bevy system with its small helpers, one generator step, one message handler. Name the file after it.
- **Aim for under about 100 lines; 300 is the limit**, not counting tests. `cargo xtask lint` (and so the pre-commit hook) enforces the limit.
- **Tests go in their own file:** `#[cfg(test)] mod tests;` in `foo.rs` (or `foo/mod.rs`) is read from `foo/tests.rs`. Every module's tests are kept this way, so context files don't list them. A second test module in the same place is named `<what>_tests.rs`, which the size check also leaves out.
- **`mod.rs` holds** the `mod` lines, the plugin and `pub use` re-exports, so other modules' paths don't change when a module is split into files.
- **Widen privacy only as far as a split needs** (`pub(super)`, `pub(crate)`), never to `pub` for convenience: the module's public API stays what its context file lists.
- **Files over the limit** are listed in `xtask/large-files.txt` and may only shrink. One with a `# reason` (such as a long data table) is large on purpose; one without is waiting to be split. After a change shrinks listed files, run `cargo xtask sizes` to lower or remove their entries. Never add an entry, or a reason, without the owner's agreement: split the file instead.

## Keeping context files up to date (required)

Whenever a change affects a module, update its `docs/context/<crate>/<module>.md` **in the same commit / branch / PR** as the code. Update it when you change:

- the module's files (added, removed, renamed),
- its public API (anything `pub` other modules use),
- which modules it depends on, or which depend on it,
- a non-obvious behaviour, gotcha or invariant.

Context files are summaries, not documentation: a few lines per item, enough to decide which file to open. Don't paste code into them. Also update the table above and `docs/context/project.md` when modules or tooling change, and add an entry to `decisions.md` when a structural decision is made (including decisions the owner makes in conversation).

## User documentation

`docs/` (except `docs/context/`) is the players' and server hosts' site, https://hallofworlds.wiki, in Markdown, published by Jekyll when it reaches `master`. Keep it plain and free of screenshots and fine detail while the game is changing quickly.

Update these **in the same PR** as the change:

| When this changes | Update |
| ------------------------------------------------------------- | ------------------------------------------------- |
| A server environment variable (added, removed, default, meaning), or a native client's connection variable | `docs/environment-variables.md`, checked by a test for server variables |
| A debug menu entry (added, removed, renamed, what it does, whether it needs the server) | `docs/debug-menu.md`, checked by a test for names |
| How a server is run or secured, its ports, the Docker image | `docs/hosting/` (the page and `docker-compose.yml`) |
| When the game pauses, or who may pause it | `docs/client.md`, `docs/controls.md`, `docs/hosting/index.md` |
| Where saves or other files are kept | `docs/saves.md` |
| Third-party art or fonts | `docs/credits.md` |
| Kinds of armour, their names, styles, materials or rough weights | `docs/armour/` |
| Kinds of footwear, their names, shapes, materials or rough weights | `docs/footwear.md` |

Every page keeps to the checklist in `docs/context/wiki-style.md`.

Everything else (the introduction, installation, the client's description, controls) is updated by hand when the owner asks, or when a change would make a page plainly wrong. Never put the owner's own server addresses in these pages.

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

- Native: `cargo run` · Server: `cargo run -p server` (with `HOW_WEB=dist` after `trunk build`, it serves the web client too, on https://localhost:8081/web/ with a self-signed certificate, and also on http://localhost:8080/web/ with `HOW_BEHIND_PROXY=true`) · Web: `trunk serve` (http://localhost:3000/web/; it has no game server: use the server's port to play) · Format: `cargo fmt`
- Debug API: build with `--features debug-api` (debug builds only; releases refuse it) and start with `HOW_DEBUG_API=true`, then `cargo xtask api <method> [key=value ...]` (`--server` for the embedded server); `rpc.discover` lists the methods. It reads and sets widgets by id (`client.ui.list`, `client.ui.set`, `client.ui.press`: prefer these), and can also click, drag, tap, scroll, press keys and type (`client.click`, `client.key`, ...), for testing the interface. See `docs/context/debug-api.md`.
- Release package: `cargo xtask package` (native client + server + `assets/` in `package/` in cargo's target folder; native releases must always include the assets).
- **Test: `cargo xtask test`**: formatting check, clippy with warnings as errors for native (with and without the debug API) and for wasm (type-check only), then `cargo test --workspace` with the debug API, then the file size check (see "File size"), then the WebGL 2 check (uniforms a multiple of 16 bytes, `Srgb` render targets, no storage buffers or compute in shaders: things the web build quits on while native works), then a check that no `tmp.` debug API methods are left. Run it before finishing any change. CI no longer runs it (too much work per PR): CI runs only `cargo check --release` on Linux for ready-for-review PRs and `master`, so it must pass locally before merging.
- **Lint: `cargo xtask lint`**: the same without the tests and the `tmp.` check. `cargo xtask sizes` lowers the entries in `xtask/large-files.txt` after files shrink. The pre-commit hook in `.githooks/` runs it (turn on once per clone with `cargo xtask hooks`). It checks the files on disk, not only what's staged. Fix what it reports rather than skipping it with `--no-verify`.

## Development workflow

- Docker isn't installed on the owner's machine, and the Docker workflow only builds the image after merging to `master` (or when run by hand on a branch): take extra care with `Dockerfile` changes.
- **Two Dockerfiles make the same image:** `Dockerfile` builds everything inside Docker (for building it yourself), and `Dockerfile.github` copies in what `.github/workflows/docker.yml` compiled on the runner. Any change to one must check whether the other needs it: the part after their `Same in` marker line must stay identical (`cargo xtask lint` checks it), and build steps (Trunk's version, build and compress commands) live in `Dockerfile` and the workflow, and usually in `build.yml` too.
- **Develop and test on the native (Windows) build.** Trust Bevy to produce an equivalent web build: `cargo xtask test` type-checks wasm, and the owner tests in the browser occasionally. Don't make browser testing or web performance a task until the owner asks (later in development).
- **Leave the latest build for the owner** after finishing a change: run `cargo xtask latest`, and start nothing. It builds the client with the debug API and copies it, with the assets, into `latest/` in cargo's target folder as `hall-of-worlds-latest`, with `play.cmd` to start it (debug API on port 15712, away from development copies'), for the owner to start when they choose. If the owner's copy is running, the copy fails: leave it and tell the owner a newer build is waiting.
- **Development copies:** every game Claude starts sets `HOW_DEBUG_TITLE=Dev:` in the same command, so its window reads "Dev: Hall of Worlds". `<target>/debug/hall-of-worlds.exe` is the build (`<target>` is cargo's target folder: `target/` unless a cargo setting moves it, as `cargo metadata` reports). Before rebuilding, close any still running (`taskkill //IM hall-of-worlds.exe //F`, which leaves the owner's `hall-of-worlds-latest.exe` alone): Windows locks the running executable and the build fails with "Access is denied".
- **Checking visuals:** run the native build with `dev_tools` environment variables to set the start position/state and save a screenshot (any of them skips the startup menu and character creation; `HOW_DEBUG_SKIP_MENU=false` and `HOW_DEBUG_SKIP_CREATION=false` keep them), then read the image (see `docs/context/client/dev_tools.md`); with `HOW_DEBUG_TITLE=Dev:` as above. With the debug API, one running copy can take many: `cargo xtask api client.screenshot path=target/shots/a.png` (see `docs/context/debug-api.md`). Add debug API methods (`tmp.` ones are fine on a branch) when they would make a task easier to check. Always set `HOW_DATA` to a scratch folder in the same command (shell variables don't carry between commands), or the run saves over the owner's character.
- Bevy compiles slowly the first time (15+ min clean); incremental builds take seconds. Run long builds in the background.

## Conventions

- Bevy version is pinned in `Cargo.toml` (currently 0.20). Bevy changes its API between versions: check the compiler errors and docs.rs for the pinned version rather than assuming older APIs.
- Client and shared code must compile for `wasm32-unknown-unknown` as well as native: no threads, blocking file IO or native-only crates without a `#[cfg(target_arch = "wasm32")]` alternative. Server code is native-only and may use them.
- Dependencies must be open source under an OSI-approved licence; MIT (or MIT/Apache-2.0 dual) preferred. When suggesting a library, give the licence and the reason, and offer alternatives where sensible: the owner cares about results, not specific libraries.
- Environment variables: all start `HOW_`; ones only for debugging and development start `HOW_DEBUG_` (those a real deployment can ignore). No `HOW_CLIENT_` / `HOW_SERVER_` prefixes: the native client embeds the server, so the split would not hold. Logging uses the usual `LOG_LEVEL` (and `RUST_LOG` for per-crate filters), so put noisy lines at `debug` or `trace` rather than behind a flag.
- Boolean options: read them with `server::option_on`. Off by default, `1`, `on`, `true` or `yes` turn them on; on by default, `0`, `off`, `no` or `false` turn them off; any case. Documentation and examples always use `true` / `false`.
- Git: the owner uses branches and PRs on GitHub (`Rycochet/game`, default branch `master`). Don't push or open PRs unless asked.
