# Architecture: client / server

Read this before any work on game rules, world generation, randomness or networking.

## Principle: don't trust the user

The game is client/server, like an MMO. The **server is authoritative**: it owns the real game state and validates every command (move, attack, dig, trade) against the same rules and data the client uses. The **client is for user experience**: rendering, input, sound, UI, and *predicting* results so the game feels responsive. If the client and server disagree, the server wins and the client corrects itself.

- Clients send **intents** ("move north", "use item 3"), never results ("I am now at 10,4", "I have 500 gold").
- The server checks each intent (is the target cell walkable? is it adjacent? does the player own the item? cooldowns?) and then broadcasts the outcome.
- The client may run the same validation locally to predict the result, but treats it only as a guess.

## Crate layout (Cargo workspace)

All three exist.

| Crate            | Runs on                         | Contains                                                                                       |
| ---------------- | ------------------------------- | ---------------------------------------------------------------------------------------------- |
| `crates/shared`  | client + server (native + wasm) | Game rules: world generation, deterministic RNG, movement/command validation, network protocol types, shared components. **No rendering, windowing, audio or input.** |
| `crates/client`  | native + browser                | Bevy app with rendering, input, UI, audio; uses `shared` for prediction; talks to the server.   |
| `crates/server`  | native only (never wasm)        | Headless Bevy app (no window) running `shared` rules authoritatively; networking, persistence.  |

Both `client` and `server` are Bevy apps. `shared` is currently plain Rust (no Bevy); if shared ECS systems are needed it may depend on the non-rendering parts of Bevy only (e.g. `bevy_ecs`, `bevy_math`), which keeps the server lean.

Native clients embed the server for single player, on its own thread with its own Bevy world (`server::start_embedded`). The web client never includes server code (it uses the filesystem): it only connects to a server, such as the Docker image.

