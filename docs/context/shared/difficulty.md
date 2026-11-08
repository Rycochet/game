# shared: difficulty

How hard the game is for one character (Easy, Normal, Hard, Ironman), and which a server allows. The rules each changes: harm taken, refill speed, what dying costs. The server keeps a character's difficulty and decides it; nothing here is about players choosing (client) or death itself (server).

## Files

- `crates/shared/src/difficulty.rs`: `Difficulty` (`ALL`, easiest first; `name`, `from_name` (any case, trimmed), `harm_taken` (×4/5 Easy, ×6/5 Hard and Ironman, in whole numbers), `refill_ticks` (half `vitals::REFILL_TICKS` on Easy), `drops_on_death` (Hard, Ironman), `rust_on_death` (hours of skill rust: Normal 8, Hard 24), `permanent_death` (Ironman), `may_change_to` (never to or from Ironman), `for_existing` (a player's default for a character with none saved: Ironman becomes Hard)).
- `difficulty/allowed.rs`: `Difficulties`, a set as bits (`ALL`, the default; `only`, `contains`, `with`, `without` (never the last), `is_empty`, `or_all`, `iter`, `len`, `closest(wanted)` (nearest allowed, the easier on a tie), `parse(text)` (`HOW_DIFFICULTIES`: comma-separated names, blank or none known is all, unknown names given back to warn about), `names`).

## Public API

- Everything above.

## Depends on

- `vitals` (`Points`, `REFILL_TICKS`), `serde`.

## Used by

- `protocol` (`Hello`, `NewCharacter`, `SetDifficulty`, `SetDifficulties`, `Characters`, `ServerMessage::Difficulty`), `server` (`ServerOptions::difficulties`, `connections/difficulty.rs`, `CharacterSave::difficulty`, harm in `connections/combat.rs`, `Client::limits`).

## Notes

- Owner's design (2026-11-08): Easy takes a fifth less harm, refills twice as fast and loses nothing on dying; Normal is the game as meant; Hard takes a fifth more and drops all but skin-layer clothes on dying; Ironman takes a fifth more, drops everything and deletes the character. What dying drops is `Inventory::drop_on_death` (`shared::wear`, `wear/death.rs`: Hard keeps what `kept_on_hard` says, worn on the Skin layer alone, not padded, not jewellery); the server puts it on the ground where they fell and deletes an Ironman character (server `connections/combat/struck.rs`). Normal and Hard also take 8 and 24 hours of skill rust on dying (`skill::Skills::rust_for_hours`), mild on purpose: a "doh" rather than a reason to quit (owner, 2026-11-08).
- Ironman is chosen only when making a character, and a player can never change to or from it; a server's rules (what it allows) can still move anyone, Ironman included, onto the nearest allowed.
- Only harm from creatures is scaled, only for players (`Difficulty::harm_taken` on the server); mobs have no difficulty.
- Integers only: harm in thousandths (`Points`).
