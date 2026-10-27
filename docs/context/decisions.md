# Decisions

Architecture and process decisions, newest last. Each entry: date, decision, why. Add one whenever a structural choice is made, including ones the owner makes in conversation. Don't reverse a decision without the owner agreeing.

This file holds only what is currently true; old entries live on in git history. Maintenance happens **only when a branch is ready to merge** (not on every commit):

- Editing an entry to keep it accurate (e.g. "planned" → done, referring to a commit or PR instead of a branch, merging a superseded entry into its replacement) needs no confirmation.
- **Removing** an entry, or removing a substantive part of one, must be confirmed by the owner first: list what you propose to remove and why, so they can check the diff.

## 2026-10-27: Rewrite in Rust with Bevy

Replaces the earlier TypeScript/React prototype (still on `master` history). Rust compiles to native apps on all desktop OSes and to WebAssembly. Bevy chosen as the engine: actively developed, supports native + web from one codebase, and its ECS/plugin model suits a modular codebase and a large simulated world.

## 2026-10-27: One Bevy plugin per feature module

Each feature lives in `src/<module>/` and exposes a single `Plugin`; `main.rs` only wires plugins together. Modules interact only through deliberately `pub` components, resources and messages, with shared types in `common`. Goal: work on one area (e.g. map drawing) without needing other areas (e.g. characters) in context.

## 2026-10-27: Context files for Claude

`CLAUDE.md` is the index; `docs/context/<module>.md` summarises each module (files, public API, dependencies, notes). They are updated in the same change as the code they describe, so Claude can load only the relevant parts instead of the whole codebase.

## 2026-10-27: Web build with Trunk

Trunk builds and serves the wasm version (`trunk serve`, port 3000) using `index.html`. Simplest standard tool for Bevy on the web.

## 2026-10-27: Client/server with an authoritative server

"Don't trust the user." The server owns game state and validates every client command using the same rules and data as the client; clients send intents and only predict results. Both client and server must build for Windows, macOS and Linux; the client also for the browser, the server never. Details: `architecture.md`.

## 2026-10-27: Cargo workspace `shared` / `client` / `server` (planned)

Rules, world generation, RNG, validation and protocol types go in `shared`, used by both sides. Client and server are both Bevy apps (server headless) so shared ECS systems run in both. Whether visual-only features live in `shared` or `client` is decided per feature and recorded in its context file. Restructure not done yet.

## 2026-10-27: Deterministic generation

All shared randomness is seeded and derived by hashing (seed, purpose, coordinates) so results are independent of generation order. Authoritative state uses integers (grid positions). Shared code avoids platform-dependent float functions (or uses `libm`). Golden-value tests run on every OS and wasm in CI. Rules: `architecture.md`.

## 2026-10-27: Network message encoding: `postcard` + `serde`

`postcard` (MIT OR Apache-2.0): compact binary format with a documented, stable wire format, works on wasm. Preferred over `bincode`, whose development stopped in 2025.

## Open: noise library (decide when world generation starts)

The old prototype used Perlin noise via `simplex-noise`; the results matter, not the library.

- **Recommended: `fastnoise-lite`** (MIT). Single-file port of FastNoise Lite, which also exists for C#, JS and GLSL, so a client-only shader could reproduce the same noise for visuals. Offers OpenSimplex2 (fewer grid-aligned artefacts than classic Perlin), Perlin, cellular/Voronoi (useful for biomes/regions), fractal layering and domain warping built in.
- Alternative: `noise` (MIT/Apache-2.0). Larger, composable set of noise functions in f64; slower and updated less often.

Either way, golden tests guard cross-platform determinism.

## Open: random number generation (decide with the `shared` crate)

- **Recommended:** a small hand-written coordinate hash in `shared` (SplitMix64-style, a few lines, no dependency) to turn (seed, purpose, x, y, z) into a value, plus `rand_pcg` (MIT OR Apache-2.0) when one place needs a sequence of numbers (seeded from that hash). PCG is fast and its output is documented as reproducible across platforms and versions.
- Alternative: `rand_chacha` (MIT OR Apache-2.0). Same guarantees, slower; only needed if numbers must be unpredictable to players.
- Never `rand::rngs::StdRng`, `thread_rng`, or `std`'s `DefaultHasher`: their output isn't guaranteed stable.

## Open: networking library (decide with a small prototype)

The browser can only use WebSockets or WebTransport (no raw UDP), so the transport must support those as well as native.

- **Recommended: `bevy_replicon` + `bevy_replicon_renet2`** (MIT OR Apache-2.0). Server-authoritative replication: server state is copied to clients automatically, clients send events (our intents). `renet2` adds WebTransport/WebSocket transports for the browser. Fits a grid roguelike with discrete commands.
- Alternative: `lightyear` (MIT OR Apache-2.0). Includes client prediction, rollback and interpolation; more powerful, more complex. Worth it if movement becomes real-time and fast-paced.
- Alternative: hand-rolled WebSockets (`tokio-tungstenite`, MIT, on the server; the browser's WebSocket on the client) with `postcard` messages. Full control and simple to understand, but we'd write reconnection, replication and interest management ourselves.

## 2026-10-27: Test command `cargo xtask test`, required in CI

One command, identical locally and in CI: formatting check plus clippy with warnings as errors for native and wasm (type-check only, no full build; real tests get added to it later). Implemented as an `xtask` helper crate (the usual Rust pattern) because cargo aliases can't chain commands and scripts would differ between Windows and Unix. The GitHub "Test" workflow runs it on every branch on all three OSes; its "All tests passed" check must pass before merging.
