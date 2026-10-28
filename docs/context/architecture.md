# Architecture: client / server

Read this before any work on game rules, world generation, randomness or networking.

## Principle: don't trust the user

The game is client/server, like an MMO. The **server is authoritative**: it owns the real game state and validates every command (move, attack, dig, trade) against the same rules and data the client uses. The **client is for user experience**: rendering, input, sound, UI, and *predicting* results so the game feels responsive. If the client and server disagree, the server wins and the client corrects itself.

- Clients send **intents** ("move north", "use item 3"), never results ("I am now at 10,4", "I have 500 gold").
- The server checks each intent (is the target cell walkable? is it adjacent? does the player own the item? cooldowns?) and then broadcasts the outcome.
- The client may run the same validation locally to predict the result, but treats it only as a guess.

## Crate layout (Cargo workspace)

All three exist; `server` is a placeholder (startup only) until networking.

| Crate            | Runs on                         | Contains                                                                                       |
| ---------------- | ------------------------------- | ---------------------------------------------------------------------------------------------- |
| `crates/shared`  | client + server (native + wasm) | Game rules: world generation, deterministic RNG, movement/command validation, network protocol types, shared components. **No rendering, windowing, audio or input.** |
| `crates/client`  | native + browser                | Bevy app with rendering, input, UI, audio; uses `shared` for prediction; talks to the server.   |
| `crates/server`  | native only (never wasm)        | Headless Bevy app (no window) running `shared` rules authoritatively; networking, persistence.  |

Both `client` and `server` are Bevy apps. `shared` is currently plain Rust (no Bevy); if shared ECS systems are needed it may depend on the non-rendering parts of Bevy only (e.g. `bevy_ecs`, `bevy_math`), which keeps the server lean.

Native clients embed the server (`ServerPlugin`) for single player. The web client never includes server code (it will need filesystem access): it only connects to a server, such as the Docker image.

## Shared vs client-only: deciding per feature

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
7. **Golden tests.** `shared` keeps tests asserting exact generated values for fixed seeds/coordinates. CI runs them natively on all three OSes and under wasm, so any divergence fails the build. A change to generation output is a deliberate, versioned change (old worlds would change), never a side effect.
