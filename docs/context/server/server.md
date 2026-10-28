# server

The authoritative game simulation. Native only, never in the browser. For now it only starts up and logs; it gains the game state and command validation when networking is added (see `architecture.md`).

## Files

- `crates/server/src/lib.rs`: `ServerPlugin` (currently a startup log line), `TICKS_PER_SECOND` (20).
- `crates/server/src/main.rs`: the standalone binary `hall-of-worlds-server`: headless Bevy (`MinimalPlugins` with a fixed-rate `ScheduleRunnerPlugin`, `LogPlugin`) running `ServerPlugin`.

## Public API

- `ServerPlugin`: added by the standalone binary and, for single player, by native clients (`crates/client/src/main.rs`, behind `#[cfg(not(target_arch = "wasm32"))]`).
- `TICKS_PER_SECOND`.

## Depends on

- `shared` (declared; not used yet). Bevy with default features off plus `std`, `bevy_log`, `multi_threaded`: no rendering, windowing, audio or input, so the Docker build stays small and needs no Linux desktop libraries.

## Used by

- Native `client` (embedded single-player server). The web client never depends on it (target-specific dependency in `crates/client/Cargo.toml`).

## Notes

- Will include filesystem access (persistence), which is why the web client must not contain it.
- In Docker it runs next to `static-web-server`, which serves the web client on port 8080 (`Dockerfile`, `docker/entrypoint.sh`). Its own network port is chosen when networking lands.
- When embedded in the client, the client's full Bevy features are active too (Cargo unifies features); that's fine.
