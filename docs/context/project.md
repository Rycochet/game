# Project

## Layout

- `Cargo.toml`: workspace root (members `crates/client`, `crates/server`, `crates/shared`, `xtask`); Bevy's default features are off at workspace level (the client turns them on, the server stays headless); shared dependency versions (`[workspace.dependencies]`) and build profiles live here.
- `crates/shared/`: library crate `shared` (plain Rust): rules and world generation. See `CLAUDE.md` table.
- `crates/server/`: package `server`, library `ServerPlugin` + binary `hall-of-worlds-server` (native only). See `docs/context/server/server.md`.
- `crates/client/`: Bevy app, package `client`, binary `hall-of-worlds` (window title "Hall of Worlds"). Native builds depend on `server` (target-specific dependency) and add `ServerPlugin` for single player; the wasm build doesn't. `src/main.rs` creates the window (bound to `<canvas id="game">` on the web) and registers each module's plugin; no game logic.
- `crates/client/assets/`: images, sounds, fonts. Loaded by Bevy's `AssetServer` natively (relative to the client crate); copied into `dist/` by Trunk for the web.
- `index.html` + `Trunk.toml`: the web build (points Trunk at `crates/client`). Serves on port 3000.
- `docs/context/`: context files for Claude (this folder).
- `xtask/`: helper commands, run as `cargo xtask <task>` via the alias in `.cargo/config.toml`. `xtask test` = fmt check + clippy `-D warnings` for native (`--workspace --all-targets`) and for `wasm32-unknown-unknown` (`client` package, which pulls in `shared`) + `cargo test --workspace`.
- `Dockerfile`, `docker/entrypoint.sh`, `.dockerignore`: the server image (see Docker below).
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

- Multi-stage `Dockerfile`, x86-64 only (`linux/amd64`; ARM can be added later under the same tags): a `tools` stage `cargo install`s Trunk 0.22.1 and static-web-server 2.44.1; `build` compiles `hall-of-worlds-server` (release) and the web client (`trunk build --release`); the runtime is `debian:trixie-slim` (same Debian as the `rust:1.99-trixie` builder, so glibc matches), non-root user `hall`.
- `docker/entrypoint.sh` runs static-web-server (web client on port 8080) and the game server together; if either exits the container stops, and `docker stop` stops both. `.gitattributes` forces LF endings for `*.sh`.
- Not testable locally (no Docker on the owner's machine); the Docker workflow builds it on every PR.

## Domains

- `hallofworlds.com`: landing page / adverts / email. `hallofworlds.app`: the owner's server running the Docker image. `hallofworlds.wiki`: game documentation (to be set up in this repo later). None live yet.

## CI

- `.github/workflows/test.yml`: on every branch push and PR, runs `cargo xtask test` on Windows, macOS and Linux. The `All tests passed` job aggregates them and is the check to require in GitHub branch protection.
- `.github/workflows/build.yml`: on push to `master` (or manually), release builds of client + server for the three OSes and the web client, uploaded as workflow artifacts (`hall-of-worlds-<OS>`, `hall-of-worlds-web`).
- `.github/workflows/docker.yml`: on PRs builds the image without pushing; on push to `master` (every merge) or manually, publishes `ghcr.io/rycochet/hall-of-worlds:latest` and `:sha-<commit>` using `GITHUB_TOKEN` (`packages: write`). Uses the GitHub Actions layer cache.
