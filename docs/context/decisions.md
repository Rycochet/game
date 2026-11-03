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

Step 4 of the map data design. The server keeps its data in its own folder (`Hall of Worlds/server` in the user's data folder, or `HOW_DATA`), as RON files: `worlds/<world id>/world.ron` (seed, world map version, save format), `sites/<site>.ron` (generator version; per level the last visit and players' changes) and `characters/<name>.ron` (position, current visit, pois and cells seen). RON because people can read and fix it, it takes any map keys (unlike JSON), Bevy already depends on it, and fields added later with `#[serde(default)]` keep old saves loading; one file per site and character so saving one doesn't rewrite everything. Postgres is planned as an option (see below). A site gets one generator version for all its levels (not one per level as first planned), since its levels have to agree on their stairs. The embedded server saves and stops when the game quits (the client waits for it), plus every 5 s in case the process is killed. Characters later moved out of the world's folder (see accounts, below).

## 2026-10-29: Movement is steps, predicted by the client and confirmed by the server

The client never tells the server where the player is, only the steps they take (`ClientMessage::Step`, one cell in 8 directions, checked with `shared::movement` on both sides). It shows each step at once and keeps the unconfirmed ones; once a tick the server sends `Position` (where it has the player after the last step it handled), and the client adds its later steps and moves the player if that differs. This keeps a fast-feeling game over a network without trusting the client. Steps carry how many map changes (`EnterLevel`, `OnWorldMap`) the client has seen, so a step sent just before going down stairs isn't applied on the next level. Click travel in sites follows `shared::pathfinding` (see "Pathfinding on levels"), so steps are normally accepted; refusals come from out-of-date memory. Steps were later given time by energy (see below). Dev tools' starting position is a `DevTeleport` the embedded server answers.

## 2026-10-29: Dev tools teleports: ctrl+click and Esc

At the owner's request, since steps can no longer go through walls: ctrl+click on either map asks the server to teleport the player there (`DevTeleport`, through walls but onto a walkable cell in a site, answered with `Teleported`), and Esc puts them back where new characters start on the world map, out of any site (`DevBackToStart`). Like all dev tools messages, only the embedded server accepts them; other servers now ignore dev tools messages with a warning instead of disconnecting, so a stray key or click doesn't end the game.

## 2026-10-29: Networking over WebSockets only, on the web server's port

The Docker image had become unplayable: it ran the web client and a server with no networking. The owner's requirements: WebSockets only (no other fallback), one port for everything (the server serves the web client's files too, so it works behind Cloudflare and an HTTPS port with nothing else opened), automatic reconnecting, and "Connecting..." in red at the bottom left meanwhile. The server uses `axum` (with `tokio` and `tower-http`, MIT) on its own thread, handing each WebSocket to the Bevy world as the same `Link` single player uses (its bytes carried by a `Wire`, one game message per WebSocket binary message); the web client uses the browser's `WebSocket` directly through `web-sys`, and native clients can join a server with `tungstenite` (MIT/Apache-2.0). `ewebsock` was tried and dropped: in the browser it switches incoming messages to `Blob`s after the first send, which are read asynchronously, so messages could arrive out of order; and natively it sends at most one message per 10 ms poll. The newest connection for a character wins, so a reconnect gets it back (see accounts, below, for how a client says who it is). The Docker image runs the server alone, saving in a `/data` volume, and it saves everyone on SIGTERM. TLS for the server's own connections came later (see below). This settles the earlier open question of a networking library: `bevy_replicon` with `bevy_replicon_renet2` (server-authoritative replication, with WebSocket/WebTransport for the browser) and `lightyear` (prediction, rollback and interpolation) were the candidates; hand-rolled WebSockets won for control and simplicity with our own pushed-chunk messages, and those remain options if replicating entities (mobs, other players) gets complex.

## 2026-10-30: Thin walls, and sight that follows them

Asked for by the owner. A wall no longer fills its cell: it is a slice about 10% thick through the middle (45–55%), running out to whichever neighbours are walls, or doors and windows in line with it, so walls make straight runs, corners, Ts and crosses; with no neighbours it is a column, with one it stops 60% of the way across. It still blocks movement in the whole cell, but sight and light only where it stands (`shared::dungeon::WallShape`, in the same exact integers on both sides). This replaced shadowcasting with a straight-line check from cell centre to cell centre (still mutual, deterministic), since a shadow per cell can't express part of a cell; the server only looks again from cells it hasn't looked from, and the client only when a viewer moves, not the camera. Walls touching only at corners don't join, so sight passes between them, as stepping across such corners already could. A wall's shape depends on its neighbours, so the server also sends the cells beside what it sends, whether or not they are seen.

New cell: `Cell::Window` blocks movement but not sight. Doors and windows are exactly one cell wide, in line with walls at each side, and drawn as a wall with a hole; they block nothing in their cell, at the drawn frame either. Generators don't place windows yet.

How it looks is the client's (`dungeon_map::walls`), chosen by the theme for now and later by tile: `Built` places (stone, wood) have straight walls, square doors and framed windows with a sill taller than the lintel; `Natural` places (crypts, caves, ice) have walls curved like a letter C, narrowest half way up at the same thickness, arched doors and windows made of lines of columns, stalagmites and stalactites; more variations and material relief are planned. Dull light halved (`AMBIENT_LIGHT` 0.1, and the tokens' ambient); it still counts as light, so unlit cells show dimly rather than being unseen. Remembered surfaces are now half see-through as well as grey (owner: to tell them from grey maps, to be judged in play), and darker so memory isn't brighter than dimly lit cells. Dev tools: `DevSetCell`, by alt+click and `HOW_DEBUG_CELLS`, to try wall shapes.

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

Climbing counts on the terrain's broad height (its three largest noise layers) at a high cost, the owner's aim being that routes go round a mountain about 4 chunks across and over a much bigger one. Measured on real mountains: raising the cost on the full surface, even growing with steepness, barely changed routes, since small bumps cost the same on any route. Routes are A* within 320 points of the player (more than the view shows) rather than a coarse long-range search; an unreachable click walks to the nearest reachable point without counting as arriving. Slopes steer routes and slow steps of both kinds, by energy (see below).

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

The owner's choice: "poi" (point of interest) replaces "feature" for things on the world map, in code and docs (`shared::poi`, `Poi`, `PoiKind`, `PoiMap`, ...). "Feature" still means a game feature (feature modules) and a Cargo feature. (Done at the save wipe of 2026-11-02: nothing below remains in the code.) Three old names stayed because changing them would damage existing worlds: the seed name `derive("features")` (it would move every poi), site files `sites/feature_<x>_<y>.ron`, and serde aliases `Feature` and `features` so saved characters still load (without them a character file is put aside as broken).

