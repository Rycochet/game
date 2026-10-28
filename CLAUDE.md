# Game: guide for Claude

Multiplayer roguelike with an infinite, procedurally generated world, written in Rust with the Bevy engine. Client/server: the client builds natively (Windows/macOS/Linux) and to WebAssembly (browser); the server builds natively only. Design notes are in `README.md` and `designs.txt`.

## Core rules (always apply)

- **Don't trust the user.** The server is authoritative and validates every command against the same rules/data as the client. Clients send intents, never results. The client only predicts.
- **Determinism.** Anything both sides compute (world generation, validation) lives in shared code, uses only the project's seeded, position-hashed RNG, integer game state, and no platform-dependent float functions.
- Full details, including how to decide shared vs. client-only code: `docs/context/architecture.md`. Read it before touching game rules, generation, randomness or networking.

The project owner is new to Rust: explain Rust concepts briefly when they come up, and prefer clear code over clever code.

## How to load context (read this first)

**Do not read the whole codebase.** Each feature module has a short context file in `docs/context/`. Read only the ones relevant to the task, then open only the source files they point to.

| Module              | Source                          | Context file                          | Covers                                              |
| ------------------- | ------------------------------- | ------------------------------------- | --------------------------------------------------- |
| (project)           | root files                      | `docs/context/project.md`             | Build, tooling, folder layout, CI                   |
| (design)            | n/a                             | `docs/context/architecture.md`        | Client/server split, trust and determinism rules    |
| shared: rng         | `crates/shared/src/rng.rs`      | `docs/context/shared/rng.md`          | Seeds, position hashing, random sequences           |
| shared: world       | `crates/shared/src/world.rs`    | `docs/context/shared/world.md`        | World settings from seed, tides                     |
| shared: surface     | `crates/shared/src/surface.rs`  | `docs/context/shared/surface.md`      | Terrain height/jitter at any grid position (noise)  |
| client: common      | `crates/client/src/common/`     | `docs/context/client/common.md`       | Current world, clock, weather, view state/stats, frame cap |
| client: player      | `crates/client/src/player/`     | `docs/context/client/player.md`       | Player grid position, keyboard movement             |
| client: surface_style | `crates/client/src/surface_style/` | `docs/context/client/surface_style.md` | Planet colour bands and thresholds              |
| client: slice_view  | `crates/client/src/slice_view/` | `docs/context/client/slice_view.md`   | Original CPU-drawn slice map (prototype port)       |
| client: mesh_view   | `crates/client/src/mesh_view/`  | `docs/context/client/mesh_view.md`    | 3D terrain window mesh, front face, camera, light, water |
| client: demo        | `crates/client/src/demo/`       | `docs/context/client/demo.md`         | Demo keys: G auto-walk, T temperature (snow) cycle  |
| client: ui          | `crates/client/src/ui/`         | `docs/context/client/ui.md`           | HUD status text                                     |
| client: dev_tools   | `crates/client/src/dev_tools/`  | `docs/context/client/dev_tools.md`    | Env-var helpers: start view/position, walk, screenshots |

Also: `docs/context/decisions.md` is the log of architecture decisions. Check it before proposing a structural change, and follow what it records.

If a task touches a module not listed here, it doesn't exist yet: create it (see "Adding a module").

Crates: `crates/shared` (plain Rust, no Bevy: rules and generation), `crates/client` (Bevy app, binary `game`), `xtask` (helper commands). There is no `crates/server` yet; it arrives with networking (see `architecture.md`).

## Module rules

- **Client:** every feature is a folder `crates/client/src/<module>/` whose `mod.rs` exposes one Bevy `Plugin` (`<Name>Plugin`), registered in `crates/client/src/main.rs`, which contains no game logic.
- **Shared:** plain Rust modules (`crates/shared/src/<module>.rs`, or a folder when it grows) with no Bevy dependency so far. Anything the server will need to agree on goes here (see `architecture.md`).
- A module may use `common` freely. It should not reach into another feature module's internals. Cross-module communication goes through types the owning module deliberately makes `pub` (components, resources, messages/events), and each context file lists them under "Public API".
- If two modules need the same type, move it to `common` rather than having them depend on each other.
- Keep modules small enough that their context file stays short. Split a module (e.g. `world/terrain`, `world/render`) when it grows past what one context file can summarise.

## Keeping context files up to date (required)

Whenever a change affects a module, update its `docs/context/<crate>/<module>.md` **in the same commit / branch / PR** as the code. Update it when you change:

- the module's files (added, removed, renamed),
- its public API (anything `pub` other modules use),
- which modules it depends on, or which depend on it,
- a non-obvious behaviour, gotcha or invariant.

Context files are summaries, not documentation: a few lines per item, enough to decide which file to open. Don't paste code into them. Also update the table above and `docs/context/project.md` when modules or tooling change, and add an entry to `decisions.md` when a structural decision is made (including decisions the owner makes in conversation).

## Before a branch is merged

When the owner says a branch is ready to merge (or asks for a PR):

1. Run `cargo xtask test`; it must pass (CI requires it too).
2. Check every context file the branch touched still matches the code.
3. Tidy `docs/context/decisions.md` following the maintenance rules at its top: fix outdated entries freely, but **ask before removing anything**, listing what and why.

## Adding a module

1. Client: create `crates/client/src/<module>/mod.rs` with a `<Name>Plugin` and register it in `main.rs`. Shared: add `crates/shared/src/<module>.rs` and `pub mod` it in `lib.rs`.
2. Copy `docs/context/_template.md` to `docs/context/<crate>/<module>.md` and fill it in.
3. Add a row to the table above.

## Commands

- Native: `cargo run` · Web: `trunk serve` (http://localhost:3000) · Format: `cargo fmt`
- **Test: `cargo xtask test`**: formatting check, clippy with warnings as errors for native and for wasm (type-check only), then `cargo test --workspace`. Run it before finishing any change. CI runs it on every branch and it must pass before merging.
- **Develop and test on the native (Windows) build.** Trust Bevy to produce an equivalent web build: CI type-checks wasm on every push, and the owner tests in the browser occasionally. Don't make browser testing or web performance a task until the owner asks (later in development).
- **Checking visuals:** run the native build with `dev_tools` environment variables to start in a view/position and save a screenshot, then read the image (see `docs/context/client/dev_tools.md`).
- Bevy compiles slowly the first time (15+ min clean); incremental builds take seconds. Run long builds in the background.

## Conventions

- Bevy version is pinned in `Cargo.toml` (currently 0.20). Bevy changes its API between versions: check the compiler errors and docs.rs for the pinned version rather than assuming older APIs.
- Client and shared code must compile for `wasm32-unknown-unknown` as well as native: no threads, blocking file IO or native-only crates without a `#[cfg(target_arch = "wasm32")]` alternative. Server code is native-only and may use them.
- Dependencies must be open source under an OSI-approved licence; MIT (or MIT/Apache-2.0 dual) preferred. When suggesting a library, give the licence and the reason, and offer alternatives where sensible: the owner cares about results, not specific libraries.
- Git: the owner uses branches and PRs on GitHub (`Rycochet/game`, default branch `master`). Don't push or open PRs unless asked.
