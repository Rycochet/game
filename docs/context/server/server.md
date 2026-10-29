# server

The authoritative game simulation. Native only, never in the browser. It runs a world (from its secret `WorldSeed`) and answers clients through `shared::protocol` messages: welcoming them, sending the world map chunks (features) coming into each player's sight, and running their visits to sites (entering, stairs, leaving) while sending each level's cells as they come into sight. Movement validation and mobs come next (see "Map data" in `architecture.md`).

## Files

- `crates/server/src/lib.rs`: `ServerPlugin { world_seed, options }`, `ServerOptions { dev_tools }` (resource: whether `DevEnterSite` is accepted; only the embedded server), `headless_app(seed, options)` (no window, `MinimalPlugins` ticking at `TICKS_PER_SECOND` 20), `ServerWorld` (resource: the world's seed, its `FeatureMap` and width; `new(seed)`, `info()` is what joining clients are told), `DEFAULT_WORLD_SEED` (12345, until worlds can be created and saved).
- `crates/server/src/connections.rs`: `Connections` (resource: a `Client` per connection: its link, whether it said hello, world map chunks sent, world map position, current `Visit`, and per-level memory), `handle_messages` / `answer` (answers `Hello` with `Welcome`, or `Refused` on a protocol mismatch, anything before `Hello`, or dev tools when not allowed; hands the rest to `world_map` and `sites`; forgets clients that have gone). `TestClient` test helper. Test: messages before hello refused.
- `crates/server/src/world_map.rs`: `moved_to` (ignored inside a site: records the position and sends each chunk within `WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD` not sent yet, features as `SeenFeature`). Tests: sent once, moving sends only new ones.
- `crates/server/src/sites.rs`: `Visit` (feature, `Site`, level, its `SeenStairs`, position, cells sent), `CellMasks` (chunk → `u64`), `LevelKey` (map seed, level). `enter_feature` (the chunk's feature, if the player is within `ENTER_REACH`, the near way round the planet), `dev_enter`, `moved_to` (only walkable cells), `use_exit` (stairs to another level, or edge ground to the world map: `LeftSite`), `enter_level` (sends `EnterLevel`, then the remembered cells, then what is in sight), `update_sight` (what the player glimpses is remembered; what is glimpsed from every cell `nearby_cells` within `LEVEL_PRELOAD_STEPS` is sent if not yet), `send_cells` (secret doors as wall, stairs included). Tests: entering, sent once and remembered next time, unwalkable positions ignored, stairs and the way out, nearby cells stop at walls.
- `crates/server/src/embedded.rs`: `start_embedded(WorldSeed) -> ClientLink`: single player. Test: an embedded server welcomes its client.
- `crates/server/src/main.rs`: the standalone binary `hall-of-worlds-server`: `headless_app` plus `LogPlugin`.

## Public API

- `start_embedded`, `DEFAULT_WORLD_SEED`: used by the native client's `connection` module.
- `ServerPlugin`, `headless_app`, `ServerWorld`, `Connections` (`add(link)` for a new client), `TICKS_PER_SECOND`.

## Depends on

- `shared` (`world`, `feature`, `protocol`, `site`, `dungeon`, `surface`). Bevy with default features off plus `std`, `bevy_log`, `multi_threaded`: no rendering, windowing, audio or input, so the Docker build stays small and needs no Linux desktop libraries.

## Used by

- Native `client` (`connection`: the embedded single-player server). The web client never depends on it (target-specific dependency in `crates/client/Cargo.toml`).

## Notes

- Embedded, the server runs on its own thread ("server") as its own Bevy `App`, so it shares no components or resources with the client: everything goes through the `Link`. The `App` is built on that thread (it can't move between threads). It adds no log plugin (the client's is used; only one can exist per process). The thread ends with the process.
- The standalone binary has no clients until networking exists.
- What each client has been sent, and what its player remembers of each level, lives only as long as its connection (per session); keeping memory per character comes with persistence.
- Reported positions are TEMPORARY until movement goes through the server: `WorldMapPosition` is trusted (it decides which chunks are sent and whether a feature can be entered); `SitePosition` must be a walkable cell, but isn't checked for being a step from the last one.
- Levels are generated again on every visit (and the other level of each staircase, to work out its way down); cheap so far.
- Will include filesystem access (persistence), which is why the web client must not contain it. Its saved data will live in its own folder, never shared with a client's.
- In Docker it runs next to `static-web-server`, which serves the web client on port 8080 (`Dockerfile`, `docker/entrypoint.sh`). Its own network port is chosen when networking lands.
- When embedded in the client, the client's full Bevy features are active too (Cargo unifies features); that's fine.
