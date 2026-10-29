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

Asked for by the owner: a second view for dungeons and buildings, a grid of floor and wall cells (about the size of the player's picture) in 3D, with the world map's camera controls and tokens. Walls block movement, sight and light; floors block nothing. Only what the player (or anything sharing its sight, `common::SharesSight`) can see (or glimpse: see the AngbandTk entry) is drawn, and of walls only the sides someone sees (at most two per viewer, never the top). A seen side facing the camera's position is solid, one facing away (the wall is between the camera and what is seen) is 50% see-through, and anything out of sight isn't drawn. Left-clicking a seen floor travels there in a straight line, as on the world map, until pathfinding exists. Light: dull everywhere, plus light sources (`common::LightSource`; the player gives 6 cells of white light), blocked by walls. Surfaces are drawn with tile pictures (see the AngbandTk entry). The game started in a 15 × 7 test room while the view was built; since sites can be entered (see "Site generators") it starts on the world map again.

Structure: the camera and tokens moved out of `world_map` into a new `view3d` module both views use; each view draws its own scene and tells `view3d` its cell size and floor height (`ViewGrid`). Which view is on screen is a Bevy state, `common::GameView`; each view hides (not despawns) its scene when not current. The grid rules (cells, walkability, line of sight and light reach, by symmetric shadowcasting in exact integers) are in `shared::dungeon`, since the server must agree on them. How bright the light looks is worked out on the grid in the client and baked into vertex colours, so tiles light exactly by the grid rule and coloured lights add up; tokens get real point lights instead. Views are switched by entering and leaving sites (the M demo switch was removed then).

## 2026-10-28: World map features, one per chunk, chosen from the chunk's position

Asked for by the owner, as the first step towards dungeon generation: the game will have many kinds of maps (single buildings, towns and cities, dungeons and caves), so several generators. What is at a place, and the seed its map is generated from, both come from its world map position. The world grid is cut into chunks (`CHUNK_SIZE` 64 points, chosen so a 400×400 window shows a few dozen); each chunk has at most one feature (80% chance, the owner's figure), and the chunk's position alone decides whether it has one, its kind, its exact position in the chunk (at least `EDGE_MARGIN` 8 from the edges, so tokens never overlap) and its map seed (`shared::feature`, since the server must agree). Every feature is an entity with a white octagonal token (owner's choice) and a picture: the 34 kinds are the Dungeon Odyssey terrain tiles `l2_terrain066`–`099` (David E. Gervais, CC BY 3.0, the owner's selection), stored like the monster tiles. The client spawns entities only for features near the player while the world map is current, and the world map hides tokens outside its window. `MapPosition` and `SmoothPosition` moved from `player` to `common`, since features have positions too. Features stand only on dry land, above high tide (owner's choice): up to 8 spots in the chunk are tried, and a chunk without dry land among them has none, so the 80% applies to chunks with land.

Follow-up (owner): every `Token` has a `size` (`TokenSize`: tiny 0.5, small 0.75, default 1, large 1.25, giant 1.5, titan 2 × the original icon size) that scales only its icon; bases are always the same size. Features are tiny (their 54×54 pictures fill the tile; half size looks right, especially the castles), the player default (it looked right in the dungeon at the original size). Icons are now always drawn in front of their own token (a long-standing bug that cut their lower part off, noticed once many were on screen).

## 2026-10-28: RogueBasin as the roguelike development reference

The owner's pointer: <https://www.roguebasin.com/index.php?title=Articles> collects articles on every part of roguelike development (map and dungeon generation, field of view, pathfinding, AI, items...). Check it when starting a new area. The site generators follow its "Basic BSP Dungeon generation", "Cellular Automata Method for Generating Random Cave-Like Levels" and "Random Walk Cave Generation" articles.

## 2026-10-28: Site generators: buildings (BSP) and caves

Asked for by the owner: every world map feature is the entrance to a site of one or more levels, generated deterministically in `shared::site` from the feature's map seed; the feature's kind picks the generator, through wrappers giving each kind its own options (`Generator::for_feature`). Only shapes for now: walls, doors (rarely secret), floors, plus outside ground, water and lava; mobs, items and decorations later. Every walkable cell must be reachable from where you arrive (tested for every kind). Cells planned as part of a room record which room (`dungeon::RoomId`), for mobs later.

- **Buildings** (BSP, the owner's choice): an outline (rectangle or square, corners cut at 45° or rounded, circle; more primitives later) filled by splitting with 1-cell walls or corridors; each final part is a room filling it to its walls (the owner's "rooms expand to fill the space"), so no wall is thicker than 3; corner rooms follow cut corners. A 3-cell ground border outside, with at least one door in (south-facing if possible), and more as buildings get bigger (owner's follow-up); the inside is only joined up to what those doors reach, so a side door may lead to a part of the building the main door doesn't. Bigger buildings have more floors; each floor up keeps the outline and is, as the owner put it, the same size (50%) or smaller by a cell per 10% of the roll (taken as 1–5 cells on every side, so walls stand on the floor below). Stairs line up between floors.
- **Caves** (also dug dungeons such as crypts, with doors): irregular caves (no rectangles) joined by winding passages; each level's size is its own. The way in is a hole (stairs down from the world map) or a cave mouth: a ground border on the south side (the world map's lower side) and sometimes the east or west, as wide as the mouth rather than the cave. Water and lava pools may block the way only where there is a way round.
- Doors are open (flat, walkable, see-through) until opening and closing come with mobs; secret doors act as wall and are only placed where nothing depends on them.
- Client (`site` module): clicking a feature's token travels there and enters its site; clicking stairs travels there and takes them; clicking outside ground on the very edge of the level travels there and leaves for the world map (owner's follow-up: ground with walls or ground all round doesn't count, so the player can walk round a building to another of its doors). The game starts on the world map. Travel in sites is still a straight line until pathfinding.

## 2026-10-29: Dungeon walls and floors from the AngbandTk tiles; stairs in 3D

The owner's choice: David E. Gervais's flat AngbandTk tiles (same source and licence as the other art, CC BY 3.0, credited in `ATTRIBUTION.md`), whose `dg_features32`, `dg_grounds32` and `dg_extra132` sheets have wall and floor textures. The sheets are split into one PNG per tile in `assets/angbandtk/` (the owner's suggestion), and the dungeon view draws each kind of surface with its own picture: walls and floors themed by the kind of place (stone, wood, crypt, cave, ice; a client-only, visual choice in `dungeon_map::theme`), doors, outside ground, water and lava the same everywhere. Light stays baked into vertex colours.

The stairs tokens (Gervais `l2_stairsiron…` pictures) didn't read well (owner), so stairs are now part of the level's 3D drawing, half a cell wide down the middle of their cell with floor round them (owner's follow-up): stairs up are a short flight of masonry steps, stairs down a hole in the floor with steps going down, darker the deeper. Both ends of a staircase run the same way (owner: if the top is in the west going down, the bottom is in the west going up), worked out from both levels (by the server since step 3 of the map data plan, `Site::stairs_descend`). Stairs are cells of the level rather than entities: arriving on one by a click takes it.

Follow-up (owner): The view draws what a viewer glimpses (`Sight::add_glimpses`: open cells partly in view count), not just the symmetric sight, so side openings off a long corridor show instead of leaving black gaps; game rules keep symmetric sight.

## 2026-10-29: Map data from the server: pushed chunks, three states, a secret seed

Designed with the owner before splitting generation (server) from display (client); details and build order in `architecture.md` ("Map data"). The client generates only what isn't secret: world map terrain, from a terrain seed derived one way from the secret world seed. Features and sites come only from server messages, as what the player would see (secret doors arrive as wall). Every map is the generated base plus persistent server changes (versioned per chunk); on the world map the server sends the changes to the public base plus the secrets now visible to that player, including per-player feature state (visited, attitude). Site levels go in 8×8 chunks with a `u64` mask of the cells included. The server pushes what each step makes visible, plus what is visible from within K steps (K=1 to start), so the client can work out sight itself; the client keeps loaded / visible / seen apart and draws remembered cells in greyscale, still clickable to travel back. The server keeps each character's seen masks, so knowledge follows the player to another machine. A world's id is derived from its seed, so the same world is the same on every server (the owner wants worlds shareable between players); it will key a client cache. Single player: the embedded server runs in its own Bevy world and talks to the client through an in-process channel with the same messages, which will also let a single-player game be opened to others. The game is open source, so by default nothing stops a hacked client reading what it was sent (the server stays authoritative over the rules); stricter anti-cheat is planned as a server owner's option.

Follow-up (owner): server data and client data (cache, settings) are always in separate folders and never overlap, also with the embedded server. Every map generator, the world map included, has its own version, a plain incrementing number (semver stays for the apps), bumped only when its output changes in a way that matters, not for cosmetic changes (tests to enforce that later). The server stores the version each map was generated with; already generated maps keep their old version, and only new ones use the new generator. Every version still in use stays runnable in `shared`: the world map is infinite and generated by the client too, and site levels are regenerated rather than stored whole, because their generators also spawn mobs and items, during play and when a site resets after nobody has been in it for a while. Persistence matters more than resets (owner: the world is big, with no starting area to farm): changes that aren't the generator's (mining through walls, breaking doors) are stored as an overlay on the generated base, and a reset brings back only the generator's spawns, never undoing them.

## 2026-10-29: Server in its own world, talking through an in-process link

Step 1 of the map data plan. Native clients start the server on its own thread as a separate Bevy `App` (`server::start_embedded`), and the two talk only through a `shared::protocol::Link`: a pair of in-process channels carrying postcard-encoded messages, so nothing can be shared by accident and a network transport can later carry the same bytes. The client starts in a new `GameView::Connecting` state and gets its world from the server's `Welcome` (world id, terrain seed, world map version); starting states from `dev_tools` wait for it. The world seed is now secret: terrain comes from a seed derived from it, so the fixed world (12345) looks different from before. The web client has no server to talk to until networking, so it shows "Connecting to the server..." (owner's choice, over a temporary offline stand-in). Step 2 (below) removed the features seed from `Welcome`.

## 2026-10-29: World map features sent by the server

Step 2 of the map data plan. The client no longer has the features seed: the server sends each 64-point chunk within sight (300 points each way) plus one chunk more, once per connection, as a `WorldChunk` with its feature or none. The client keeps them in `common::KnownFeatures` and spawns feature tokens from that. (The client first reported its position for this; movement now goes through the server as steps, below.)

## 2026-10-29: Site levels sent by the server, remembered cells drawn in grey

Step 3 of the map data plan. The server runs visits to sites: the client asks to enter a feature (`EnterSite`, only from within 2 points of it), moves about the level (now by steps, below) and asks to use the stairs or edge ground it stands on (`UseExit`); the server answers with the level (`EnterLevel`: kind, floor, bounds, arrival) and its cells in 8×8 chunks with a `u64` mask (`LevelCells`, secret doors as wall, stairs with their way down), and `OnWorldMap` on leaving. It sends what the player glimpses from every cell within 2 steps (`LEVEL_PRELOAD_STEPS`), so the client works out sight itself on the cells it has (unknown ones are wall). What the player has seen of each level is remembered by the server (saved with the character, below) and sent first, marked `remembered`, on coming back. The client draws remembered cells out of sight with grey copies of the tile pictures at an even shade, including the outside of walls seen from inside, and remembered floor can be clicked to travel to. The client never has a site's seed any more. Dev tools enter made-up sites by `DevEnterSite`, which only the embedded server accepts (`ServerOptions::dev_tools`).

## 2026-10-29: Server saves in RON files, one per world, site and character

Step 4 of the map data design. The server keeps its data in its own folder (`Hall of Worlds/server` in the user's data folder, or `HALL_OF_WORLDS_SERVER_DATA`), as RON files: `worlds/<world id>/world.ron` (seed, world map version, save format), `sites/<site>.ron` (generator version; per level the last visit and players' changes) and `characters/<name>.ron` (position, current visit, features and cells seen). RON because people can read and fix it, it takes any map keys (unlike JSON), Bevy already depends on it, and fields added later with `#[serde(default)]` keep old saves loading; one file per site and character so saving one doesn't rewrite everything. Binary or a database can replace it if saves get big. A site gets one generator version for all its levels (not one per level as first planned), since its levels have to agree on their stairs. The embedded server saves and stops when the game quits (the client waits for it), plus every 5 s in case the process is killed. The character is chosen by whoever accepts the connection (the embedded server's one `player`), not by the client: accounts come with networking.

## 2026-10-29: Movement is steps, predicted by the client and confirmed by the server

The client never tells the server where the player is, only the steps they take (`ClientMessage::Step`, one cell in 8 directions, checked with `shared::movement` on both sides). It shows each step at once and keeps the unconfirmed ones; once a tick the server sends `Position` (where it has the player after the last step it handled), and the client adds its later steps and moves the player if that differs. This keeps a fast-feeling game over a network without trusting the client. Steps carry how many map changes (`EnterLevel`, `OnWorldMap`) the client has seen, so a step sent just before going down stairs isn't applied on the next level. Click travel stays a straight line that ignores walls until pathfinding, which the owner wants in its own session (and is useful now for testing refused steps). No speed limit yet. Dev tools' starting position is a `DevTeleport` the embedded server answers.

## 2026-10-29: Dev tools teleports: ctrl+click and Esc

At the owner's request, since steps can no longer go through walls: ctrl+click on either map asks the server to teleport the player there (`DevTeleport`, through walls but onto a walkable cell in a site, answered with `Teleported`), and Esc puts them back where new characters start on the world map, out of any site (`DevBackToStart`). Like all dev tools messages, only the embedded server accepts them; other servers now ignore dev tools messages with a warning instead of disconnecting, so a stray key or click doesn't end the game.