At the wipe, `derive("features")` became `derive("pois")`, the site files `poi_<x>_<y>.ron`, and the aliases went.

## 2026-10-31: Pois are hidden until known, and shown as 3D models

The owner's rules: nearly all pois start invisible, and only towns are seen as the player walks around. How a character learns of the others is for later (`KnownPoi::known` is ready, nothing sets it). The server decides this and never sends what isn't known, including for the map or on entering. Woods are cut back only round pois this character knows of, so they give nothing away, and this is worked out per character rather than cached for the world. `HOW_DEBUG_REVEAL_POIS` shows them all in development, on that connection only.

Pois are small 3D models standing on the land, with no token base: placeholders built in code, since the project has no models yet, to be replaced kind by kind. They reflect the site through `shared::site::PoiLook`, which the server sends in place of anything secret: it comes from a generator's plan without generating a level, so a tower's height is its real floors and a town's size its real buildings. Claude's choice: the maps' pictures are the same triangles rasterised isometrically on the CPU when first needed, rather than baked PNGs, so there are no files to keep up to date and every look gets one. Claude's choice, not yet the owner's to judge: plain colours rather than textures from the old tiles.

**For now every poi is known (owner):** `world_map::ALL_POIS_KNOWN` sends them all to every character, as nothing teaches a character of one yet. **To do when characters get means of obtaining pois:** turn it off and set `KnownPoi::known` from those means; the hiding, entry check and per-character woods are already built and tested with it off.

## 2026-10-31: Ruined buildings, `Cell::Hole` and cave skylights

The owner's design. `BuildingOptions::ruin` (0 to 5) damages a building's finished floors with its own seed, so the floorplan doesn't change. A ruined building's floors depend on the floor below, so they are made together and kept rather than each generated alone; unruined buildings still generate per level. The ground floor loses cells to `Cell::Ground`; floors above lose them to the new `Cell::Hole` (open to sight and light, not walkable, not outside), which underground dungeons can use later. Holes are drawn as geometry, with dark through them, because the client doesn't have the level below. One cell of hole can be jumped, as liquid can (owner's). Ruined places are named "Ruined ..." (owner's), through `LevelInfo::ruined` and `PoiLook`. `for_poi` passes ruin 0: which pois are ruined is decided with poi visibility.

Generator versions: their old code was removed with the first `FORGET_BEFORE` bump (see "Starting clean in development").

## 2026-10-31: Which buildings are ruined

Each building's `ruin` comes from its poi's map seed (`Generator::for_site`), so the world map's model and the site agree. Claude's first numbers, to tune by eye: 30% of buildings are ruined, evenly from 1 to 5; tents and ice shelters never are.

## 2026-11-01: Generate POI, and "..." on entries with submenus

The owner's design: Generate POI makes a site of any kind, ruin and seed and goes straight into it, using the current biome and location for outside light, without adding it to the world map. It is a submenu listing the kinds, then a panel. Any debug option with a submenu ends in "...", and its plain name is the submenu's title. The Dropdown and Dropdown With Search widgets were kept (owner), though only Show Widgets uses them. Filtering and arrow-key movement are shared code in `widgets`, with each list drawing its own rows, as the menu's grid and a small list have little else in common (Claude's choice, agreed).

## 2026-11-01: Full-screen panels are layers

The owner's design: a full-screen panel (the debug menu, the full map) darkens everything outside it, and a second one above it is smaller and darkens again, so some areas are darkened twice. Clicks outside do nothing but close the top layer. The menu is now 90% of the window; a second layer will be 75% (Claude's number). Claude's choices: the full map keeps its own size and stands above moveable panels, so they are darkened and blocked too; the debug menu doesn't open over it, as it would be the bigger.

## 2026-11-01: Forgetting sites, and when a generator change reaches players

The owner's request: Forget Map (Server) makes the server forget a site, or on the world map every site, so they are made anew with the current generators; plain Forget Map on the world map forgets the character's memory of every site. Claude's choices, agreed in planning: a version bump alone still leaves a site as it is while the old version can run, so players' changes survive; a version that can't run any more is remade instead of refusing entry. Characters record the version they knew: their memory of a changed site is dropped, and anyone in one when it changes, or loading into one, is put out at its poi to go in again. Whoever asks for the forgetting stays and arrives on the new map, since a made-up dev site has no way back in.

## 2026-10-31: Starting clean in development

The owner's request: a way to make everyone forget everything from before a point. `shared::site::FORGET_BEFORE` is a number set in the code; bumping it makes servers ignore (and save over) characters and sites saved under a lower one, and raises every generator version with it, so all sites are made new. Clients keep nothing between connections but their guest token, so forgetting on the server is enough for them. Old files aren't deleted until saved over. The world map version isn't raised: a mismatch there refuses the world, and the world comes from its seed anyway.

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

The owner's choice, so a tablet can use the debug menu: `HOW_DEBUG_TOOLS=true` makes a standalone server accept dev tools messages, from anyone connected, so it is only for private test servers; it stays off by default, and the server logs a warning when on. `Welcome` says whether they are accepted, and the debug menu disables what would be refused. The server still checks every message.

Two more entries, the owner's: Reveal Map (a site's floor is all known, and saved so) and Forget Map (the current floor, or all of the world map, is forgotten, in the save too). Both answer by sending the level again as if arriving, or for the world map a new `ForgetWorldMap` and the chunks in sight. Secret doors stay walls when revealed. Forgetting the world map also forgets which pois were visited or known. Both are refused in towns, which arrive whole.

## 2026-11-01: Lexend as the default font

The owner's choice, replacing Bevy's Fira Mono for all text. It sits at Bevy's default font id (`fonts`), so `TextFont::from_font_size` needs no font named, and new text gets it too. It is built into the program, like the loading screen's title font, so text never waits for it. The variable font was taken over a single weight, so later code can use any weight with no more files. Bevy's own font stays compiled in, as turning it off means listing Bevy's other features by hand.

## 2026-11-01: Pings are timed where the bytes are carried

