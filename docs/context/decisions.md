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

Rules, world generation, RNG, validation and protocol types go in `shared`, used by both sides. Client and server are both Bevy apps (server headless) so shared ECS systems run in both. Whether visual-only features live in `shared` or `client` is decided per feature and recorded in its context file. `crates/shared` and `crates/client` exist; `shared` is plain Rust with no Bevy dependency until shared ECS systems are needed. `crates/server` was added later (see "Server crate" below).

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

`crates/server` is the authoritative server: a library (`ServerPlugin`) plus a standalone headless binary. Native clients link it for single player; the web client depends on it only through a non-wasm target dependency, so browser builds contain no server code (which uses the filesystem) and only connect to a server. The server uses Bevy without default features so it stays headless.

## 2026-10-28: Docker image published to GHCR on every merge

A multi-stage `Dockerfile` builds an x86-64 image (ARM can be added under the same tags later) containing the game server and the web client, served on port 8080 (now by the game server itself, below; at first by static-web-server alongside it). The Docker workflow builds and publishes `ghcr.io/rycochet/hall-of-worlds:latest` on every push to `master` (no PR builds). The Build workflow, run manually, keeps native (client + server, three OSes) and web release builds as artifacts.

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

Asked for by the owner: a second view for dungeons and buildings, a grid of floor and wall cells (about the size of the player's picture) in 3D, with the world map's camera controls and tokens. Walls block movement, sight and light; floors block nothing. Only what the player (or anything sharing its sight, `common::SharesSight`) can see (or glimpse: see the AngbandTk entry) is drawn, and of walls only the sides someone sees (at most two per viewer, never the top). A seen side facing the camera's position is solid, one facing away (the wall is between the camera and what is seen) is 50% see-through, and anything out of sight isn't drawn. Left-clicking a seen floor travels there (by `shared::pathfinding`, see "Pathfinding on levels"). Light: dull everywhere, plus light sources (`common::LightSource`; the player gives 6 cells of white light), blocked by walls. Surfaces are drawn with tile pictures (see the AngbandTk entry). The game started in a 15 × 7 test room while the view was built; since sites can be entered (see "Site generators") it starts on the world map again.

Structure: the camera and tokens moved out of `world_map` into a new `view3d` module both views use; each view draws its own scene and tells `view3d` its cell size and floor height (`ViewGrid`). Which view is on screen is a Bevy state, `common::GameView`; each view hides (not despawns) its scene when not current. The grid rules (cells, walkability, line of sight and light reach, by symmetric shadowcasting in exact integers) are in `shared::dungeon`, since the server must agree on them. How bright the light looks is worked out on the grid in the client and baked into vertex colours, so tiles light exactly by the grid rule and coloured lights add up; tokens get real point lights instead. Views are switched by entering and leaving sites (the M demo switch was removed then).

## 2026-10-28: World map pois, one per chunk, chosen from the chunk's position

Asked for by the owner, as the first step towards dungeon generation: the game will have many kinds of maps (single buildings, towns and cities, dungeons and caves), so several generators. What is at a place, and the seed its map is generated from, both come from its world map position. The world grid is cut into chunks (`CHUNK_SIZE` 64 points, chosen so a 400×400 window shows a few dozen); each chunk has at most one poi (80% chance, the owner's figure), and the chunk's position alone decides whether it has one, its kind, its exact position in the chunk (at least `EDGE_MARGIN` 8 from the edges, so tokens never overlap) and its map seed (`shared::poi`, since the server must agree). Every poi is an entity with a white octagonal token (owner's choice) and a picture: the 34 kinds are the Dungeon Odyssey terrain tiles `l2_terrain066`–`099` (David E. Gervais, CC BY 3.0, the owner's selection), stored like the monster tiles. The client spawns entities only for pois near the player while the world map is current, and the world map hides tokens outside its window. `MapPosition` and `SmoothPosition` moved from `player` to `common`, since pois have positions too. Pois stand only on dry land, above high tide (owner's choice): up to 8 spots in the chunk are tried, and a chunk without dry land among them has none, so the 80% applies to chunks with land.

Follow-up (owner): every `Token` has a `size` (`TokenSize`: tiny 0.5, small 0.75, default 1, large 1.25, giant 1.5, titan 2 × the original icon size) that scales only its icon; bases are always the same size. Pois are tiny (their 54×54 pictures fill the tile; half size looks right, especially the castles), the player default (it looked right in the dungeon at the original size). Icons are now always drawn in front of their own token (a long-standing bug that cut their lower part off, noticed once many were on screen).

## 2026-10-28: RogueBasin as the roguelike development reference

The owner's pointer: <https://www.roguebasin.com/index.php?title=Articles> collects articles on every part of roguelike development (map and dungeon generation, field of view, pathfinding, AI, items...). Check it when starting a new area. The site generators follow its "Basic BSP Dungeon generation", "Cellular Automata Method for Generating Random Cave-Like Levels" and "Random Walk Cave Generation" articles.

## 2026-10-28: Site generators: buildings (BSP) and caves

Asked for by the owner: every world map poi is the entrance to a site of one or more levels, generated deterministically in `shared::site` from the poi's map seed; the poi's kind picks the generator, through wrappers giving each kind its own options (`Generator::for_poi`). Only shapes for now: walls, doors (rarely secret), floors, plus outside ground, water and lava; mobs, items and decorations later. Every walkable cell must be reachable from where you arrive (tested for every kind). Cells planned as part of a room record which room (`dungeon::RoomId`), for mobs later.

- **Buildings** (BSP, the owner's choice): an outline (rectangle or square, corners cut at 45° or rounded, circle; more primitives later) filled by splitting with 1-cell walls or corridors; each final part is a room filling it to its walls (the owner's "rooms expand to fill the space"), so no wall is thicker than 3; corner rooms follow cut corners. A 3-cell ground border outside, with at least one door in (south-facing if possible), and more as buildings get bigger (owner's follow-up); the inside is only joined up to what those doors reach, so a side door may lead to a part of the building the main door doesn't. Bigger buildings have more floors; each floor up keeps the outline and is, as the owner put it, the same size (50%) or smaller by a cell per 10% of the roll (taken as 1–5 cells on every side, so walls stand on the floor below). Stairs line up between floors.
- **Caves** (also dug dungeons such as crypts, with doors): irregular caves (no rectangles) joined by winding passages; each level's size is its own. The way in is a hole (stairs down from the world map) or a cave mouth: a ground border on the south side (the world map's lower side) and sometimes the east or west, as wide as the mouth rather than the cave. Water and lava pools may block the way only where there is a way round.
- Doors are open (flat, walkable, see-through) until opening and closing come with mobs; secret doors act as wall and are only placed where nothing depends on them.
- Client (`site` module): clicking a poi's token travels there and enters its site; clicking stairs travels there and takes them; clicking outside ground on the very edge of the level travels there and leaves for the world map (owner's follow-up: ground with walls or ground all round doesn't count, so the player can walk round a building to another of its doors). The game starts on the world map. Travel in sites follows `shared::pathfinding` (see "Pathfinding on levels").

## 2026-10-29: Dungeon walls and floors from the AngbandTk tiles; stairs in 3D

The owner's choice: David E. Gervais's flat AngbandTk tiles (same source and licence as the other art, CC BY 3.0, credited in `ATTRIBUTION.md`), whose `dg_features32`, `dg_grounds32` and `dg_extra132` sheets have wall and floor textures. The sheets are split into one PNG per tile in `assets/angbandtk/` (the owner's suggestion), and the dungeon view draws each kind of surface with its own picture: walls and floors themed by the kind of place (stone, wood, crypt, cave, ice; a client-only, visual choice in `dungeon_map::theme`), doors, water and lava the same everywhere (outside ground's picture too, painted the local land's colour: see "A site's outside shares the world map's land and weather"). Light stays baked into vertex colours.

The stairs tokens (Gervais `l2_stairsiron…` pictures) didn't read well (owner), so stairs are now part of the level's 3D drawing, half a cell wide down the middle of their cell with floor round them (owner's follow-up): stairs up are a short flight of masonry steps, stairs down a hole in the floor with steps going down, darker the deeper. Both ends of a staircase run the same way (owner: if the top is in the west going down, the bottom is in the west going up), worked out from both levels (by the server since step 3 of the map data plan, `Site::stairs_descend`). Stairs are cells of the level rather than entities: arriving on one by a click takes it.

Follow-up (owner): The view draws what a viewer glimpses (`Sight::add_glimpses`: open cells partly in view count), not just the symmetric sight, so side openings off a long corridor show instead of leaving black gaps; game rules keep symmetric sight.

## 2026-10-29: Map data from the server: pushed chunks, three states, a secret seed

Designed with the owner before splitting generation (server) from display (client); details and build order in `architecture.md` ("Map data"). The client generates only what isn't secret: world map terrain, from a terrain seed derived one way from the secret world seed. Pois and sites come only from server messages, as what the player would see (secret doors arrive as wall). Every map is the generated base plus persistent server changes (versioned per chunk); on the world map the server sends the changes to the public base plus the secrets now visible to that player, including per-player poi state (visited, attitude). Site levels go in 8×8 chunks with a `u64` mask of the cells included. The server pushes what each step makes visible, plus what is visible from within K steps (`LEVEL_PRELOAD_STEPS`, now 3, a server setting), so the client can work out sight itself; the client keeps loaded / visible / seen apart and draws remembered cells in greyscale, still clickable to travel back. The server keeps each character's seen masks, so knowledge follows the player to another machine. A world's id is derived from its seed, so the same world is the same on every server (the owner wants worlds shareable between players); it will key a client cache. Single player: the embedded server runs in its own Bevy world and talks to the client through an in-process channel with the same messages, which will also let a single-player game be opened to others. The game is open source, so by default nothing stops a hacked client reading what it was sent (the server stays authoritative over the rules); stricter anti-cheat is planned as a server owner's option.

Follow-up (owner): server data and client data (cache, settings) are always in separate folders and never overlap, also with the embedded server. Every map generator, the world map included, has its own version, a plain incrementing number (semver stays for the apps), bumped only when its output changes in a way that matters, not for cosmetic changes (tests to enforce that later). The server stores the version each map was generated with; already generated maps keep their old version, and only new ones use the new generator. Every version still in use stays runnable in `shared`: the world map is infinite and generated by the client too, and site levels are regenerated rather than stored whole, because their generators also spawn mobs and items, during play and when a site resets after nobody has been in it for a while. Persistence matters more than resets (owner: the world is big, with no starting area to farm): changes that aren't the generator's (mining through walls, breaking doors) are stored as an overlay on the generated base, and a reset brings back only the generator's spawns, never undoing them.

## 2026-10-29: Server in its own world, talking through an in-process link

Step 1 of the map data plan. Native clients start the server on its own thread as a separate Bevy `App` (`server::start_embedded`), and the two talk only through a `shared::protocol::Link`: a pair of in-process channels carrying postcard-encoded messages, so nothing can be shared by accident, and over the network WebSockets carry the same bytes (see the networking entry below). The client starts in a new `GameView::Connecting` state and gets its world from the server's `Welcome` (world id, terrain seed, world map version); starting states from `dev_tools` wait for it. The world seed is now secret: terrain comes from a seed derived from it, so the fixed world (12345) looks different from before. Until networking (below), the web client had no server and only showed that it was connecting (owner's choice, over a temporary offline stand-in). Step 2 (below) removed the pois seed from `Welcome`.

## 2026-10-29: World map pois sent by the server

Step 2 of the map data plan. The client no longer has the pois seed: the server sends each 64-point chunk within sight (300 points each way) plus one chunk more, once per connection, as a `WorldChunk` with its poi or none. The client keeps them in `common::KnownPois` and spawns poi tokens from that. (The client first reported its position for this; movement now goes through the server as steps, below.)

## 2026-10-29: Site levels sent by the server, remembered cells drawn in grey

Step 3 of the map data plan. The server runs visits to sites: the client asks to enter a poi (`EnterSite`, only from within 2 points of it), moves about the level (now by steps, below) and asks to use the stairs or edge ground it stands on (`UseExit`); the server answers with the level (`EnterLevel`: kind, floor, bounds, arrival) and its cells in 8×8 chunks with a `u64` mask (`LevelCells`, secret doors as wall, stairs with their way down), and `OnWorldMap` on leaving. It sends what the player glimpses from every cell within `LEVEL_PRELOAD_STEPS` steps (2 at first, now 3, a server setting), so the client works out sight itself on the cells it has (unknown ones are wall). What the player has seen of each level is remembered by the server (saved with the character, below) and sent first, marked `remembered`, on coming back. The client draws remembered cells out of sight with grey copies of the tile pictures at an even shade, including the outside of walls seen from inside, and remembered floor can be clicked to travel to. The client never has a site's seed any more. Dev tools enter made-up sites by `DevEnterSite`, which only the embedded server accepts (`ServerOptions::dev_tools`).

## 2026-10-29: Server saves in RON files, one per world, site and character

Step 4 of the map data design. The server keeps its data in its own folder (`Hall of Worlds/server` in the user's data folder, or `HALL_OF_WORLDS_SERVER_DATA`), as RON files: `worlds/<world id>/world.ron` (seed, world map version, save format), `sites/<site>.ron` (generator version; per level the last visit and players' changes) and `characters/<name>.ron` (position, current visit, pois and cells seen). RON because people can read and fix it, it takes any map keys (unlike JSON), Bevy already depends on it, and fields added later with `#[serde(default)]` keep old saves loading; one file per site and character so saving one doesn't rewrite everything. Binary or a database can replace it if saves get big. A site gets one generator version for all its levels (not one per level as first planned), since its levels have to agree on their stairs. The embedded server saves and stops when the game quits (the client waits for it), plus every 5 s in case the process is killed. The character is chosen by whoever accepts the connection (the embedded server's one `player`), not by the client: accounts come with networking.

## 2026-10-29: Movement is steps, predicted by the client and confirmed by the server

The client never tells the server where the player is, only the steps they take (`ClientMessage::Step`, one cell in 8 directions, checked with `shared::movement` on both sides). It shows each step at once and keeps the unconfirmed ones; once a tick the server sends `Position` (where it has the player after the last step it handled), and the client adds its later steps and moves the player if that differs. This keeps a fast-feeling game over a network without trusting the client. Steps carry how many map changes (`EnterLevel`, `OnWorldMap`) the client has seen, so a step sent just before going down stairs isn't applied on the next level. Click travel in sites follows `shared::pathfinding` (see "Pathfinding on levels"), so steps are normally accepted; refusals come from out-of-date memory. No speed limit yet. Dev tools' starting position is a `DevTeleport` the embedded server answers.

## 2026-10-29: Dev tools teleports: ctrl+click and Esc

At the owner's request, since steps can no longer go through walls: ctrl+click on either map asks the server to teleport the player there (`DevTeleport`, through walls but onto a walkable cell in a site, answered with `Teleported`), and Esc puts them back where new characters start on the world map, out of any site (`DevBackToStart`). Like all dev tools messages, only the embedded server accepts them; other servers now ignore dev tools messages with a warning instead of disconnecting, so a stray key or click doesn't end the game.

## 2026-10-29: Networking over WebSockets only, on the web server's port

The Docker image had become unplayable: it ran the web client and a server with no networking. The owner's requirements: WebSockets only (no other fallback), one port for everything (the server serves the web client's files too, so it works behind Cloudflare and an HTTPS port with nothing else opened), automatic reconnecting, and "Connecting..." in red at the bottom left meanwhile. The server uses `axum` (with `tokio` and `tower-http`, MIT) on its own thread, handing each WebSocket to the Bevy world as the same `Link` single player uses (its bytes carried by a `Wire`, one game message per WebSocket binary message); the web client uses the browser's `WebSocket` directly through `web-sys`, and native clients can join a server with `tungstenite` (MIT/Apache-2.0). `ewebsock` was tried and dropped: in the browser it switches incoming messages to `Blob`s after the first send, which are read asynchronously, so messages could arrive out of order; and natively it sends at most one message per 10 ms poll. No accounts yet: a random guest number from the client (kept in the browser's local storage) names its character, and the newest connection for a character wins, so a reconnect gets it back. The Docker image runs the server alone, saving in a `/data` volume, and it saves everyone on SIGTERM. Planned: TLS for the server's own connections (see `architecture.md`). This settles the earlier open question of a networking library: `bevy_replicon` with `bevy_replicon_renet2` (server-authoritative replication, with WebSocket/WebTransport for the browser) and `lightyear` (prediction, rollback and interpolation) were the candidates; hand-rolled WebSockets won for control and simplicity with our own pushed-chunk messages, and those remain options if replicating entities (mobs, other players) gets complex.

## 2026-10-30: Thin walls, and sight that follows them

Asked for by the owner. A wall no longer fills its cell: it is a slice about 10% thick through the middle (45–55%), running out to whichever neighbours are walls, or doors and windows in line with it, so walls make straight runs, corners, Ts and crosses; with no neighbours it is a column, with one it stops 60% of the way across. It still blocks movement in the whole cell, but sight and light only where it stands (`shared::dungeon::WallShape`, in the same exact integers on both sides). This replaced shadowcasting with a straight-line check from cell centre to cell centre (still mutual, deterministic), since a shadow per cell can't express part of a cell; the server only looks again from cells it hasn't looked from, and the client only when a viewer moves, not the camera. Walls touching only at corners don't join, so sight passes between them, as stepping across such corners already could. A wall's shape depends on its neighbours, so the server also sends the cells beside what it sends, whether or not they are seen.

New cell: `Cell::Window` blocks movement but not sight. Doors and windows are exactly one cell wide, in line with walls at each side, and drawn as a wall with a hole; they block nothing in their cell, at the drawn frame either. Generators don't place windows yet.

How it looks is the client's (`dungeon_map::walls`), chosen by the theme for now and later by tile: `Built` places (stone, wood) have straight walls, square doors and framed windows with a sill taller than the lintel; `Natural` places (crypts, caves, ice) have walls curved like a letter C, narrowest half way up at the same thickness, arched doors and windows made of lines of columns, stalagmites and stalactites; more variations and material relief are planned. Dull light halved (`AMBIENT_LIGHT` 0.1, and the tokens' ambient); it still counts as light, so unlit cells show dimly rather than being unseen. Remembered surfaces are now half see-through as well as grey (owner: to tell them from grey maps, to be judged in play), and darker so memory isn't brighter than dimly lit cells. Dev tools: `DevSetCell`, by alt+click and `GAME_CELLS`, to try wall shapes.

