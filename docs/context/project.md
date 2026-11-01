# Project

## Layout

- `Cargo.toml`: workspace root (members `crates/client`, `crates/server`, `crates/shared`, `xtask`); Bevy's default features are off at workspace level (the client turns them on, the server stays headless); shared dependency versions (`[workspace.dependencies]`) and build profiles live here.
- `crates/shared/`: library crate `shared` (plain Rust): rules and world generation. See `CLAUDE.md` table.
- `crates/server/`: package `server`, library (`ServerPlugin`, `start_embedded`) + binary `hall-of-worlds-server` (native only). See `docs/context/server/server.md`.
- Running a networked server locally: `trunk build` (the web client into `dist/`), then `HALL_OF_WORLDS_WEB=dist cargo run -p server` and open http://localhost:8080; a native client joins it with `HALL_OF_WORLDS_SERVER=ws://localhost:8080/ws`.
- Saved data (not in the repository): the server saves in `Hall of Worlds/server` in the user's data folder (`%APPDATA%` on Windows), or wherever `HALL_OF_WORLDS_SERVER_DATA` points; the embedded single-player server too. The client will keep its own data beside it (`Hall of Worlds/client`), never in the server's folder.
- `crates/client/`: Bevy app, package `client`, binary `hall-of-worlds` (window title "Hall of Worlds"). Native builds depend on `server` (target-specific dependency) and start the embedded server for single player (`connection` module); the wasm build doesn't. `src/main.rs` creates the window (bound to `<canvas id="game">` on the web) and registers each module's plugin; no game logic.
- `crates/client/assets/`: images, sounds, fonts. `ATTRIBUTION.md` credits third-party art (required by its licences; add an entry with any new third-party asset). `fonts/`: Lexend (the default font, `fonts` module) and Cinzel Decorative (`loading_screen`), both built into the program. `monsters/`: David E. Gervais's Dungeon Odyssey monster tiles (CC BY 3.0), 932 54×54 RGBA PNGs with lower-case names. `terrain/`: the same artist's Dungeon Odyssey terrain tiles, 960 54×54 RGBA PNGs, same treatment (`l2_terrain066`–`099` were the world map poi pictures, unused since pois became models). `fonts/`: Cinzel Decorative Bold (OFL), the loading screen's title, built into the client. `shaders/`: the world map's WESL shaders. `angbandtk/`: the same artist's flat AngbandTk tiles, three spritesheets (`dg_features32`, `dg_grounds32`, `dg_extra132`) split into 432 32×32 PNGs named `<sheet>_<row>_<column>.png`, used for dungeon walls and floors (`dungeon_map::theme`). Loaded by Bevy's `AssetServer`: native debug builds read this folder directly (`ASSET_FOLDER` in `main.rs`, an absolute compile-time path, so the exe works from `target/debug`); release builds look for `assets/` next to the executable, so every native release must ship it (`cargo xtask package` does); Trunk copies it into `dist/` for the web (and so the Docker image).
- `index.html` + `Trunk.toml`: the web build (points Trunk at `crates/client`). Serves on port 3000.
- `docs/context/`: context files for Claude (this folder).
- `tools/`: scripts run by hand: `make_town_atlas.py` (Python with Pillow) makes `assets/town_atlas.png`.
- `xtask/`: helper commands, run as `cargo xtask <task>` via the alias in `.cargo/config.toml`. `xtask test` = fmt check + clippy `-D warnings` for native (`--workspace --all-targets`) and for `wasm32-unknown-unknown` (`client` package, which pulls in `shared`) + `cargo test --workspace`. `xtask lint` = the same without the tests. `xtask hooks` points git at `.githooks/`, whose `pre-commit` runs `xtask lint`. `xtask package` = release build of client + server, then `target/package/` with both executables and `assets/` (the layout players get; used by the manual Build workflow and the VSCode "Release build (native)" task).
- `Dockerfile`, `.dockerignore`: the server image (see Docker below).
- `LICENSE`: CC BY-NC 4.0 (official legal text); `license = "CC-BY-NC-4.0"` in `Cargo.toml` for every crate.
- `README.md` (human setup, domains, game design, licence), `designs.txt` (older design notes).

## Tooling

- Rust stable, pinned via `rust-toolchain.toml` (also installs the `wasm32-unknown-unknown` target, clippy, rustfmt).
- `Cargo.toml`: dev profile builds our code at opt-level 1 and dependencies at 3 (Bevy is unusably slow unoptimised). Release uses thin LTO.
- Web: Trunk (`cargo install --locked trunk`). `index.html` asks Trunk to run `wasm-opt -Os` in release builds.
- Windows needs the MSVC C++ build tools (installed: VS 2019 Build Tools). Linux needs alsa/udev/wayland/xkbcommon dev packages (see CI).

## VSCode

- `.vscode/tasks.json`: Run/Build native, Serve web, release builds, Test (default test task).
- `.vscode/launch.json`: F5 debugs the native build via CodeLLDB.
- `.vscode/settings.json`: rust-analyzer uses clippy, format on save, separate target dir.

## Docker

- Multi-stage `Dockerfile`, x86-64 only (`linux/amd64`; ARM can be added later under the same tags): a `tools` stage `cargo install`s Trunk 0.22.1; `build` compiles `hall-of-worlds-server` (release) and the web client (`trunk build --release`, into `/srv/web`); the runtime is `debian:trixie-slim` (same Debian as the `rust:1.99-trixie` builder, so glibc matches), non-root user `hall`.
- One process, one port: the game server is the entrypoint and serves both the web client and the game's WebSocket on 8080 (`HALL_OF_WORLDS_ADDRESS=0.0.0.0:8080`, `HALL_OF_WORLDS_WEB=/srv/web`). Saves go in `/data` (`HALL_OF_WORLDS_SERVER_DATA`, a `VOLUME`, owned by `hall`): mount a volume there to keep them across updates. The server handles SIGTERM (it is PID 1), saving everyone before it stops.
- Not testable locally (no Docker on the owner's machine), and CI only builds it after merging to `master`.

## Domains

- `hallofworlds.com`: landing page / adverts / email. `hallofworlds.app`: the owner's server running the Docker image. `hallofworlds.wiki`: game documentation (to be set up in this repo later). None live yet.

## CI

- `.github/workflows/test.yml` ("Check"): only `cargo check --release --workspace` on Linux, for pull requests that are ready for review (draft PRs are skipped) and on pushes to `master` (which warms the cache PRs reuse). Cut down from `cargo xtask test` on three OSes, which was too much work per PR; run that locally. The `All tests passed` job keeps its old name because it is the check required in branch protection (what auto-merge waits for).
- `.github/workflows/build.yml`: manual only (Run workflow in the Actions tab), `cargo xtask package` for the three OSes (client, server and assets) and the web client, uploaded as workflow artifacts (`hall-of-worlds-<OS>` = `target/package`, `hall-of-worlds-web` = `dist`).
- `.github/workflows/docker.yml`: on push to `master` (every merge) or manually, builds and publishes `ghcr.io/rycochet/hall-of-worlds:latest` and `:sha-<commit>` using `GITHUB_TOKEN` (`packages: write`). Uses the GitHub Actions layer cache.
