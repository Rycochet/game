# shared: vitals

Health and Mana: how much harm a creature can take and how much magic it has left, with how much there can be and refilling (Mana always, Health out of combat). The same rules for players and mobs. Attacks harm (`shared::combat`); Heal spends Mana and restores Health (`shared::magic`). What happens at 0 Health is the server's (`server/server.md`).

## Files

- `crates/shared/src/vitals/points.rs`: `Points` (thousandths: `whole`, `milli`, `shown`, `as_f32`), `PER_POINT`.
- `crates/shared/src/vitals.rs`: `Vital::{Health, Mana}` (`ALL`, `name`, `attribute`: Toughness, Will), `Vitals` (on `Sheet`: `health`, `mana`; `at`, `told`, `settle`, `fill`, `hurt`, `spend_mana`, `restore`, `in_combat`, `most(vital, attributes, totals, playtime)`, `limits(attributes, playtime)` (no bonuses: mobs), `limits_with(attributes, totals, playtime, until)`), the numbers below. In `vitals/`: `track.rs` (`Track`: one of them, `Rate { most, refill, restore, until }`, `refilled`, `VitalNow { value, most, refills_in, refill, restore, restore_ends_in }` with `after`), `limits.rs` (`Limits { health, mana, refill, restore }` (`get`, `rate`): `refill` is the ticks from empty to full, `REFILL_TICKS` from `Vitals::limits`, set by the server from the character's `Difficulty` (`Situation::refill`); `Restore { health, mana, until }`; `MostParts { vital, attribute, playtime, practice, bonus }` with `total`), `rebase.rs` (`Vitals::rebase`: keeping shares when bonuses change the most), `old_change.rs` (`Change`, crate-only: older saves' changes, read into `Vitals::changes` and turned into effects by `Sheet::settled`).

## Public API

- Everything above. Numbers (first guesses; owner, 2026-11-06: balance by playing): `ATTRIBUTE_PER_POINT` 5 (a median 100 gives 20), `REFILL_TICKS` (5 minutes, empty to full), `CALM_AFTER` (10 s), `PLAYTIME_PER_POINT` (24 h of active playtime adds 1 to both), `PRACTICE_STEP` 25 (practice turns into points by a square root in steps of it: 25 is 1, 100 is 2), `PRACTICE_PER_HOUR` 20.

## Depends on

- `sheet` (`Attributes`), `bonus` (`Totals`, `Stat`), `energy` (`ACTION_TICKS_PER_SECOND`), `serde`.

## Used by

- `difficulty` (`Points`, `REFILL_TICKS`), `combat` (`Points`), `sheet` (`Sheet::vitals`, `Sheet::limits`), `bonus`, `protocol` (`ServerMessage::Vitals`, `ClientMessage::DevVitals`), `server` (`connections/vitals.rs`).

## Notes

- Stored like `energy`: a value and the action tick it refills from (`since`), so any later tick is a formula. Refilling is a straight line at most ÷ `REFILL_TICKS` a tick, so it uses the same attribute as the most, and "other factors" can scale it later. Integers only.
- Amounts are `Points`, thousandths of a point (owner, 2026-11-07: rounding whole points made harm too easy or too hard), so harm and refilling keep their fractions; the most stays in whole points, and players see whole points rounded up (`Points::shown`), so the living never show 0. Practice is kept in thousandths too. A `Track` saved in whole points (no `milli` flag) is turned into thousandths on reading (`TrackSave`).
- Being hurt is combat (`in_combat`): Health doesn't refill until `CALM_AFTER` after it. Starting an attack calls it too (server `connections/combat.rs`, which then sends `Vitals` so the client knows). `fighting(now)` is whether Health is still waiting: no changing floors then (owner, 2026-11-08); the client works it out from `VitalNow::refills_in` (`common::ToldVitals::fighting`). Mana always refills, and spending it isn't combat (owner, 2026-11-07). `restore` gives Health or Mana back up to the most (the rest lost), keeping any wait.
- The most = attribute (with its bonuses) ÷ 5 + playtime ÷ a day + √(practice ÷ 25), then the most's own bonuses (`shared::bonus`), rounded once; Health never below 1, Mana never below 0 (heavy armour can take it all; owner, 2026-11-14). At a most of 0 the value is kept as it was (not settled), to come back when the most does. Practice is what was taken or spent, at most `PRACTICE_PER_HOUR` a playtime hour each (counted by the hour playtime is in, so a clock as the rust plan has).
- Full is kept as full (`u32::MAX` in a `Track`), so a new or old save starts full and full stays full when the most rises. Below full, playtime and practice growth keep the same number; bonuses changing the most keep the share instead, rounding down (`rebase`, owner, 2026-11-14: 50/100 with +20 is 60/120, and back to 50/100), so wearing or removing never heals or refills. Anything changing the most settles first, at the old limits.
- Restoring from bonuses (`Limits::restore`, `bonus::Stat::Restore`) adds points a second to refilling's straight line while refilling (not while waiting after a fight), until `until`; `VitalNow` carries it so the client's `after` matches `at` (tested).
- Refilling goes by the server's `ActionClock`, which runs while a player is offline on a networked server: logging out refills them (owner: fine, as the monsters around them refill too). Pausing stops it.
- A client is told `VitalNow` (value, most, ticks until refilling starts, ticks to refill from empty) and refills it itself with `after`, which matches `at` (tested).
- Refilling speed comes from `Limits::refill` (Easy is twice as fast: `shared::difficulty`). Anything changing it settles first at the old speed (server `connections/difficulty.rs`).