Follow-up (owner): walls side-on to the player are drawn, see-through, and both visible sides of a wall always have the same transparency: if a seen side is turned away from the camera, all of them are see-through.

Follow-ups (owner): the floor of a wall's cell stops at the wall's foot, with nothing under the wall, and in `Natural` places the wall's picture fades out across the floor beside it; `Built` places keep the hard edge. A side out of the player's sight that faces the camera, in a part of a wall that shows from round its end (wall ends, columns), is drawn as memory rather than left as a hole, even if never seen before, since the wall is already known. Later (owner): under solid walls the foot's ground is filled in the wall's picture, because the curve let the background show as thin black slits; see-through and remembered walls keep the dark band.

## 2026-10-30: Liquids are channels in the floor, and one cell of them can be jumped

Asked for by the owner, following on from thin walls. Rules (`shared::movement`): a step onto liquid carries on over it onto the walkable cell beyond (a jump, still one `Step` intent, so the protocol didn't change, and the server decides where it lands); nobody stops on liquid, and liquid that can't be crossed that way can't be stepped onto. Until swimming exists, wider water is a barrier. The player can't take another step or turn until they have landed; a journey starts its jump at the edge of the liquid cell and is free again at the edge of the landing cell (the owner's suggestion).

Drawing (`dungeon_map::liquid`): a cell with floor on two opposite sides is a channel half a cell wide and half a cell deep; corners and pool edges meet those edges, curving round with the outside of a bend past the cell's middle; where liquid cells meet the depth reaches a full cell, except between two narrow cells (owner's follow-up: a narrow channel's bed stays flat rather than dipping at each seam). A translucent surface, with sides and bed in the surrounding floor's picture. Walls count as liquid: liquid runs up to the wall's foot and the wall carries on straight down to the bed, keeping its curve above, with its picture repeating rather than stretching. Water is translucent and lava solid, by surface, so other liquids can sit in between.

## 2026-10-30: Pre-commit lint hook

`.githooks/pre-commit` runs `cargo xtask lint` (formatting, clippy for native and wasm) before each commit, turned on per clone with `cargo xtask hooks`. It catches locally what CI no longer checks. The tests stay out of it: `cargo xtask test` before merging still covers them. It lives in `xtask` rather than a PowerShell or shell script for the reason in "Test command `cargo xtask test`"; the hook itself is one line of shell because git requires that.

## 2026-10-30: Stairs have styles, drawn as 3D models in holes

Asked for by the owner: the single flight didn't work well. Seven styles (`shared::site::StairsStyle`): spiral, turning (180 degrees, solid to the ground), ladder, log ladder, rope ladder, slat ladder, knotted rope. Going up, the model stands on the floor, built taller than going down so it climbs at a believable rate and cut off at 1.5 tiles, without handrails. Going down, it is sunk into a hole of about its outline, with a shaft side and a black base, ladder tops coming up above the floor, ropes tied to pegs, and on the two stairs a handrail round the top of the hole rather than on the steps. Stairs hiding a viewer go half see-through, like walls in the way. Clicks only land on the floor of a stairs' cell, not the model above it, so what is behind can be clicked (the owner may change this back). The server sends the style in `SeenStairs`, as it does the way a staircase runs. Which styles suit which kind of place, picked by a hash of seed and upper level, was Claude's choice and is easily changed (`StairsStyle::choices`): stone places spiral or turning; wooden ones turning or ladders; crypts spiral, turning or ladder; caves ladders and ropes. Only for looks: no generator version bump.

## 2026-10-30: Pathfinding on levels: A* with costs in time, then straightening

Shared code (`shared::pathfinding`) for the player, later monsters and some spells; the world map has its own search (see "World map routes"). The owner chose A* over the breadth-first search and plain Dijkstra on RogueBasin: walking time will follow distance and terrain costs are coming, which breadth-first can't take without becoming Dijkstra, and A* is Dijkstra steered towards the goal. Costs are whole-number time units (straight 70, diagonal 99, a jump its distance) from one function, `movement::step_cost`, which walking speed and terrain are to use too. Diagonals are spread out by straightening the path afterwards (lines accepted only if legal and exactly as quick), rather than by a small penalty on consecutive diagonals, which would have made each cell up to 8 search states and bunched the diagonals differently. An unreachable click walks to the nearest reachable cell without counting as arriving. The client only routes through cells the player has seen. A distance map flooded from a goal is the likely addition when many monsters chase one target.

## 2026-10-30: World map routes: water blocks, slopes cost, a limited search

The owner's choices. The world map now has terrain rules, in `shared::movement` and checked by the server for every step: water can't be walked into, and slopes cost time. Water follows the tide, the owner's choice over a fixed shoreline, so walkable ground matches what is drawn: the server now owns the game hour, saves it and tells clients, and the water moves once per game hour. Anyone already in water may step anywhere, so nobody caught by the tide is stuck, and a journey the tide cuts finds a new route. Pois stay above high water, and so never stand on ice, which can be walked on: the server now owns the snow line too (not saved, dev tools set it), and the client draws what it says.

Climbing counts on the terrain's broad height (its three largest noise layers) at a high cost, the owner's aim being that routes go round a mountain about 4 chunks across and over a much bigger one. Measured on real mountains: raising the cost on the full surface, even growing with steepness, barely changed routes, since small bumps cost the same on any route. Routes are A* within 320 points of the player (more than the view shows) rather than a coarse long-range search; an unreachable click walks to the nearest reachable point without counting as arriving. With no speed limit yet, slopes steer routes and slow click travel as shown by the client, not keyboard steps.

## 2026-10-30: World map biomes, rivers and sky

Asked for by the owner. **Biomes** follow the Whittaker diagram (the owner's choice): nine biomes from noise-driven temperature and moisture (`shared::climate`), with height cooling the air, so climbing moves through colder biomes. Colours blend across the diagram in `surface_style`, roughened by `jitter`. The old hill and mountain bands are gone (owner): high ground turns white by temperature, so how much snow there is follows a planet's new `warmth` and the local climate, tuned to about the same height on average as before. This replaces the fixed bands in the Legend entry's look. High ground also fades to tundra's colours nearly everywhere (owner, to try: more interesting maps). Colours are stored so a world's base hue can rotate later (the owner wants more than green, with green most likely), each biome taking as much of the rotation as suits it.

**Rivers** (owner's design, deliberately simple): a chance per chunk of a source, by biome from 3% in deserts to 30% in rainforests (first 5% everywhere; the owner wanted more, fewer where dry), running downhill until the sea or a low point, where they just end. They affect nothing for now, not even movement; merging happens only because downhill converges; width depends only on how far they have run, 1 px at the source and 1 px more per chunk up to 4 px (at first 4 px at any sea mouth, which made short rivers fat). A dark see-through edge sets them into the ground. They freeze white before the land around them, the more so the higher up, meeting the sea's shallows at about the same time.

**Sky**: clouds and rain come from 4D simplex noise (the wrapped surface's three dimensions plus time; hand-written in `shared::weather`, as `fastnoise-lite` stops at 3D), so all clients draw the same sky from the public terrain seed and the server's clock. That clock is now a float of game hours that always runs (owner): the server's own, saved, with `Hour` still sent on each whole hour for the tide, and clients keep theirs in step with a `Ping`/`Pong` once a second, which also gives a latency figure, easing rather than jumping so clouds never jerk. This brought the first custom shaders (WESL): low-resolution sky textures worked out on the CPU a second apart and faded between, read by a terrain material for moving water and a slight darkening straight below clouds (the owner's choice over shadows along the sun's direction), a cloud sheet and precipitation (rain, sleet, snow, hail). The sheet is straight above the map, its exact shape with a hard edge, and doesn't move relative to the map as the camera turns (owner). Clouds are at most 30% opaque, so the map stays readable, and drawn in blocks about 8 screen pixels across for a low-resolution look (owner).

To do: lakes at river low points (owner's idea); choosing the base hue per world; vegetation by biome and elevation, to give the land texture.

## 2026-10-30: World map vegetation: painted by the terrain shader, on a raised canopy

Asked for by the owner, chosen from several options after a proposal. Tree cover and low growth per grid point (`shared::vegetation`, from climate and patch noise; visual only for now, in `shared` as forests will probably matter to rules) reach the terrain's fragment shader in the mesh's UV slots. The shader draws tree crowns and grass with bump shading, changing the existing colour. On top, the owner chose after comparing them to raise the drawn surface under woods by a few units, accepting that later surface decoration has to clear the canopy around it. Rivers do this already: woods are cut back about 4 units either side, wider than the river as grid points are a unit apart, so they run in a fold, which from low angles hides parts of them. Picking, tokens and edge faces follow the raised surface; walking and routes don't, as this is only drawn. Rejected: layered shells and individual trees (more drawing work than the detail wanted).

**Planned, not built:** fields and roads should show in preference to vegetation (owner). Grid points are too coarse for fence lines and tracks, so they would be thin meshes draped over the terrain like rivers, clearing the canopy around them the same way, probably wider and flatter for roads. They should come from the server rather than be worked out on the client, as they run to pois and would otherwise reveal undiscovered ones.

## 2026-10-30: No sand colour: the shore comes from the biome

The owner's call: the fixed yellow sand was meant to mark the water's edge and the tide, but it detracted from the map. Shore points (just below the water) now show the biome's colour turned 30 degrees round the colour wheel and darkened 20%, so they still mark the tide. This also means shores follow a world's base hue. Colour only: where the shore is, and drawn heights, are unchanged. Deserts are still a biome.

## 2026-10-31: A site's outside shares the world map's land and weather

Asked for by the owner: `Cell::Ground` is the world outside, so it matches the place the site was entered from (`common::WorldReturn`, moved from `site`). The client works it out itself from the public seed, so nothing is added to the protocol: the ground takes the land's colour there (biome or snow, one colour per site), and the weather there falls on outside cells in sight, with the world map's rules. Purely visual. Clouds and daylight aren't carried over.

## 2026-10-31: Towns: solid buildings on roads, known from the start

Designed with the owner. A town's size is decided by its poi's kind (hamlet, village, town, city, up to 150 buildings), rolled by a separate hash so other chunks keep their poi. That changes the world map, so its version is 2; the owner agreed not to keep version 1 runnable, and saved worlds at 1 simply carry on with some pois turned into towns. A town is a straight road, T or crossroads, a square by size, gardens, and a ground border; it grows building by building from a list, each beside an earlier one and lined up with it, with side roads getting likelier as a road gets longer (the owner's design, replacing blocks cut by roads with plots laid along them, which spread out too much). Owner's rules: no more than 10 cells of building in a line without a gap (one building may pass it, but nothing may then be added), a road never replaces another road, parallel roads never touch (kept 3 cells apart, room for a building), and roads reaching the border run on to the edge of the level. Houses have a plain door with no sign. Buildings are solid (no inside), at least 3 × 3, sometimes L, T or H but with no single cell sticking out, each with its door in a straight wall facing something walkable: a road unless it is a house. A town's layout is sent whole on entering and its `RoomId`s (buildings) are public, which is the one exception to `RoomId`s staying on the server.

Walls of different buildings don't join (`dungeon::Level::wall_shape`), so there is a gap to see and to see along, which can't be walked: the owner chose real sight through gaps over blocking it (line of sight will be tweaked later anyway, and a town has no per-step sending to pay for it). `Cell::BuildingDoor` and `Cell::BlindWindow` count as wall for every rule; only the client draws them differently, and nothing can be seen through a window. Using a door is walking to the cell it faces.

Kinds: houses first (the owner accepted more than 30% where the layout forces it), then by weight with a minimum town size per kind, each copy halving that kind's weight (owner), and falling much faster past a twentieth of the town (added because plain halving still let a rare weight's rivals pile up in villages). Each building has a quality (0–1000) and trade count (10–1000, by town size) from hashes; the server will keep trades added and the clock at the last interaction (`BuildingSave`), for stock refreshes later. Light is fixed in towns for now (owner: positional lighting comes with decorations). Everything a town draws is one atlas (`Surface::Town`) rather than some 40 more surfaces, each of which would cost its own meshes and materials. Gardens are at least 3 × 3, about one per 25 plots, larger minimum in larger towns; their hedges (20% of a cell thick, tall and inset) and arches (1.5 cells tall) are client-side only and block nothing (owner). The half cell under a wall's outside is always the land's own ground, not the road or garden beyond it (owner).

## 2026-10-31: Minimap and full map, painted on the CPU

Asked for by the owner: a round minimap in the top-right corner whose contents and north notch turn with the camera, and a parchment map on M or a click on the minimap, closed by M, Esc, an X button or a click outside it, opening at twice the minimap's scale and zoomed by the wheel towards the mouse up to 4x, dragged with the mouse but never away from what is known. Only known things show: on the world map the land's plain biome colours and pois' pictures, in sites remembered tiles as a grid coloured by their ground, with hatching inside the walls along what they meet (floors, dungeon doors, other buildings), house doors that look different from dungeon doors, shop signs, and stairs as a flat stepped icon whose high side says which way they go. Both are painted on the CPU into an image shown by Bevy UI, rather than by an extra camera or a shader, because the pictures have to stay upright while the map turns, and it needs no new shaders. It only redraws on a change; its cost has not been measured. The maps take their mouse and keys in `PreUpdate`, out of Bevy's input resources, so no other module needs to know about them.

What the world map knows is now the chunks a character has seen, saved with them whether or not a chunk has a poi (`CharacterSave::pois`, which held only chunks with one) and all sent again on connecting, so the map survives a restart. Worlds saved earlier only remember their chunks with pois, so old explored land shows gaps until walked again.

Claude's choices, easily changed: the full map keeps north at the top, the minimap hides while it is open, the keyboard still walks under it, and rivers aren't drawn (the owner's list was walkable tiles and biome colours). `widgets::raised_box` is the shared raised box, for the map's hover name first and other things later (owner).

## 2026-10-31: World map features are now called pois

The owner's choice: "poi" (point of interest) replaces "feature" for things on the world map, in code and docs (`shared::poi`, `Poi`, `PoiKind`, `PoiMap`, ...). "Feature" still means a game feature (feature modules) and a Cargo feature. Three old names stay because changing them would damage existing worlds: the seed name `derive("features")` (it would move every poi), site files `sites/feature_<x>_<y>.ron`, and serde aliases `Feature` and `features` so saved characters still load (without them a character file is put aside as broken).

**To do at the next full wipe of saves (owner):** rename `derive("features")`, the site files and the character fields too, and drop the aliases.

## 2026-10-31: Pois are hidden until known, and shown as 3D models

The owner's rules: nearly all pois start invisible, and only towns are seen as the player walks around. How a character learns of the others is for later (`KnownPoi::known` is ready, nothing sets it). The server decides this and never sends what isn't known, including for the map or on entering. Woods are cut back only round pois this character knows of, so they give nothing away, and this is worked out per character rather than cached for the world. `GAME_REVEAL_POIS` shows them all in development, on that connection only.

Pois are small 3D models standing on the land, with no token base: placeholders built in code, since the project has no models yet, to be replaced kind by kind. They reflect the site through `shared::site::PoiLook`, which the server sends in place of anything secret: it comes from a generator's plan without generating a level, so a tower's height is its real floors and a town's size its real buildings. Claude's choice: the maps' pictures are the same triangles rasterised isometrically on the CPU when first needed, rather than baked PNGs, so there are no files to keep up to date and every look gets one. Claude's choice, not yet the owner's to judge: plain colours rather than textures from the old tiles.

**For now every poi is known (owner):** `world_map::ALL_POIS_KNOWN` sends them all to every character, as nothing teaches a character of one yet. **To do when characters get means of obtaining pois:** turn it off and set `KnownPoi::known` from those means; the hiding, entry check and per-character woods are already built and tested with it off.

## 2026-10-31: Ruined buildings, `Cell::Hole` and cave skylights

The owner's design. `BuildingOptions::ruin` (0 to 5) damages a building's finished floors with its own seed, so the floorplan doesn't change. A ruined building's floors depend on the floor below, so they are made together and kept rather than each generated alone; unruined buildings still generate per level. The ground floor loses cells to `Cell::Ground`; floors above lose them to the new `Cell::Hole` (open to sight and light, not walkable, not outside), which underground dungeons can use later. Holes are drawn as geometry, with dark through them, because the client doesn't have the level below. One cell of hole can be jumped, as liquid can (owner's). Ruined places are named "Ruined ..." (owner's), through `LevelInfo::ruined` and `PoiLook`. `for_poi` passes ruin 0: which pois are ruined is decided with poi visibility.

Generator versions: their old code was removed with the first `FORGET_BEFORE` bump (see "Starting clean in development").

## 2026-10-31: Which buildings are ruined

Each building's `ruin` comes from its poi's map seed (`Generator::for_site`), so the world map's model and the site agree. Claude's first numbers, to tune by eye: 30% of buildings are ruined, evenly from 1 to 5; tents and ice shelters never are.

## 2026-10-31: Starting clean in development

The owner's request: a way to make everyone forget everything from before a point. `shared::site::FORGET_BEFORE` is a number set in the code; bumping it makes servers ignore (and save over) characters and sites saved under a lower one, and raises every generator version with it, so all sites are made new. Clients keep nothing between connections but their guest id, so forgetting on the server is enough for them. Old files aren't deleted until saved over. The world map version isn't raised: a mismatch there refuses the world, and the world comes from its seed anyway.

First used straight away (owner): `FORGET_BEFORE` is 1, as ruins and pois had just changed, so nothing saved under the earlier generators is kept and their old version code is deleted: a clean sheet. The one old version still accepted is world map version 1, in `ServerWorld::open`: it isn't generator code, and removing it would refuse those worlds rather than clean them.

## 2026-10-31: Extra building rules

The owner's rules: towers have at least 5 floors, all the same size, though one with more may have a top about half the size; castles and forts have at least 3 floors; pyramids go on up, 3 to 4 cells smaller per side each floor, until a 3 by 3 room no longer fits; and buildings with more than one floor may have more than one staircase between two floors, more likely the bigger they are. The staircases needn't connect on the other floor, as everything only has to be reachable from some way in. Generated floors are still connected for now, so that freedom is only used when ruin cuts one off. `FORGET_BEFORE` went to 2 for this, as what buildings generate has changed.

## 2026-10-31: Other floors on the full map, numbered by locale

The owner's design: buttons on the full map (not the corner minimap) for the floors the player knows, numbered from 0 in the EU, UK and Latin America and otherwise from 1. Knowledge stays the server's: it sends what the character remembers of a site's other levels on coming into the site (`RememberedLevel`), and the client keeps the level it leaves by stairs, so taking stairs doesn't generate or send every known level again. The locale comes from `sys-locale` (MIT/Apache-2.0, already in the build through Bevy), which covers native and browsers; it gives the language setting's region, which is usually but not always where the player is.

## 2026-10-31: Farms: fields, fences you jump, and looks chosen by the client

The owner's design: a farm poi is a farmhouse of 2 floors, sometimes with a basement, in the middle of 1 to 8 rectangular fields, one of which may be a pond. `Cell::Field` is walkable like a garden. Fences began as looks, like hedges, but the owner chose to make them cells that work like water: they can't be stepped on, one can be jumped, sight passes, and only the gate (`Cell::FieldGate`, never on a corner, nearest the front door) is walked through. So a fence takes a cell of its own, and fields need a walkable gap between them to land in. A jump costs its distance, so click travel hops fences rather than going round by a gate, as it does over water; the owner accepted that.

What grows is cosmetic and client-side: five stages (ploughed, flat grass, 6, 12 and 18 pixels tall), chosen from a number the server sends with each field (`FieldLook`, `ServerMessage::Fields`), since the client may not know a whole field's cells; a livestock field stays at flat or short grass. The time of the in-game year is to decide it later. Crops have a mesh of their own, rebuilt only when sight changes, not with the camera as everything else is. Ruined fields are, by a hash of each cell, 20% the land's own, 20% ploughed and 60% flat grass (owner), and fences lose stretches.

Claude's additions, agreed by the owner: 3 × 3 places round the house, dirt tracks joining the door, gates and the edge, ponds in a third of farms with room, climate limits (grassland, woodland, seasonal forest, savanna), fixed light outside with the house dark inside. Farms replace what a chunk rolled by their own hash, as towns do, rather than joining the roll, so the kind can be added without moving every other poi; world map version 3 for this, and older worlds carry on with some pois turned into farms (`ServerWorld::open` now accepts any version up to the current one). A site saved under the poi it was before regenerates as a farm.

Basements are the last level of a site, so existing levels keep their indexes (`LevelInfo::basement`, `storey`). The floor buttons show "B", and sort by storey.

## 2026-10-31: Herbalists' huts: a patch that grows by the clock

The owner's design: a one-storey hut of 5 to 8 a side over a basement that can be larger, with one herb patch outside of 3 to 5 parallel strips 7 to 12 long. Each tile (`Cell::HerbBed`) grows herbs or mushrooms, chosen by the client: the same for every client, as it comes from a hash of the place and the server's clock. Every game day a tile moves on a stage (plain ground, green or grey dots, stalks, full grown). The owner chose a fixed list of plants in a mixed order, each with all 4 stages, and each tile starting at a random hour in it, so tiles are at different stages and don't all change at once. Unlike fields, this depends only on time, not season. Upright plants stand in a `#` through the middle of the two plants in each picture, to look more solid.

Claude's choices: 5 shapes of each with 3 colours, so 30 plants in the list; walkable strips with ground between, no fence or track; huts in forest biomes only, by their own hash; a ruin's patch grows on. Farms and huts share world map version 3.

## 2026-10-31: A showcase plain for looking at pois

The owner asked for an area of the world map outside the normal generators, reached by a key press (P; Esc comes back), with one poi per chunk: every kind in order running east, new kinds at the end, 8 unruined seeds of each running south, and one for each level of ruin running north, counting up. `shared::showcase`, behind a flag on for now. Claude's choice: coordinates in the existing world rather than a second planet, which isn't supported; far south of the poles (chunk row 1000), since `y` is unbounded and nobody walks that far, and not further because drawing uses floats. The surface, climate, vegetation, rivers and pois each ask it first. Weather carries on as normal.

## 2026-10-31: Windows in outside walls

The owner's rules: most buildings have windows in their outside walls, each with wall on either side and no wall at a diagonal, which looks tight; castles, towers and forts only above the ground floor, pyramids never. Drawn already; on the maps a pair of parallel lines right across the cell, white between them and beyond them the colour of what is known to be there, with walls beside them hatched as beside a door. Placed by hashes after any ruin, drawing nothing from the floors' sequences, so no generator version changed.

## 2026-10-31: Touch: taps are clicks, two fingers move the camera

Asked for by the owner, after the web client loaded and drew well on an iPad. A new `touch` module makes a tap a left mouse click for one frame, on lifting, so all existing click code works and the start of a pinch doesn't walk the player. Readers of the cursor use `common::PointerPosition` (finger, otherwise mouse) rather than faking the cursor: `Window::set_cursor_position` moves the real mouse on native and fails in browsers. Two fingers pinch to zoom and twist to turn; moving both up or down tilts, Claude's addition since tilt was otherwise out of reach on a tablet. One finger on the main view only taps; on the full map it drags. Target: browsers on tablets and phones, not native mobile apps.

## 2026-11-01: A small widget library

The owner's request, for consistency as more interface arrives: a `widgets` module holds panels with titles, buttons (normal, selected, active, disabled), tooltips and moveable panels, and other modules use those rather than styling Bevy's `Node`s. A moveable panel fits its contents and has a title with three thick stripes running from it to just inside the panel's edge; the bar moves it, and a close button sits on the right, where the stripes stop (owner's choice). `widgets` takes clicks on its pieces in `PreUpdate`, before the maps, so nothing under them sees them, the same way the maps keep input from the game. It tests against laid-out nodes, as Bevy's own `Interaction` doesn't take the click away from other systems and reacts to a finger landing rather than a tap.

Rule (owner): anything said about clicking or other mouse actions also has to work by touch; if how is unclear, ask before building it.

## 2026-11-01: A debug menu instead of debug keys

The owner's design, after RimWorld's debug menu. Backtick or a warning-triangle button opens a full-screen panel titled Debug Menu, with a close button, a filter box and an alphabetical grid; a click, tap or Return chooses, which closes it, a toggle after showing its new state for a second. Entries that can't be used now stay visible but disabled. This replaces the demo module and its keys: G, T, B, F, H, P and Esc's back-to-start became entries (Claude's suggestion for those not named, agreed by the owner), K, C, X and Z went with no replacement, ctrl+click became Teleport Mode, and alt+click became Change Cell Mode. Stairs, Fields and Herbs open moveable panels with Previous and Next. Show Widgets opens a panel with one of every widget (owner). The status text lost its instructions, and the rest is behind Show Debug Info, on by default.

Browsers can't raise the on-screen keyboard from a canvas, and only focus an input during the tap itself, so a hidden HTML input takes typing on tablets, focused by a listener on the canvas (owner's choice over no filter by touch).

## 2026-11-01: A loading screen until the game takes input

A full-screen title and step bar (`loading_screen`) covers the game from the first frame until a few frames after the server's welcome, and takes all input (owner). It replaces the red "Connecting..." line, so it also comes back while the connection is lost and shows a server's refusal. Steps are `common::LoadingStep`, with fixed places on the bar until startup work is timed. The title uses Cinzel Decorative (OFL), built into the program to be there on the first frame. The web page stays black until the wasm has loaded: a browser splash waits until web work is asked for.

## 2026-11-01: Standalone servers can opt in to dev tools

The owner's choice, so a tablet can use the debug menu: `HALL_OF_WORLDS_DEV_TOOLS=1` makes a standalone server accept dev tools messages, from anyone connected, so it is only for private test servers; it stays off by default, and the server logs a warning when on. `Welcome` says whether they are accepted, and the debug menu disables what would be refused. The server still checks every message.

Two more entries, the owner's: Reveal Map (a site's floor is all known, and saved so) and Forget Map (the current floor, or all of the world map, is forgotten, in the save too). Both answer by sending the level again as if arriving, or for the world map a new `ForgetWorldMap` and the chunks in sight. Secret doors stay walls when revealed. Forgetting the world map also forgets which pois were visited or known. Both are refused in towns, which arrive whole.
