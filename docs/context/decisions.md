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

Rules, world generation, RNG, validation and protocol types go in `shared`, used by both sides. Client and server are both Bevy apps (server headless) so shared ECS systems run in both. Whether visual-only features live in `shared` or `client` is decided per feature and recorded in its context file. `crates/shared` and `crates/client` exist; `shared` is plain Rust with no Bevy dependency until shared ECS systems are needed. `crates/server` is added with networking.

## 2026-10-27: Deterministic generation

All shared randomness is seeded and derived by hashing (seed, purpose, coordinates) so results are independent of generation order. Authoritative state uses integers (grid positions). Shared code avoids platform-dependent float functions (or uses `libm`). Golden-value tests run natively on all three OSes in CI (running them under wasm too is still to add). Rules: `architecture.md`.

## 2026-10-27: Network message encoding: `postcard` + `serde`

`postcard` (MIT OR Apache-2.0): compact binary format with a documented, stable wire format, works on wasm. Preferred over `bincode`, whose development stopped in 2025.

## 2026-10-27: Noise library: `fastnoise-lite`

Chosen by the owner. The old prototype used Perlin noise via `simplex-noise`; the results matter, not the library. Uses OpenSimplex2 with f64 coordinates; internally only `sqrt`/`floor`, so it's deterministic across platforms.

- **Chosen: `fastnoise-lite`** (MIT). Single-file port of FastNoise Lite, which also exists for C#, JS and GLSL, so a client-only shader could reproduce the same noise for visuals. Offers OpenSimplex2 (fewer grid-aligned artefacts than classic Perlin), Perlin, cellular/Voronoi (useful for biomes/regions), fractal layering and domain warping built in.
- Alternative: `noise` (MIT/Apache-2.0). Larger, composable set of noise functions in f64; slower and updated less often.

Either way, golden tests guard cross-platform determinism.

## 2026-10-27: Random number generation: own SplitMix64 in `shared::rng`

Chosen by the owner. A small hand-written hash turns (seed, purpose, coordinates) into a value, and the same SplitMix64 generator provides sequences, so there's no dependency.

- `rand_pcg` (MIT OR Apache-2.0) can be added if a sequence ever needs better statistical quality; PCG's output is documented as reproducible across platforms and versions.
- Alternative: `rand_chacha` (MIT OR Apache-2.0). Same guarantees, slower; only needed if numbers must be unpredictable to players.
- Never `rand::rngs::StdRng`, `thread_rng`, or `std`'s `DefaultHasher`: their output isn't guaranteed stable.

## Open: networking library (decide with a small prototype)

The browser can only use WebSockets or WebTransport (no raw UDP), so the transport must support those as well as native.

- **Recommended: `bevy_replicon` + `bevy_replicon_renet2`** (MIT OR Apache-2.0). Server-authoritative replication: server state is copied to clients automatically, clients send events (our intents). `renet2` adds WebTransport/WebSocket transports for the browser. Fits a grid roguelike with discrete commands.
- Alternative: `lightyear` (MIT OR Apache-2.0). Includes client prediction, rollback and interpolation; more powerful, more complex. Worth it if movement becomes real-time and fast-paced.
- Alternative: hand-rolled WebSockets (`tokio-tungstenite`, MIT, on the server; the browser's WebSocket on the client) with `postcard` messages. Full control and simple to understand, but we'd write reconnection, replication and interest management ourselves.

## 2026-10-27: Test command `cargo xtask test`, required in CI

One command, identical locally and in CI: formatting check, clippy with warnings as errors for native and wasm (type-check only), then `cargo test --workspace`. Implemented as an `xtask` helper crate (the usual Rust pattern) because cargo aliases can't chain commands and scripts would differ between Windows and Unix. The GitHub "Test" workflow runs it on every branch on all three OSes; its "All tests passed" check must pass before merging.

## 2026-10-27: Develop on native; browser checked occasionally

Day-to-day building, running and testing use the native Windows build: it rebuilds about 3× faster than the web build, and Bevy produces an equivalent web client. CI type-checks wasm on every push. Browser testing and web performance are not tasks until later in development; avoid premature optimisation, but keep tracking performance numbers.

## 2026-10-27: World map style: Legend (1992) as the reference, modern look

The world map is inspired by Legend (1992): a square slab of terrain seen from the south in perspective on black, with an earth cut-away at the front and clear colour bands (water, beach, grass, hills, snowy mountains). It should be recognisable to people who know that style, but look modern rather than retro.

## 2026-10-27: Two world-map renderers while the look is decided

`slice_view` is a faithful port of the prototype's CPU slice renderer (the original look and a performance baseline). `mesh_view` renders the same `shared::surface` data as a lit 3D mesh on the GPU, with the same composition: a fixed 400×400 window around the player that moves one step at a time, a flat front face, and the camera aimed at sea level so it doesn't bob. Its camera uses a narrow field of view from far away for flat, near-isometric perspective, and it bakes the slice view's strong slope shading into the vertex colours under a harsh sun, keeping the old look's surface variety. Water is drawn as part of the terrain at the water surface (as the slice view does), and tonemapping is off so the original palette isn't washed out. Brightness is tuned to roughly match the slice view. Tab switches between them; both use `surface_style` for colours. Which one continues is still to be decided.

## 2026-10-27: Frame rate capped at ~30 fps

Not a twitch game: the client updates about 30 times a second while focused (input still wakes it immediately) and once a second unfocused, to save GPU and power.

## 2026-10-27: Native dev tools for visual checks

`dev_tools` (native only, inert by default) lets environment variables choose the starting view and position and save a screenshot, then quit, so changes can be checked visually without a person watching.

## 2026-10-27: Temperature demo and shared colour formula

The prototype's snow-line animation is back as a demo (T key; G toggles auto-walk), driven by a `Weather::snow_line` resource that both views read. Both views now get colours from one function, `SurfaceStyle::shade`, so they can't drift apart. At the owner's request the beaches are about half as wide (`shore_jitter` 0.5) and sand is a brighter, fixed yellow.
