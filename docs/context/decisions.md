# Decisions

Architecture and process decisions, newest last. Each entry: date, decision, why. Add one whenever a structural choice is made, including ones the owner makes in conversation. Don't reverse a decision without the owner agreeing.

This file holds only what is currently true; old entries live on in git history. Maintenance happens **only when a branch is ready to merge** (not on every commit):

- Editing an entry to keep it accurate (e.g. "planned" → done, referring to a commit or PR instead of a branch, merging a superseded entry into its replacement) needs no confirmation.
- **Removing** an entry, or removing a substantive part of one, must be confirmed by the owner first: list what you propose to remove and why, so they can check the diff.

## 2026-10-27: Rewrite in Rust with Bevy

Replaces the earlier TypeScript/React prototype (still on `master` history). Rust compiles to native apps on all desktop OSes and to WebAssembly. Bevy chosen as the engine: actively developed, supports native + web from one codebase, and its ECS/plugin model suits a modular codebase and a large simulated world.

## 2026-10-27: One Bevy plugin per feature module

Each client feature lives in `crates/client/src/<module>/` and exposes a single `Plugin`; `main.rs` only wires plugins together. Modules interact only through deliberately `pub` components, resources and messages, with shared types in `common`. Goal: work on one area (e.g. map drawing) without needing other areas (e.g. characters) in context.

## 2026-10-27: Context files for Claude

`CLAUDE.md` is the index; `docs/context/<crate>/<module>.md` summarises each module (files, public API, dependencies, notes). They are updated in the same change as the code they describe, so Claude can load only the relevant parts instead of the whole codebase.

## 2026-10-27: Web build with Trunk

Trunk builds and serves the wasm version (`trunk serve`, port 3000) using `index.html`. Simplest standard tool for Bevy on the web.

## 2026-10-27: Client/server with an authoritative server

"Don't trust the user." The server owns game state and validates every client command using the same rules and data as the client; clients send intents and only predict results. Both client and server must build for Windows, macOS and Linux; the client also for the browser, the server never. Details: `architecture.md`.

## 2026-10-27: Cargo workspace `shared` / `client` / `server`

Rules, world generation, RNG, validation and protocol types go in `shared`, used by both sides. Client and server are both Bevy apps (server headless) so shared ECS systems run in both. Whether visual-only features live in `shared` or `client` is decided per feature and recorded in its context file. `crates/shared` and `crates/client` exist; `shared` is plain Rust with no Bevy dependency until shared ECS systems are needed. `crates/server` is added with networking.

## 2026-10-27: Deterministic generation

All shared randomness is seeded and derived by hashing (seed, purpose, coordinates) so results are independent of generation order. Authoritative state uses integers (grid positions). Shared code avoids platform-dependent float functions (or uses `libm`). Golden-value tests run natively via `cargo xtask test` (CI ran them on all three OSes until it was cut down to `cargo check`; running them under wasm too is still to add). Rules: `architecture.md`.

## 2026-10-27: Network message encoding: `postcard` + `serde`

`postcard` (MIT OR Apache-2.0): compact binary format with a documented, stable wire format, works on wasm. Preferred over `bincode`, whose development stopped in 2025.

## 2026-10-27: Noise library: `fastnoise-lite`

Chosen by the owner. The old prototype used Perlin noise via `simplex-noise`; the results matter, not the library. Uses OpenSimplex2 with f64 coordinates; internally only `sqrt`/`floor`, so it's deterministic across platforms; golden tests guard that.

- **`fastnoise-lite`** (MIT). Single-file port of FastNoise Lite, which also exists for C#, JS and GLSL, so a client-only shader could reproduce the same noise for visuals. Offers OpenSimplex2 (fewer grid-aligned artefacts than classic Perlin), Perlin, cellular/Voronoi (useful for biomes/regions), fractal layering and domain warping built in.

## 2026-10-27: Random number generation: own SplitMix64 in `shared::rng`

Chosen by the owner. A small hand-written hash turns (seed, purpose, coordinates) into a value, and the same SplitMix64 generator provides sequences, so there's no dependency.

