# Project

## Layout

- `Cargo.toml`: workspace root (members `crates/client`, `crates/shared`, `xtask`); shared dependency versions (`[workspace.dependencies]`) and build profiles live here.
- `crates/shared/`: library crate `shared` (plain Rust): rules and world generation. See `CLAUDE.md` table.
- `crates/client/`: Bevy app, package `client`, binary `game`. `src/main.rs` creates the window (bound to `<canvas id="game">` on the web) and registers each module's plugin; no game logic.
- `crates/client/assets/`: images, sounds, fonts. Loaded by Bevy's `AssetServer` natively (relative to the client crate); copied into `dist/` by Trunk for the web.
- `index.html` + `Trunk.toml`: the web build (points Trunk at `crates/client`). Serves on port 3000.
- `docs/context/`: context files for Claude (this folder).
- `xtask/`: helper commands, run as `cargo xtask <task>` via the alias in `.cargo/config.toml`. `xtask test` = fmt check + clippy `-D warnings` for native (`--workspace --all-targets`) and for `wasm32-unknown-unknown` (`client` package, which pulls in `shared`) + `cargo test --workspace`.
- `README.md` (human setup + game design), `designs.txt` (older design notes).

## Tooling

- Rust stable, pinned via `rust-toolchain.toml` (also installs the `wasm32-unknown-unknown` target, clippy, rustfmt).
- `Cargo.toml`: dev profile builds our code at opt-level 1 and dependencies at 3 (Bevy is unusably slow unoptimised). Release uses thin LTO.
- Web: Trunk (`cargo install --locked trunk`). `index.html` asks Trunk to run `wasm-opt -Os` in release builds.
- Windows needs the MSVC C++ build tools (installed: VS 2019 Build Tools). Linux needs alsa/udev/wayland/xkbcommon dev packages (see CI).

## VSCode

- `.vscode/tasks.json`: Run/Build native, Serve web, release builds, Test (default test task).
- `.vscode/launch.json`: F5 debugs the native build via CodeLLDB.
- `.vscode/settings.json`: rust-analyzer uses clippy, format on save, separate target dir.

## CI

- `.github/workflows/test.yml`: on every branch push and PR, runs `cargo xtask test` on Windows, macOS and Linux. The `All tests passed` job aggregates them and is the check to require in GitHub branch protection.
- `.github/workflows/build.yml`: on push to `master` (or manually), full release builds for the three OSes plus `trunk build --release`.