The owner asked for a more accurate ping that keeps going while the window is frozen. The number used to include the game on both sides: a frame on the client (a second apart out of focus) and up to a 20 Hz tick on the server. Now the server's network thread answers `Ping` itself, from a copy of the clock set each tick, and the client's socket (its thread on native, the browser's callback on the web) sends pings and times the answers, handing the game only the latest. These two messages are recognised from their bytes, by size first, so other messages are not decoded twice. Single player has no network, so it shows a dash. The browser has no thread for this: a busy page still delays its timing (a Web Worker could fix that, left until the owner takes up web performance).

## 2026-11-01: The server times connections itself, with WebSocket pings

The owner asked whether the server could use the ping, suggesting the client hand back its last pong with the time it held it. That number would come from the client, and the server must not trust clients, so the server times its own WebSocket pings instead, once a second, on the network thread. Clients answer in their network code, including browsers, without any game code, so a frozen game doesn't count and no message changed. The numbers in the pings can't be guessed, so a client can look slower but not faster, and latency must never earn it anything. Checked first: pings and their payloads pass through Cloudflare. For now it logs connections in trouble and drops silent ones after 30 s, as nothing timed them out before; scaling how much is preloaded can follow when needed.

## 2026-11-01: A copy of the loading screen in the page, for the download

The owner saw 5-6 seconds of black page while the wasm downloaded. `index.html` now shows the same screen saying "Downloading...", until the game has drawn its own and removes it, which also covers compiling the wasm and starting the graphics. It needs no JavaScript beyond showing the title once its font has loaded, as the owner wants no flash of another font there. The fonts come from the same server as the page, so nothing needs the internet. The cost is colours and sizes kept in two places.

## 2026-11-01: Symbols for the status text, from Noto Emoji

The owner's choice: the status text labels its numbers with symbols, and sits beside the warning button, with the mode clicks are in (in red) on its first line, shown even when the rest is hidden. Lexend has no symbols, so Noto Emoji is built in as a fallback (SIL OFL 1.1), one colour like the rest of the text. The 419 KB version 1.05 already on hand lacks World Map (U+1F5FA), so the owner chose Round Pin (U+1F4CD) for position over downloading the current 2 MB font. The snow line only shows during Cycle Temperature, and single player's ping is a dash.

## 2026-11-01: Covered ground, and stilt huts on pillars

The owner's design: `Cell::CoveredGround` looks like the land's own ground but gets no weather or outside light, for places with something built over them. Stilt huts are its first use: the covered outline with lone pillars and one ladder, and a first floor of the same outline over it. Plain ground round most sites had no outside light, so stilt huts now get the farm's, or the shade wouldn't show. Huts already saved stay single-storey until the next `FORGET_BEFORE`, through their own `STILTS_VERSION`, rather than wiping saves. Castle courtyards can use the cell once castles have them.

## 2026-11-01: Suns and moons from the terrain seed

The owner's design: a world has 1 to 3 suns and 0 to 5 moons (`shared::celestial`), which only make light. They come from the public seed, so clients work them out themselves and the server can too, later for quests (`bodies()`, `at(hours)`, with colour names), with no messages. Moon light uses real phases rather than "the brightest sun not up", since it costs one dot product. No time zones: one time of day for the whole planet, like tides. Baked things follow the light once a game hour, which the owner is happy with. Outside light in sites comes from the same source (a second PR).

## 2026-11-01: The world's clock counts minutes, on its own thread

The owner's choice: game minutes since the world began (one a real second) rather than hours, so the numbers sent and saved are easy to read; the tide, weather and sky keep their per-hour rates through `MINUTES_PER_HOUR`. It is worked out from when it was last set, so server load can't slow it, and saved every 10 real seconds on its own thread (it used to be saved only on each new hour, so short runs lost their time). Nothing passes while the server is off. Dev tools can move it either way (`DevSetTime`); anything later comparing against old times must allow for a time in the future. Save format 2 converts older clocks. Disk use is to be looked at later.

## 2026-11-01: Environment variables start `HOW_`, and logging uses `LOG_LEVEL`

The owner's convention: every variable starts `HOW_`, and those only for debugging and development start `HOW_DEBUG_`, replacing `HALL_OF_WORLDS_*`, `GAME_*` and `SITE_*`. There are no `HOW_CLIENT_` / `HOW_SERVER_` prefixes, as there are few settings and the native client embeds the server, so reads both sides' (`HOW_DATA`). Noisy logging goes at a lower level instead of behind a flag: `LOG_LEVEL`, with `debug` and `trace` kept to our crates so Bevy's lines don't bury them, and `RUST_LOG` overriding it. Both programs refused to start with a `HALL_OF_WORLDS_*` variable set until the save wipe of 2026-11-02, because ignoring the old data folder would have quietly saved somewhere else.

## 2026-11-01: One way to read boolean options

The owner's convention: an option that is off by default turns on with `1`, `on`, `true` or `yes`, and one that is on by default turns off with `0`, `off`, `no` or `false`, in any case, while documentation and examples always say `true` / `false`. Anything else leaves the default, so only the words that change an option count. `server::option_on` reads them all, as `dev_tools` is native only and so can reach it. Before this, some options wanted exactly `1` and some took any value, even `0`. Options that aren't plain yes or no keep their own values (`HOW_DEBUG_PANEL`, the numbers in `HOW_DEBUG_TOKEN`).

## 2026-11-01: Outside light in sites follows the sky

The owner's design: every site's open cells take the world map's flat-ground light at its place, once a game hour, and spill it through openings and down stairs open to the sky. Each cell takes the strongest spill rather than adding them up, and a low light stretches it along its way. No shadows from mountains, and holes don't light the level below. The server says which stairs are open to the sky (`SeenStairs::open_to_sky`), as the client doesn't know where stairs lead. Town windows light up at night, building by building, on the client only, as flavour.

## 2026-11-01: TLS on by default, with keys remembered by clients

The owner's requirements: the server secures its connections with HTTPS and `wss://` on port 8081, and the plain port only answers with an error unless told it is behind a reverse proxy (`HOW_BEHIND_PROXY=true`), which opens the plain port as well rather than closing the secure one, so a proxy can encrypt its traffic to the server. Only `HOW_DISABLE_SSL=true` keeps 8081 closed, for those who can't have it bound, and it needs the proxy option, as otherwise nothing would protect connections; it can make its own certificate, use the admin's files (`HOW_TLS_CERTIFICATE`, `HOW_TLS_KEY`) or get one from Let's Encrypt (`HOW_LETS_ENCRYPT_DOMAIN`), using its own until that comes; it gives out the public certificate on request (`/certificate`); clients check it. A certificate fetched from the server proves nothing by itself, as an attacker could answer that too, so native clients accept what the operating system's certificate authorities vouch for, and otherwise remember the public key per `host:port` on the first visit and refuse a change, as SSH does (`HOW_SERVER_KEY` names it in advance). The key is pinned rather than the certificate, and a generated key is kept for good, so renewing changes nothing for clients. Browsers do their own checks and pages can't add to them, so a self-signed server shows the browser's warning. This narrows the one-port decision above: the game is on one port, but the plain one stays open to say so.

`rustls` with `ring`, not `aws-lc-rs` (which can need extra build tools on Windows), and no OpenSSL. `webpki-roots` is avoided as its licence, CDLA-Permissive-2.0, isn't OSI-approved; `rustls-platform-verifier` uses the operating system's store. The server's TLS uses `tokio-rustls` with axum's own `Listener` rather than `axum-server`, which kept `TCP_NODELAY` and needed fewer dependencies. Let's Encrypt uses TLS-ALPN-01, needing no second port (the public 443 forwarded to the secure one), through `instant-acme` (Apache-2.0). `rustls-acme` was tried first, as it does renewals itself, but always brings in `webpki-roots`. Its website is checked with the operating system's store, so the Docker image installs `ca-certificates`. If a certificate runs out unrenewed, the self-signed one takes over, which clients that have seen the authority refuse: they would refuse the expired one anyway.

## 2026-11-01: Accounts own characters, and characters belong to the server

The owner's design: a player has an account with one or more characters. A client says `Hello`, is told the ways to log in (`LoginOptions`), and logs in (`Login`); an account is then told its characters (`Characters`) to choose from or add to, and `Welcome` follows. A guest, optional with `HOW_GUESTS` (on for now), has one character and skips that step. Accounts will log in with OIDC, and later Steam, as new `LoginMethod` and `Credentials` variants, with no UI until then; single player has no account. Login must always be secure, and TLS being on unless turned off covers servers. Clients must send real logins only over `wss://`; guest tokens may also use `ws://`, which the owner's VPN test port needs.

Characters are saved at `<data>/characters/<id>.ron`, as they will move between worlds and a server will one day run several at once (not built): the file says which world they are in and keeps what they know of each. Old character files were left behind rather than moved (the owner: nothing important is lost).

Logins are kept apart from accounts (`logins/<hash>.ron` → account id), so that several ways in can lead to one account, or a guest could become a full one, and no secret is stored. Guest tokens are 32 secure random bytes, one per server per client, so a server can't use one on another. One account plays one character at a time, the newest connection winning, so items can't be passed between its own characters later. Names are 1 to 24 characters and not unique, to be looked at again when players can see each other.

## 2026-11-01: Pausing is single player only, and the server stops the clock

While the system menu is open, the client asks the server to pause and only stops once told. Only a server with `ServerOptions::pausable` obeys: the embedded one, not any networked one, whatever its dev tools setting, and turning on play with others must turn it off. Pausing stops `WorldClock`, and on the client `ServerClock` and `Time<Virtual>`; ticks, saves and pings carry on, with the pong's clock ignored while paused. Anything going by game time must go by these, so it stops too.

## 2026-11-01: User documentation is Jekyll in `docs/`, kept in step by a rule and tests

The site at `hallofworlds.wiki` is Markdown in `docs/`, built by GitHub Pages' Jekyll with the Just the Docs theme (MIT; sidebar and search with no code of our own) and deployed by a workflow, with `docs/context/` left out. It stays simple while the game changes quickly. `CLAUDE.md` says which changes must update which page; the server variables and debug menu entries (the owner's two must-haves) are also checked by tests, which read the pages.