- `rand_pcg` (MIT OR Apache-2.0) can be added if a sequence ever needs better statistical quality; PCG's output is documented as reproducible across platforms and versions.
- Never `rand::rngs::StdRng`, `thread_rng`, or `std`'s `DefaultHasher`: their output isn't guaranteed stable.

## Open: networking library (decide with a small prototype)

The browser can only use WebSockets or WebTransport (no raw UDP), so the transport must support those as well as native.

- **Recommended: `bevy_replicon` + `bevy_replicon_renet2`** (MIT OR Apache-2.0). Server-authoritative replication: server state is copied to clients automatically, clients send events (our intents). `renet2` adds WebTransport/WebSocket transports for the browser. Fits a grid roguelike with discrete commands.
- Alternative: `lightyear` (MIT OR Apache-2.0). Includes client prediction, rollback and interpolation; more powerful, more complex. Worth it if movement becomes real-time and fast-paced.
- Alternative: hand-rolled WebSockets (`tokio-tungstenite`, MIT, on the server; the browser's WebSocket on the client) with `postcard` messages. Full control and simple to understand, but we'd write reconnection, replication and interest management ourselves.

## 2026-10-27: Test command `cargo xtask test`

One command, identical locally and in CI: formatting check, clippy with warnings as errors for native and wasm (type-check only), then `cargo test --workspace`. Implemented as an `xtask` helper crate (the usual Rust pattern) because cargo aliases can't chain commands and scripts would differ between Windows and Unix. It must pass locally before merging; CI no longer runs it (see "CI cut down to `cargo check`").

## 2026-10-27: Develop on native; browser checked occasionally

Day-to-day building, running and testing use the native Windows build: it rebuilds about 3× faster than the web build, and Bevy produces an equivalent web client. `cargo xtask test` type-checks wasm. Browser testing and web performance are not tasks until later in development; avoid premature optimisation, but keep tracking performance numbers.

## 2026-10-27: World map style: Legend (1992) as the reference, modern look

The world map is inspired by Legend (1992): a square slab of terrain seen from the south in perspective on black, with an earth cut-away at the front and clear colour bands (water, beach, grass, hills, snowy mountains). It should be recognisable to people who know that style, but look modern rather than retro.

## 2026-10-28: The world map is a 3D renderer

`world_map` renders `shared::surface` as a lit 3D mesh on the GPU with the prototype's composition: a 400×400 window around the player with flat earth faces at its edges (it turns and zooms with the camera, see the camera entry below), and the camera aimed at sea level so it doesn't bob. A narrow field of view from far away gives flat, near-isometric perspective; the prototype's strong slope shading is baked into the vertex colours under a harsh sun, keeping the old look's surface variety; water is drawn as part of the terrain at the water surface; tonemapping is off so the original palette isn't washed out. A faithful port of the prototype's CPU slice renderer served as the visual reference and performance baseline while this was tuned, then was removed at the owner's request (it is in the history of Rycochet/game#2).

## 2026-10-27: Frame rate capped at ~30 fps

Not a twitch game: the client updates about 30 times a second while focused (input still wakes it immediately) and once a second unfocused, to save GPU and power.

## 2026-10-27: Native dev tools for visual checks

`dev_tools` (native only, inert by default) lets environment variables set the starting position and state and save a screenshot, then quit, so changes can be checked visually without a person watching.

## 2026-10-27: Temperature demo and shared colour formula

The prototype's snow-line animation is back as a demo (T key; G toggles auto-walk), driven by a `Weather::snow_line` resource. Colours come from one function, `SurfaceStyle::shade`, ported from the prototype. At the owner's request the beaches are about half as wide (`shore_jitter` 0.5) and sand is a brighter, fixed yellow.

## 2026-10-28: Click to travel, smooth positions

Left-clicking the 3D map sends the player travelling to the clicked grid point at constant speed (`player::TravelTo`). The position is fractional while moving (`SmoothPosition`) but always ends on a whole grid point (`MapPosition`); a new click re-targets from the current position. Clicks are found by walking the camera ray against the drawn terrain heights rather than using a physics or picking library. The player is shown as a flat white cylinder with a black outline (now a token, see "Entity tokens"), made with the inverted-hull technique to avoid a custom shader.

## 2026-10-28: The game is called Hall of Worlds

Domains (not live yet): `hallofworlds.com` landing page / adverts / email, `hallofworlds.app` the owner's server running the Docker image, `hallofworlds.wiki` game documentation (to be part of this repo later). Binaries are `hall-of-worlds` (client) and `hall-of-worlds-server`.

## 2026-10-28: Server crate; native clients embed it, the web client never does

`crates/server` is the authoritative server: a library (`ServerPlugin`) plus a standalone headless binary. Native clients link it for single player; the web client depends on it only through a non-wasm target dependency, so browser builds contain no server code (which will include filesystem access) and only connect to a server. The server uses Bevy without default features so it stays headless.

## 2026-10-28: Docker image published to GHCR on every merge

A multi-stage `Dockerfile` builds an x86-64 image (ARM can be added under the same tags later) containing the game server and the web client, served on port 8080 by static-web-server (MIT OR Apache-2.0, a small Rust static file server, run alongside the game server by `docker/entrypoint.sh`). The Docker workflow builds and publishes `ghcr.io/rycochet/hall-of-worlds:latest` on every push to `master` (no PR builds). The Build workflow, run manually, keeps native (client + server, three OSes) and web release builds as artifacts.

## 2026-10-28: Licence: CC BY-NC 4.0

The project is licensed under Creative Commons Attribution-NonCommercial 4.0 (`LICENSE`, SPDX `CC-BY-NC-4.0`), chosen by the owner: free to share and adapt with attribution, not for commercial use. It is not an OSI open-source licence (non-commercial clause), and Creative Commons doesn't recommend its licences for software; the owner accepted this. Dependencies must still be OSI-licensed (see `CLAUDE.md`).

## 2026-10-28: CI runs only on ready PRs and master

To avoid running CI on every push, the Check workflow runs on pull requests that are ready for review, skipping drafts, plus pushes to `master`. Iterate on a draft PR, then mark it ready to test. GitHub's merge queue would run checks only at merge time, but it isn't available to personally owned repositories.

## 2026-10-28: Client memory optimisations measured but not adopted

Trimming Bevy features and a few render/thread settings cut memory by about 25% in Task Manager terms, but the owner judged it not worth the upkeep for now. Measurements, versions and the exact changes are in `optimisations.md`.

## 2026-10-28: Entity tokens

Every entity shown on the map is a "token": a base shape (circle, triangle, square, hexagon, octagon; polygons show a flat side to the camera except the triangle) extruded upwards, coloured by its attitude to the player (white none, green friendly, blue neutral, red unfriendly), with the inverted-hull black outline. Tokens can also be "completed" (probably mostly for white ones; to be judged in play): about 30% darker and slightly see-through (opacity 0.9). This replaces the player-only marker; the player is a white circle (cylinder). The `Token` component lives in client `common` so any module can give an entity one, and `view3d` (originally `world_map`) decides how to draw it, on separate view entities so game entities carry no rendering components. Each token has an anchor above it that always faces the camera, showing the entity's 2D `Icon`: pixel art sized relative to the token (it scales with the map, for a blockier retro look; may change with new artwork), while the outline stays a fixed width in screen pixels. `Token` is client-only for now; attitude may move to `shared` once the server decides it.

## 2026-10-28: Third-party art: David E. Gervais tiles, credited in `ATTRIBUTION.md`

Entity icons come from David E. Gervais's tiles library (<http://pousse.rapiere.free.fr/tome/>), licensed CC BY 3.0, chosen by the owner. They are kept as individual files (easier to add and edit; a spritesheet can be packed at build time or by Bevy later if the web build needs fewer downloads) in `crates/client/assets/`, in the same folders as the original zips, with lower-case names and the magenta background turned into real transparency. Every third-party asset is credited in `crates/client/assets/ATTRIBUTION.md`, which ships with the web build. The CC BY-NC project licence and the dependency rule (OSI licences) are about code; art may use other licences that allow this use, recorded there.

## 2026-10-28: Native releases ship the assets folder

All assets must be in the native builds. Release executables load `assets/` from their own folder, so `cargo xtask package` assembles `target/package/` (client, server, `assets/`) and the Build workflow uploads that. Assets stay files beside the executable rather than embedded in it, so art can be replaced (the Gervais tiles are planned to be replaced by commissioned artwork) without rebuilding, and the executable stays small. Native debug builds read the source `crates/client/assets/` directly.

## 2026-10-28: World map camera: turn, tilt and zoom around the player

Chosen by the owner: right-drag turns (sideways) and tilts (up and down) the world map view around the player, the mouse wheel zooms, the view stays where it is left, and Home returns to the starting view (turn, tilt and zoom). The camera is a reusable `OrbitCamera` (now in `view3d`, shared with the dungeon view). Zoom can't go further out than the starting view, nor further in than 0.6 of its distance (owner's limits); tilt is 10°–85° (adjustable). Zooming changes what is in the frame, not the frame's size: the window shrinks as the camera moves in. As the camera tilts towards straight down, token icons slide down, half the way from standing on the token's centre to standing a couple of pixels above its near edge (the full slide looked too much).

