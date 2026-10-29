# server

The authoritative game simulation. Native only, never in the browser. It runs a world (from its secret `WorldSeed`) and answers clients through `shared::protocol` messages; for now that's only welcoming them. Game state, command validation and sending maps come next (see "Map data" in `architecture.md`).

## Files

- `crates/server/src/lib.rs`: `ServerPlugin { world_seed }`, `headless_app` (no window, `MinimalPlugins` ticking at `TICKS_PER_SECOND` 20), `ServerWorld` (resource: the world's seed; `info()` is what joining clients are told), `DEFAULT_WORLD_SEED` (12345, until worlds can be created and saved).
- `crates/server/src/connections.rs`: `Connections` (resource: one `ServerLink` per client), `handle_messages` (answers `Hello` with `Welcome`, or `Refused` on a protocol mismatch; forgets clients that have gone).
- `crates/server/src/embedded.rs`: `start_embedded(WorldSeed) -> ClientLink`: single player. Test: an embedded server welcomes its client.
- `crates/server/src/main.rs`: the standalone binary `hall-of-worlds-server`: `headless_app` plus `LogPlugin`.

## Public API

- `start_embedded`, `DEFAULT_WORLD_SEED`: used by the native client's `connection` module.
- `ServerPlugin`, `headless_app`, `ServerWorld`, `Connections` (`add(link)` for a new client), `TICKS_PER_SECOND`.

## Depends on

- `shared` (`world::WorldSeed`, `protocol`). Bevy with default features off plus `std`, `bevy_log`, `multi_threaded`: no rendering, windowing, audio or input, so the Docker build stays small and needs no Linux desktop libraries.

## Used by

- Native `client` (`connection`: the embedded single-player server). The web client never depends on it (target-specific dependency in `crates/client/Cargo.toml`).

## Notes

- Embedded, the server runs on its own thread ("server") as its own Bevy `App`, so it shares no components or resources with the client: everything goes through the `Link`. The `App` is built on that thread (it can't move between threads). It adds no log plugin (the client's is used; only one can exist per process). The thread ends with the process.
- The standalone binary has no clients until networking exists.
- Will include filesystem access (persistence), which is why the web client must not contain it. Its saved data will live in its own folder, never shared with a client's.
- In Docker it runs next to `static-web-server`, which serves the web client on port 8080 (`Dockerfile`, `docker/entrypoint.sh`). Its own network port is chosen when networking lands.
- When embedded in the client, the client's full Bevy features are active too (Cargo unifies features); that's fine.