## 2026-11-01: UUID v6 ids, items in grids, and bags that nest within limits

The owner's design, the first of the planned creatures work. Ids are version 6 UUIDs (`shared::id::Id`, `uuid`, MIT/Apache-2.0), unique and kept for good whatever they are for, rather than counters: items and later mobs use them, and existing ids are converted by separate tasks. Only the server makes them, with random bytes in place of a network address, so they say nothing about the host. An item keeps its id wherever it goes.

Items have a weight and a shape, and go in grids, turned by quarter turns (`shared::item`). Bags can nest because a backpack's grid alone holds too few rings to be believable, so a 2 × 2 pouch can hold a 4 × 4 grid. The owner's limits: only a smaller shape than the container's outside goes in (always), at most 3 grids deep, and per kind of container the most it holds in all and the most for each thing. Each step in having fewer cells is enough to stop endless nesting; the depth limit keeps the panels manageable.

Handling is the owner's: an item pressed on is held by the cursor at its true size until let go, and anywhere it can't go it snaps back. Holding is only on the client and a drop is one `MoveItem`, so nothing can be lost part way. Claude's choices: turning by right-click, and by a tap with a second finger (agreed to try and adjust after testing); pockets of 6 × 4 until creature kinds exist; every `MoveItem` answered with the whole `Inventory`, as it is small; dropping on a closed bag uses the first place it fits. Items are coloured cells with letters until there is art, and the kinds and weights are placeholders. What is carried belongs to the character across worlds and isn't forgotten by `FORGET_BEFORE`, which is about maps. The owner added a bin in the pockets' panel: dropping anything on it, from any grid, destroys it and its contents, with `DestroyItem`. There is no requester (Claude's choice, as it goes red and says so first), to look at again when items can be dropped on the ground.

## 2026-11-02: Saves go through a save thread and a backend; Postgres is planned for later

The owner asked what Postgres would take, as an option beside RON files, and chose relational tables rather than blobs, so the data is useful to query and not only safer. A server would choose when it is created, with a separate tool to migrate either way. That waits until the data schema has matured: `postgres-migration.md` is the plan, and RON stays, as the same Rust save structs are what keep the two in parallel.

What was worth doing now protects file saves too. The game hands whole save structs to `SaveBackend` (`saves.rs`), through one save thread (`save_queue.rs`) that keeps the latest version of each until it has really been written, so a failed write is no longer forgotten and writing doesn't hold up a tick. Loads look in its queue first, as reconnecting saves a character and loads it straight away. A save that can't be reached is no longer treated like one that is broken: only what was read and makes no sense is put aside and started again, and a player whose save can't be reached is refused, because a locked file used to mean a blank character saved over a good one. `ServerWorld::open_site` loads a site where someone can be refused, so `site` needn't fail.

## 2026-11-02: Creature kinds and clothes by body, layer and size

The owner's design. Players and mobs share kinds of creature, from a table, each with a body: which locations it has, which layers each takes, and how many places. A kind's size is separate from its appearance. An item has one layer and its locations, and fits if the body has them all at that layer, so a chicken can wear trousers but not a T-shirt; different bodies use different locations (`Forelegs`, `Wings`) rather than a compatibility list. Nothing can go on or come off under something on a higher layer, which both the panel and the server check. Rings are locked under gloves and gauntlets unless the gloves are fingerless, and gloves and gauntlets need exactly the wearer's number of fingers, for creatures with other numbers (mitten gauntlets excepted, see Hand armour); gloves are on the middle layer, not skin as first described, or a ring could never be under one.