Also the owner's choice: the map's outline stays the same on screen and the view into the world turns (and zooms), rather than the square slab rotating. The terrain window is a square lined up with the camera, cut out of the world-aligned terrain on the CPU (triangles crossing its border are clipped, so edges are exact; no custom shader), with earth faces down its edges. Per-point heights, normals and colours are kept in a wrapping store keyed by world position, so stepping and turning only work out newly covered points. The sun and the baked slope shading stay fixed to the world (sun in the east), so lighting is consistent rather than identical from every side. Arrow keys / WASD follow the camera (owner's choice): each steps in the grid direction nearest to its screen direction, through `common::ViewFacing`, so steps stay on the grid.

## 2026-10-28: CI cut down to `cargo check`

The repo is public with auto-merge enabled, and the full CI (`cargo xtask test` on three OSes, Docker test builds on PRs, release builds of every merge) was too much work. For now, the owner's call: PRs and `master` run only `cargo check --release --workspace` on Linux (still reported as the required "All tests passed" check, so auto-merge keeps working); the Docker image is built and published only on merge to `master`; the Build workflow is manual. `cargo xtask test` stays the local must-pass test before merging. Restore the full test workflow later (from git history).

## 2026-10-28: Dungeon view, sharing the camera and tokens with the world map

Asked for by the owner: a second view for dungeons and buildings, a grid of floor and wall cells (about the size of the player's picture) in 3D, with the world map's camera controls and tokens. Walls block movement, sight and light; floors block nothing. Only what the player (or anything sharing its sight, `common::SharesSight`) can see is drawn, and of walls only the sides someone sees (at most two per viewer, never the top). A seen side facing the camera's position is solid, one facing away (the wall is between the camera and what is seen) is 50% see-through, and anything out of sight isn't drawn. Left-clicking a seen floor travels there in a straight line, as on the world map, until pathfinding exists. Light: dull everywhere, plus light sources (`common::LightSource`; the player gives 6 cells of white light), blocked by walls. Floors dark grey and walls medium brown with black borders until tile pictures replace them. The game starts in a 15 × 7 test room with a 1 × 3 wall two steps west of the player (temporary).

Structure: the camera and tokens moved out of `world_map` into a new `view3d` module both views use; each view draws its own scene and tells `view3d` its cell size and floor height (`ViewGrid`). Which view is on screen is a Bevy state, `common::GameView`; each view hides (not despawns) its scene when not current. The grid rules (cells, walkability, line of sight and light reach, by symmetric shadowcasting in exact integers) are in `shared::dungeon`, since the server must agree on them. How bright the light looks is worked out on the grid in the client and baked into vertex colours, so tiles light exactly by the grid rule and coloured lights add up; tokens get real point lights instead. M switches views (demo); each view keeps its own player position.
