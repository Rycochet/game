# client: lightning

The flash and bolts of lightning: keeps `common::Lightning` (how bright the flash is now) from the server's `Lightning` message or the debug menu's Lightning Now, passes each bolt on as `common::BoltStruck`, and makes, fades and removes bolt meshes, which `world_map` and `dungeon_map` place (`spawn_bolt`). The views brighten what is under the sky by `Lightning`.

## Files

- `crates/client/src/lightning/mod.rs`: `LightningPlugin`; `flash` (every frame: a strike starts a flash, and `Lightning` is set to its strength, only when it changes).
- `crates/client/src/lightning/bolt.rs`: `spawn_bolt`, private `Bolt` (its age) and `fade_bolts` (every frame; despawns once faded). Each piece is two crossed strips, unlit and see-through, casting no shadow, with no black outline (it is light, not a model).
- `crates/client/src/lightning/shape.rs`: `segments(seed)`: a 12-step jagged main stroke from height 1 to the origin, and 1 to 3 thinner forks. Tests.
- `crates/client/src/lightning/envelope.rs`: `strength(seconds, gentle)`: full for `FLASH_SECONDS` (0.1 s, like a token's red flash), or with Reduce Motion fading up to `GENTLE_MOST` (0.35) over 0.2 s and down over 0.4 s. `bolt_alpha`: full for the flash, then fading over 0.25 s; with Reduce Motion as the gentle flash, up to half.

## Public API

- `spawn_bolt(commands, meshes, materials, seed, ground, height, width) -> Entity`: draws a bolt striking `ground`; the caller adds its view's scene marker.
- Otherwise `common::Lightning`, `LightningStrike`, `BoltStruck`.

## Depends on

- `common` (`Lightning`, `LightningStrike`, `BoltStruck`, `Preferences`), `shared::rng`, `connection` (`FromServer`), `shared::protocol`.

## Used by

- `world_map` and `dungeon_map` (`spawn_bolt`).

## Notes

- When it flashes is the server's choice (`shared::weather`'s `lightning`, once a second), so everyone in the same storm sees the same flashes, within their latency.
- Runs in `Update` without ordering against the views: a flash may show a frame late.