Sizes are on the item, with a size it fits and a range it can be worn in (the owner's two values), loose outside its own size unless it adjusts; magical items fit any fingers and sizes within reach of their quality, but not any body. Loose has no penalty yet.

Dragging on and off is the same holding as in the grids. Claude's choices: a panel of rows and layers, opening with the pockets, since there are no pictures of bodies; the creature kept on `Inventory` for now; existing characters get the starter clothes once; Change Creature loses what doesn't fit, being a debug tool. Pockets now come from the kind.

## 2026-11-02: Stats, skills, abilities and looks start from the kind

The owner's design: every creature kind has base stats, skills and abilities used by gameplay and mob generators, and appearance that only changes how it looks. Each character has a `Sheet` starting from their kind. Claude's choices: Strength, Dexterity, Intelligence and Constitution from `designs.txt`, agreed in planning; abilities only from the kind for now; which kinds are playable; all the numbers. What can be carried is strength × (size + 1) kg, burdened past half and overloaded past all of it, shown on a new character panel, which energy then turns into speed. `CreateCharacter` takes a kind and look, checked by the server, ready for a creation menu. Appearance only shows as words: tokens are fixed pictures, and how they vary is left for when entities have looks.

## 2026-11-02: Energy for actions, by formula rather than by loop

The owner's design, from a roguelike scheduling article (the concept, not its code, as that was for a synchronous loop): a creature gains its speed in energy each tick, acts with 0 or more, and an action's cost can take it below 0, so creatures of different speeds really do move at different speeds, and other actions can cost time too. The owner asked for the last time to be stored rather than looping over creatures, which also lets things pause: energy is worked out by formula, so idle creatures cost nothing. It stops just under a tick's worth, so waiting saves nothing up.

Online, intents arrive whenever they do, so ones without the energy wait on the server instead of being refused: network jitter only delays them, and only too many are dropped, which is what stops a hacked client going faster. Entering and leaving wait too, or an exit could overtake the steps leading to it. A whole-number `ActionClock` is saved and stops with pausing but dev tools can't move it. Claude's choices: today's pace as normal speed, to be judged separately; speeds by kind; burdened 75% and overloaded 40%; dressing costs by weight, on the server only; energy isn't saved. A schedule of who acts next comes with mobs, as there are only players to look at so far.

## 2026-11-02: Mobs and other players on the map

The owner's design: players and mobs are the same kinds of creature, with mobs only controlled by the server. The server keeps who is on each site level in use, shared by everyone on it, and sends each player only what is in their sight: kind, look and place, and never stats. Mobs come from a hash of the site's secret seed, apart from level shapes, so no generator version changes; each is given a UUID once, saved against its number with the level, so it can carry memories later. A level nobody is on stays for a minute for reconnecting. Claude's choices: other players use the same messages, on the world map too within sight; who lives where and how many; mobs stay in sites for now, as agreed; they stand still until behaviours; a player's id lasts the connection until characters have UUIDs (a queued task).

## 2026-11-02: Mobs have a current action, and a level runs until they settle

The owner's design: a mob has a current action, and a level stays running until all are back at a neutral one, even with nobody there. That also gives mobs time to wander away from the stairs and go home, so they don't become an ambush that someone waits an hour for. It replaced Claude's first idea of freezing an empty level. Only mobs not at rest are on a level's schedule; one at rest needs nothing until it notices someone. Actions so far are going after someone, fleeing, settling and going home; attacking and healing come with combat. Every action must end: if going home takes too long, a mob gets there once nobody can see. Attitude comes from how kinds feel about each other (a kind likes its own), and each kind's reaction to that; tokens show it to each player. Mobs know the whole map, as the owner suggested, but only notice someone they can see; secret doors stay walls to them, or a mob walking through one would give it away. Claude's choices: all the numbers, who dislikes whom, and that mobs can share a cell until there is combat.

## 2026-11-02: Mobs follow on stairs and come home

The owner's design: mobs can chase up and down stairs, and a floor runs until its mobs have settled. A mob follows only if it saw the stairs taken, and if its kind can use stairs; it remembers the way back, and its home level stays in use until it returns. It stops at a site's edge, as mobs stay in sites for now. Mobs not at rest are saved with their level and carry on, so quitting single player can't end a chase. Changed from the plan, Claude's choice: they carry on when the level is next used, rather than the server listing such levels and running them from starting, as time doesn't pass while it is off and that would mean generating sites nobody is in; a mob caught on another level starts again from home. On a networked server, logging out still counts as being lost: to look at with combat.

## 2026-11-02: Any mob can remember, so any can be an NPC

The owner's design: every mob can be an NPC, learning things and remembering them, so a beggar given money feels differently from then on. Memories are facts about characters rather than words, so that the game can act on them, and each moves a scale of warmth from where its kind's feeling starts. They are kept against the mob's id, with what it was given, saved with the level it belongs on, so only mobs someone has dealt with cost anything. Claude's choices: 30 warmth for any gift, whatever it is; 32 facts kept per mob; a new Beggar kind, a few in each town, as the owner's example needed someone to give to; giving by dropping on a name in a Nearby panel, within 2 cells; a gift can't be taken back. Dialogue, trade and what mobs do with what they carry are for later.

## 2026-11-02: UUIDs for ids that are made or worked out, and a save wipe

The owner's request, waiting until mobs were in. Ids the server issues (items, mobs, characters, accounts, and the single player's character, made on first run through a `LoginKey::SinglePlayer` account rather than a new kind of file) are version 6 UUIDs. A `WorldId` is a version 5 UUID of the seed under a fixed namespace, the owner's idea: a hash, so the same seed is still the same world on any server; the seed stays in `world.ron` and never reaches a client. Other players are seen by their character's id, so `Client::entity_id` lasts between connections.

Staying as they are, because a UUID wouldn't suit them: `RoomId` and building ids (small numbers both sides generate), `SiteId` (worked out from the place) and touch ids (the operating system's).

The ids change every file name, so the owner chose a wipe over converting saves, and took the waiting jobs with it: the old "feature" names, `refuse_old_variables`, and code only for older saves (`hour`, `serde(default)` on what is saved, which the rule in `storage` brings back field by field as saves grow). This also moves every poi. Saves are moved to `old-saves-<date>/` rather than deleted, so a wrong `HOW_DATA` can't destroy anything, and certificates stay, so a wipe can't take a server's HTTPS down. `save_format.ron` says what a folder holds; `OLDEST_SAVE_FORMAT` is what a later wipe raises. The format file and moving are `FileBackend` matters, not part of `SaveBackend`.

