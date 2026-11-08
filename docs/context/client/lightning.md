# client: lightning

The flash of lightning: keeps `common::Lightning` (how bright the flash is now) from the server's `Lightning` message or the debug menu's Lightning Now. It draws nothing itself: `world_map` and `dungeon_map` brighten what is under the sky by it.

## Files

- `crates/client/src/lightning/mod.rs`: `LightningPlugin`; `flash` (every frame: a strike starts a flash, and `Lightning` is set to its strength, only when it changes).
- `crates/client/src/lightning/envelope.rs`: `strength(seconds, gentle)`: full for `FLASH_SECONDS` (0.1 s, like a token's red flash), or with Reduce Motion fading up to `GENTLE_MOST` (0.35) over 0.2 s and down over 0.4 s.

## Public API

None: other modules use `common::Lightning` and `common::LightningStrike`.

## Depends on

- `common` (`Lightning`, `LightningStrike`, `Preferences`), `connection` (`FromServer`), `shared::protocol`.

## Used by

- Nothing directly (registered in `main.rs`).

## Notes

- When it flashes is the server's choice (`shared::weather`'s `lightning`, once a second), so everyone in the same storm sees the same flashes, within their latency.
- Runs in `Update` without ordering against the views: a flash may show a frame late.