**Networking is WebSockets only**, on one port: the server serves the web client's files and the game's WebSocket (`/ws`) together, so it works behind an HTTPS proxy (Cloudflare and the like pass WebSockets) with no second port to open. One game message (postcard bytes) per WebSocket binary message; the same `Link` type as single player, its bytes carried by a `Wire`. Clients reconnect by themselves. No accounts yet: a client sends a random guest number in `Hello` (kept in the browser's local storage), which names its character; the newest connection for a character wins.

## Shared vs client-only: deciding per poi

Ask: **can this affect game state the server must validate?**

- **Yes → `shared`.** Terrain type, walkability, ore locations, settlements, anything harvestable, anything that blocks movement, anything affecting combat or loot.
- **No, purely visual → may be client-only.** Weather shading, particles, animation, decorative grass, sounds.
- **Grey area**: pure-data generation that's cheap and deterministic (e.g. decorative landscape) may live in `shared` anyway if keeping it beside related generation code is more readable. Record which choice was made in the module's context file.

## Determinism rules (required for all `shared` code)

The client and server must generate identical worlds from the same seed, on every platform (Windows, macOS, Linux, wasm), across builds.

1. **No unseeded randomness.** Never use `rand::rng()`/`thread_rng`, OS entropy, time, or Bevy's non-deterministic sources in `shared`. All randomness comes from the project's own seeded RNG/hash functions in `shared`.
2. **Use explicitly named, portable algorithms**, never "default" ones whose output may change between library versions (e.g. not `rand::rngs::StdRng`, not `std::collections::hash_map::DefaultHasher`).
3. **Position-based randomness for the infinite world.** Values for a place (cell, chunk, settlement) are derived by hashing `(world seed, purpose, coordinates)`, not by drawing from a sequence, so the result doesn't depend on what was generated first or which chunks a player has visited.
4. **Integers for authoritative state.** Positions are integer grid coordinates; game quantities are integers. Floats are fine for rendering.
5. **Careful floats in generation.** Basic float arithmetic (`+ - * /`, `sqrt`, `floor`) is identical across platforms; transcendental functions (`sin`, `cos`, `exp`, `powf`, `ln`) may not be. In `shared` code, avoid them or use the `libm` crate (pure Rust, same results everywhere). Convert noise output to integer/enum results (e.g. terrain type) as early as possible.
6. **No iteration-order dependence.** Don't let results depend on `HashMap`/`HashSet` iteration order; use `BTreeMap` or sort first.
7. **Golden tests.** `shared` keeps tests asserting exact generated values for fixed seeds/coordinates. CI runs them natively on all three OSes and under wasm, so any divergence fails the build. A change to generation output is a deliberate, versioned change (it bumps that generator's version, see "Generator versions"), never a side effect.

## Map data: what the client knows and how it learns it

Agreed design, built in steps (build order at the end). The client never generates anything secret; it learns maps from server messages and remembers them.

**Every map is a generated base plus persistent changes.** The base comes from the world seed; changes (dug walls, buildings put up, doors opened) are server state, per chunk, each chunk with a version number bumped on change.

**Seeds and world identity.** Each world has one secret world seed, kept on the server. Everything else is derived from it one way (hashed, so it can't be worked back):

- the **terrain seed**, sent to the client: surface, tides, weather. The client generates world map terrain itself; it isn't secret.
- pois and sites (and later loot, mobs) use the secret seed: the client only knows what it has been sent.
- the **world id**, public: the same seed gives the same world (and the same dungeons) on any server, so players can share worlds they enjoy. It keys any client cache, together with the generator version.

**Generator versions.** Every map generator (the world map, each site generator) has its own version, a plain incrementing number (the apps keep semver). It changes only when the generator's output changes in a way that matters (not for cosmetic changes); later, tests will check that a golden-test change comes with a version bump. The server stores the version each map was generated with: a world records its world map version, and each site level the version it was first generated with. Already generated maps keep their old version for everything that depends on it; only maps generated afterwards use the new one.

Generators are needed again after the first time, so every version still in use stays runnable in `shared`; an old version can only be removed once no stored world or level uses it:

- The world map is infinite and generated by the client too: the server tells the client the world's world map version.
- A site level is regenerated from its seed and version rather than stored whole. Generators make more than shapes: mobs and items, at first and during play. When nobody has been in a site for a while it resets: its generator spawns mobs and some items again.

**Persistence comes first.** The world is big, with no starting area to farm, so it is persistent rather than reset for loot. Changes that aren't the generator's (a player mining through a wall, breaking a door to get somewhere) are stored as an overlay on the generated base, applied whenever the map is generated, and a reset never undoes them; only the generator's spawns come back.

There are several worlds and players move between them, so every map is addressed with its world: the world map, or a site level (world id, poi chunk, level).

**The server pushes; the client never asks.** The client sends intents (step, travel, take stairs); the server validates each step and sends whatever that makes visible, plus a margin. It already knows where everyone is, so a client's "I already have…" list isn't needed (later, perhaps on reconnect with a cache).

**Three states per cell, all on the client:**

- **loaded**: sent by the server, not necessarily seen. The server sends what is visible from every cell within K steps of the player (K is `LEVEL_PRELOAD_STEPS` on the server, now 3, to become a server option; more for latency, along a click-travel route), so the client can work out sight itself with the shared sight rules: everything in sight, including the walls that stop it and the cells beside those (which decide a wall's shape), is loaded. Loaded-but-unseen cells are never drawn.
- **visible**: in sight now, worked out by the client from loaded cells (the server works it out too and is authoritative). Drawn normally.
- **seen**: remembered. Drawn in greyscale (or similar): "not now, but you remember it". Remembered floor can be clicked to travel back; the server checks every step, and memory can be out of date (a door you remember open may be closed now) until seen again.

**What is sent is what the player would see, not the truth**: a secret door is sent as wall until found; `RoomId` never leaves the server, except that a town's buildings are public (`Buildings`), as a town's whole layout is known from the start.

**Messages** (`shared`, `serde` + `postcard`):

- World map, per 64-point poi chunk: not chunk data but the changes to the public base, plus the secrets now visible to this player: the chunk's terrain changes, and its poi if known (kind, position, and per-player state such as visited or its attitude). Never the poi's map seed.
- Site levels: on entering, the level's size, theme and stairs directions (the client won't have the other level to work them out from). Cells come in 8×8 chunks: `{ level, chunk, version, mask: u64, cells }`, one mask bit per cell, only the masked cells included; a delta the client merges into what it knows. When a chunk with a newer version comes into view, its visible cells are sent again.
- Entities (mobs, items, other players) are separate: replicated each tick while visible, not kept in chunk memory.

**Knowledge kept on the server**, per character: for each site level, each chunk's seen mask (`u64`) and the version it was seen at; for the world map, the pois known and their per-player state. It is what makes playing on another machine work: entering a level sends what the character remembers of it. The loaded-but-unseen set is per session.

**Single player without a network stack.** Native clients run the embedded server in its own Bevy world (its own `App` or `SubApp`), talking to the client only through an in-process channel carrying the same message types a network transport will. They share no components or resources, even in one process, or single player would work while multiplayer broke. Tests round-trip every message through `postcard`. This also makes it easy to later open a single-player game to other players (the server accepts network clients beside the local one).

**Server and client data never overlap.** The server's saved data (worlds, generated levels, changes, characters' knowledge) and the client's data (its cache, settings) are kept in separate folders, also when the server is embedded in a native client: neither reads or writes the other's, and the client learns anything server-side only through messages.

**Cheating.** It's an open source game: by default the server doesn't try to stop hacked clients from reading what they've been sent (the loaded margin, remembered maps). The server is still authoritative over every rule. A server option for stricter play is planned (below).

**Build order:** (1) seed split, message types, in-process transport, server in its own world (done); (2) world map pois from the server, terrain still generated by the client (done); (3) site levels in 8×8 chunks with loaded/visible/seen, the dungeon view drawing known cells and memory in greyscale (done); (4) persistence on the server (done: RON files in the server's data folder: each world's seed and world map version, each entered site's generator version (one per site, so its levels agree on stairs) and per level the last visit and the overlay of changes, and per character their position, current visit, pois seen and cells seen; the player resumes where they left off). Chunk versions (for changes) come with the first thing that changes cells. Then movement through the server (done): the client sends one-cell steps (`shared::movement`, the same rules on both sides), predicts them, and the server says once a tick where it has the player, so a refused step is put right; steps carry a count of map changes so they can't land on the next map. No speed limit yet; on the world map water stops steps, following the tide by the server's clock (`Welcome`, `Hour`; `Ping`/`Pong` keep clients' running clocks in step), and ice by its snow line (`SnowLine`) can be walked on. Click travel follows `shared::pathfinding` in sites and on the world map.

**Later:**

- Client cache, in the client's data folder, keyed by world id and generator version (perhaps shared by the players on one computer): keep only the generated base there, separately from a server's changes and from what each character has seen, which the server still decides. Preload it when entering a map.
- Hack detection / prevention, chosen by the server owner (off by default; maybe on for a Steam release): smaller loaded margin, never sending unseen cells, checks on client behaviour.
- Opening a single-player game to other players.
- TLS for the server's own connections: the server generates a certificate and shares the public one with clients as part of the handshake (so a server not behind an HTTPS proxy is still encrypted, and native clients can connect with `wss://`).