## 2026-11-02: One to a cell, and the first creature tweaks

The owner's changes after trying the creature work. Mobs can't stand on a cell with another mob or a player. A player stepping onto a friendly or neutral mob swaps places with it, and can't step onto an enemy, so routes go round enemies. Mobs share only a level's stairs by waiting. Character, Clothes and Pockets are one Character panel, in that order. The bin is an item slot: what is dropped there stays until something else replaces and destroys it. Detect Monsters is a debug toggle showing every mob on the map. Moveable panels keep their share of the window rather than their pixels. Giving stays as it is, to become a trading panel. Claude's choices: other players still share cells; what is in the bin weighs nothing; one person button, with C and I both opening the panel; a mob on a cell someone arrives on by stairs or teleport moves aside.

## 2026-11-02: The edge of sight is drawn as real lines, and only drawn

The owner asked for line of sight drawn with actual lines rather than tile by tile, keeping the server's block-based sight. So nothing in the game's rules or what is sent changes: `Sight` stays on the grid, and the client works out polygons from the same wall bars (`Level::wall_bars`), in floats, as they affect only the picture. They are drawn into a mask the floors' shader reads, rather than cutting meshes, which rebuild every frame the camera moves and would be slow and complicated to cut. The mask covers what the camera shows on every level, the owner's choice, so big buildings cost the same as anything else. Claude's choices: the lines follow the smooth position; memory stays per cell; light stays on the grid. Walls, doors, stairs, towns and plants are cut too. Tops of walls and roofs stay on the cell rule, as sight stops at the wall below them, so a line can't say whether they are seen.

## 2026-11-02: Item quality, and what is planned on it

Owner's design; only quality, its colours and rolls, and when items were made (`MadeAt`) are built so far. Mobs' gear and shops follow. Numbers are first guesses for tweaking.

- **Quality:** poor (grey), common (white), uncommon (green), rare (blue), epic (purple), legendary (orange), artifact (gold). Better is rarer. Some kinds have none. Quality will decide base stats and how many bonuses an item can have. Drops are two rolls: quality, then the item. Artifacts are named.
- **Mobs** carry and use what they are generated with, and that is what a player can loot, steal or buy, by attitude and skills.
- **Set items** can be of any quality, shown by a metal edge rather than a colour of their own, and are always named and part of a named set (2 to 7 pieces), with bonuses for wearing more, mostly better than other bonuses. They are never rolled: one comes at the end of a quest's story. Each planet has one, chosen from its seed, so players may later seek out planets. A character finds one per planet, for good; after that any other is a secret from them. That matters when a group finishes in the same place: each who hasn't found it sees their own. Being part of a set is a flag (`Item::set`), with no sets or names yet. The edge's diagonal bands go by screen position, so all set items match, and slide with turning, tilting, zooming and walking, eased rather than tied directly (owner). Text can't use a UI shader, so the name stays in its quality's colour and its box gets the metal edge; kinds without quality can't be set items (Claude's choices). Claude's assumptions: set items fit any wearer, can't be made or improved, and a character who has found one can't be given another from that planet.
- **Improving:** resources and time raise quality, each step harder, in step with the bonuses it holds. Legendary to artifact is a questline on an existing item: many hours, other planets, exact actions at exact times, and a mistake restarts it. The item keeps its id. Claude's assumptions: any item with quality can climb the whole way, and a mistake costs progress, not the item. Artifacts can also be rolled, rarely, with placeholder names.
- **Making:** shops, NPCs and skilled players make anything but set items, and artifacts only by that questline. Items store when they were made. The time to make one depends on quality and kind (a lot) and the shop's quality.
- **Shops** in towns keep stock, sized by shop quality, with odds biased by quality and trades, and may take commissions for a kind of item at a higher cost. Stock is caught up on opening from the time the last item was made: quality rolls fill the time, then only what remains is rolled as items, with a limit past which everything would have been replaced. Selling is by a gentle curve (a legendary half as likely to go as a common; owner chose this over twice its make chance after a simulation filled shops with legendaries). Several shops of a kind in a town each work √n faster and sell √n faster. A shop reopening below its minimum after time away is topped up to close to its count. A shop's panel opens on clicking its door when adjacent, or on arriving after clicking it from further away.

## 2026-11-02: Shops have hidden keepers, and trading panels belong to an Id

Owner's design. Every shop has a keeper: a mob with an `Id` of its own, in the level's `records`, that is never on the map or in the live mob list. Its `carried` is the shop's stock and its memory is the shop's. Quality, trades and restock times stay with the building (`BuildingSave`). Opening a trade names an `Id`, so an NPC trader will use the same messages, and there is never more than one panel per `Id`. Claude's choices: keeper ids are made when a town level is first sent and sent as `ServerMessage::Keepers` (generation stays pure, and the client knows who to ask for before asking); the client module became `trade`. The grid's order isn't kept: it is worked out at each opening (owner). Selling is giving for now (owner), with more to come on it: what is dropped on a grid joins the stock. Taking is free too, for now (owner): stock can be picked up, moved within its grid, or taken to the pockets or a bag. Claude's choice: wearing it or binning it straight from a shop snaps back, so it goes into a grid first. Claude's choice: dropping on something already there isn't refused; it goes in the first free place, shown in green first.

## 2026-11-02: God mode is one debug flag for everything it comes to do

The owner asked for a God Mode toggle, with more to be added to it later. It is one dev tools flag (`DevGodMode`, `Client::god_mode`, `common::GodMode`), per connection and not saved, which new powers check rather than having messages or menu entries of their own. For now mobs take no notice of the player, and the player walks through them. Claude's choice: walking through swaps places, as with friendly mobs, so only one still stands on a cell.

## 2026-11-02: An item's size sets how many cells it takes

Owner's rule, the first step towards generated 3D items. Each side of an item's shape is multiplied by its size's value, rounding down when enlarging and up when reducing, so everything 1 × 1 stays so until Giant, which is 2 × 2. An item's picture and its place in a grid look the same at any size; only its name, and later the item popup, say so. Claude's choices: the values are the client's token sizes (0.5, 0.75, 1, 1.25, 1.5, 2), with shared `Giant` as the top one the owner called titan; sides stop at 8, as much as a `Shape` holds, so only a Giant sword is cut short (2 × 8); gaps scale in proportion. A bag's grid, its limits and all weights stay as they are for now, as scaling containers is to be decided, and bags aren't rolled at other sizes until it is. Saved items that now overlap are left as they are, being harmless, until an item wipe once the item plans are finalised.

## 2026-11-02: Generated items: forms saved on the item, and 3D pictures

Owner's design: each kind has one generic model drawn differently by values that come from the seed, drawn once to an image rather than in 3D each frame. Rings and gems only, until other kinds have been researched: the generic name isn't enough, as a sword's measurements decide whether it is a rapier or a katana, and so its name.

- **Values are saved, not worked out again.** Material will affect value, so the server rolls an item's form once (seeded from its `Id`) and saves it (`Item.form`). Changing the rolls never changes items already made. Claude's choice, against working it out from the seed each time as first discussed.
- **Rolls are ordered choices.** Each sees those before it, and can lean on one by weights or limit it with a weight of 0 (owner). Quality stays the first roll, as shops need that order, so better rolls lean towards finer materials (Claude's choice): a gold ring is still more often a good one, and making an item can roll the other way when that is built.
- **Pictures are kept by what they show** (`item_art::Look`), not by seed or id, so a change to what can be seen makes a new picture and others keep theirs. They are kept in memory only: a disk cache was planned, and dropped after measuring (`optimisations.md`), as drawing is quick enough.
- **Gems are real items** set in rings' settings (planned), so they can be taken out, replaced or added to, and a setting can be empty (owner). Cuts are round, oval, emerald, cushion, princess, pear, trillion and uncut (owner); only clear stones for now, as opaque ones are usually cut as domes, which isn't one of these. Uncut gems won't go in settings (Claude's choice).
- **Size stays out of pictures:** see the size entry above.

All built: size rule, gems, ring values, ring model, pictures in grids.

## 2026-11-02: Gems set in rings are whole items

Owner's design: a ring's settings each hold a whole gem item with its own id, or nothing, and a setting stays when its gem goes, so gems can later be taken out, replaced or added, and settings added. A ring may have an empty setting when made, rarely, and several gems strongly favour the same stone. A setting takes a gem of its size, smaller, or one size up (owner: ugly, but a thing people do). Claude's choices: gems sit inside the ring's form rather than its `contents`, as those mean a grid to open; a gem matching the first copies its cut too; a ring's quality is how it was made and doesn't change with its gems, each of which has its own; a size given is used as it is (a wearer's or a mob's), and only shop stock rolls one; `Inventory::find` doesn't see set gems until something can move them.

## 2026-11-02: 3D models have black outlines

Owner's rule: any 3D model gets a black outline unless told otherwise. Bags are the first built this way, with a wider one while hovered. Poi models, outlined only while hovered, are older than the rule.

## 2026-11-02: Items on the ground

Owner's design. Items can lie on any cell of a site's levels, never on the world map, in one unlimited pile per cell saved with the level and shared by everyone on it. They stay until taken. Only someone standing on the cell is told what is in a pile; dropping on the Drop icon or the ground section, and taking to grids, clothes or the bin, are all checked by the server and wait behind the player's steps, as where they stand decides what they mean. Moving something to a different container takes time by weight (10 kg is one straight step, to depend on stats later); sorting within a grid and throwing away are free. What is on the ground shows from afar only as a bag, by how full it looks. Characters remember how bags looked when last seen, saved, and aren't told anything else until they see the place again, so a remembered bag may be gone. Mobs dropping their gear follows.

## 2026-11-02: Weapons from size grids and named rows

Owner's design. Knives to polearms aren't a list of kinds: a weapon has a blade and handle length and a set of properties, the grids in `shared::blade` say which sizes may be made at all, and its name comes from what it is like: the most specific that fits, or else a generic name for its size. Names are limited to those people know, using the Google Books Ngram Viewer, about 5 to 10 a family. Scythes and sickles are left for tools later. Research: `research-bladed-weapons.md`.

## 2026-11-02: UI design guide and shared widget rules

`docs/context/ui-design.md` is the palette for planning screens, from the owner's guide based on Material UI, and is updated as needs change. The owner chose, piece by piece, what to take: the game keeps its parchment, raised edges, square corners and Lexend, with one look (no dark mode). Text comes in four named sizes rather than numbers, gaps are multiples of 4 px, and moveable panels are told from static ones by lighter stripes. Ripple, hover and pressed states are planned for all clickable widgets; stacks and gamepads are separate later tasks.

## 2026-11-02: Scaling weapons, and requests with limits

Owner's rules. Whoever asks for a weapon can give limits on its cells and weight, and the generator keeps to them without rolling again, or makes nothing. It first takes an ordinary-sized weapon that fits, so a tiny creature usually gets a dagger of some form. Otherwise, or now and then, it scales one, whose size and weight both change, and which keeps only its generic name ("Giant Sword"). A named weapon at another size ("Tiny Katana") only comes from asking for exactly that, as a shop commission or a player's ability might. Each family is its own `ItemKind`, so saved swords stay as they were and no wipe is needed.

## 2026-11-02: Most items come in one size

Owner's rule: all wearables come in sizes, and everything else is `Medium` for now, whoever carries it, so a giant's coin is an ordinary coin. Claude's choice, agreed: gems keep their rolled sizes, as ring settings depend on them, and blades scale by the rules above. Items already saved keep their sizes until the item wipe.

## 2026-11-02: Magical, blessed and cursed items are secret until found out

Owner's design. Only things worn can be any of these. Magical needs better than common, and fits perfectly within a reach set by quality (uncommon 1 size each way, one more per quality), with any fingers, going by its usual range beyond that. An item is blessed, cursed or neither, and either can go with magical. Blessed things can only be taken off by their wearer; cursed things by nobody, until an ability exists. God mode takes a player's own off without limits. All three stay secret until found out: a curse by trying and failing to take it off, magic by fitting when it would otherwise not have, and nothing yet for a blessing. Truesight (debug) shows them and secret doors, and will show anything else hidden. Secret doors stay shut under it, as finding them comes later. Claude's choice: one shows as a pale tinted wall, and as a door on the full map, rather than a door model, as it can't be walked through.

Claude's choices: the server doesn't send a secret at all (`Item::hide_secrets`) rather than trusting the client to hide it. The rule for taking things off is asked by the player's own actions rather than built into `move_item`, as the owner's wording leaves room for disarming and theft, which would have rules of their own. So a killed mob still drops what it wore. Found out is per item, for everyone from then on, not per character. A curse holds only while worn. Layers still apply in god mode. Since a client can't know what would fit by magic, it asks the server about anything refused only for size or fingers, so those no longer show red. 5% each for magical, blessed and cursed, as first guesses. Glows are one shader working from an item's picture or cells, so blessed and cursed cost one colour each, with no extra pictures to keep. They stay inside the item's frame (owner): round a picture, or in from the edges of plain cells.

## 2026-11-03: Armour: sets of pieces, and the body rules they need

Owner's plan. Armour is researched and modelled one body location at a time (Head, Neck, Shoulders, Arms, Torso, Waist, Hands, Legs, Feet), and multi-location sets are single items built from those pieces, all of one material, quality and size, and slightly better than the same pieces worn separately once stats exist. Each piece's model is built in place on a standing body, so the same model makes its inventory picture and, later, the character's own picture showing everything visible that it wears. Creature armour is researched alongside but made later. New materials found (general category and specific type) are listed for the owner to decide on, and recorded here once agreed. Research and plan: `research-armour.md` (to come with the first models).

Body rules, owner's decisions: bipeds' Waist takes Outer (faulds, tassets), keeping belts Middle. A new Back location, with two places and Outer only, holds a backpack and a cloak, which both moved there, so a cloak goes over armour. Backplates are part of Torso armour, so armour never stops a pack. Head and Neck take Middle (arming caps, coifs). Gambesons are Middle items of their own, but a full set comes with its padding: `WearDef::padded` lists where an item takes Middle as well as its own layer. Claude's choice: Full Armour is padded on its shoulders, arms and torso, not its legs.

Owner's answers to the research questions: a new `Over` layer above Outer for cloth over armour (surcoats, tabards), and horses' caparisons use it over barding. A piece that only half covers a location doesn't count for it (a knee-length hauberk isn't Legs). Style (Gothic, Milanese, elven…) is a property every piece of a set shares. Fantasy materials go under the ordinary categories, with no Fantasy category: mithril is Metal, dragon scale and chitin Bone. Ring mail and studded leather stay. A Plant category holds Rattan and Cork, for armour and anything else later. Claude's choices: Over is taken by a biped's Shoulders, Arms, Torso, Waist and Legs and a horse's Head, Neck and Torso; Surcoat (Torso, Waist) and Tabard (Shoulders, Torso) at tailors, Caparison (Head, Neck, Torso) at armoursmiths.

Claude's choices for the body frame (`shared::body`): every kind's frame is at an ordinary person's 1750 mm, as blade lengths are, with only its proportions its own, and an item's size goes on top. Arms hang a little out from the sides so that sleeves and gauntlets show. Bodies are drawn a little turned and from slightly above, as a token shows a character. `shared::armour::Construction` holds the eleven constructions from the research, so every location's pieces use the same list.

## 2026-11-03: Hand armour, armour materials and styles

Owner's decisions on the Hands research (`research-armour-hands.md`). Mitten gauntlets fit any number of fingers; fingered ones still need the wearer's count. Demi-gauntlets (Outer, hands only, so rings stay free) and Padded Gloves (Middle, under gauntlets) are kinds of their own beside Gauntlets. Items are pairs, so single gauntlets (the manifer, the bridle-hand gauntlet) are dropped. Rings are always against the skin, locked under gloves and gauntlets. A style narrows what plate may roll, so each looks like itself. Claws and knuckle spikes will give damage bonuses in unarmed combat.

Approved materials, added with the hands as the first location to need them: Steel, Bronze, Brass, Mithril and Adamantine (Metal); Coir (Plant); Horn, Dragon Scale and Chitin (Bone); a Leather category (Leather, Hardened Leather, Buff Leather) and a Textile category (Linen, Wool, Cotton, Silk). Claude's choice: rings keep rolling from copper, gold, iron and silver only, so the new metals don't change them. `shared::armour::Style` is the one list of styles (White, Gothic, Milanese, Maximilian, Greenwich, Kastenbrust, Elven, Dwarven) for every location, as the parent asked.

Claude's choices for the models: each hand is built in place on the frame, hanging from the wrist with its palm to the thigh; the frame gives only the hand's length, and `armour::hands::Hand` works out the rest from it. A gauntlet's cuff sits 28 mm out from the forearm, over a vambrace, reaching up to 200 mm (agreed with the Arms chat). The picture lays both hands down, fingers to the right. Gauntlets get a form, and so a rolled weight, from now on; gauntlets already made keep their old picture and weight.

## 2026-11-03: Shoulder armour

Owner's decisions on the Shoulders research (`research-armour-shoulders.md`). Shoulder pieces come as one item per pair (`ShoulderArmour`, Outer on Shoulders), and a pair may be uneven. Fantasy adornments (spikes, horns, a beast's head, skulls) can go on any style, with more options later. Ailettes stay, painted with simple heraldry. The Chukchi wing stays. The Gambeson covers Shoulders as well as Torso and Arms, so there are no separate padded shoulder pads. Japanese, Roman, Russian and Bronze Age pieces have their own names, others generic ones.

Claude's choices: a form says the pair's kind (pauldrons, spaulders, shoulder guards, cape, discs, ailettes, wing), so its name, cells and weight come from it. Besagews and gardbraces are parts of pauldrons and spaulders, not items. Edges agreed with the Torso, Arms and Neck chats: rigid shoulder pieces lie 35 mm out from the skin, outside the breastplate and gorget; lames end about 130 mm down the arm, over the rerebrace; hanging guards are loose outside anything on the arm. `Heraldry` (two tinctures, metal on colour, and a charge) is its own type, for shields and surcoats later. The armour model helpers (`Stuff`, `Model`, `plate`) moved to `item_art/armour/mod.rs` for every location, and `item_art/body.rs` gives the trunk's surface (`trunk_half`, `trunk_depth`) for armour to keep clear of.

Owner's decisions from the Waist review, made in the Shoulders PR for every location: armour weighs by the square of its size, not the cube as blades do, as its area grows and its thickness stays (`grams_at`, hands and shoulders). `Style` also holds regional looks, Japanese, Chinese, Steppe, Classical, Byzantine and Indo-Persian, added at the end so saved names stay stable, and each construction is made only in some styles (`Style::allows`). Claude's choices: padding goes with any style and the fantasy styles with anything but mirrors; a kind of piece whose constructions a style allows none of is made in its usual ones (`in_style`). Japanese shoulder guards, Classical doublings, yokes and Dendra and Roman spaulders now come from the style rather than from flags of their own.

## 2026-11-03: Client preferences, and Item Pictures by page

Owner's request: Item Pictures shows one kind of item at a time, chosen from a dropdown, and remembers the last choice; the gems' cut names head only the Gems page. Claude's choices, agreed in the plan: the choice is kept between runs in `common::Preferences`, the client's one file of the player's choices (`client.ron` beside the known servers, or `HOW_PREFERENCES`; the browser's local storage on the web), which later choices such as interface scale join. It is saved whenever it changes, and anything missing or unreadable falls back to the defaults, so fields can be added without a wipe. A page is kept by its name, so reordering pages loses nothing. Only the chosen page's pictures are drawn.
