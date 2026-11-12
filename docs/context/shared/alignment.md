# shared: alignment

A creature's hidden alignment: how good or evil it has shown itself to be, from −1000 (evil) to +1000 (good), moved only by deeds. Players always have one; every people kind has one; animals have none (owner, 2026-11-12: they act on instinct, not by choice) until pets exist and get one. Never sent to any client: only developer tools show it.

## Files

- `crates/shared/src/alignment.rs`: `Alignment`, `Band`, the limits and bands. Tests.
- `alignment/deed.rs`: `Deed` and `shift`, with each deed's worth.
- `alignment/kinds.rs`: `CreatureKind::alignment_leaning`, `Alignment::rolled`, `ROLLED_SPREAD`.

## Public API

- `Alignment` (serde; `Default` 0): `NEUTRAL`, `new(i32)` (clamped to `MOST_EVIL` −1000 and `MOST_GOOD` 1000), `value`, `band`, `shifted(by)`, `after(deed)`, `rolled(kind, hash)` (a mob's: within `ROLLED_SPREAD` (250) of its kind's leaning; `None` for an animal).
- `Band::{Evil, Neutral, Good}` (`name`): Evil at or below `EVIL_FROM` (−300), Good at or above `GOOD_FROM` (300). Quests and later effects go by the band, never the number.
- `Deed::{Killed { victim, provoked }, Rescued, Escorted, Helped, BetrayedQuest}`, `shift()`: whole numbers, no randomness (all *to balance*). Killing someone Neutral −10 (`KILLED_NEUTRAL`), someone Good from −10 to −60 (`KILLED_BEST`) by how good; someone Evil, or anyone in self-defence (`provoked`), nothing. Rescued +20, Escorted +15, Helped +5, BetrayedQuest −30. Only `Killed` is used yet; the rest are for the Quest system (escorts, chains).
- `CreatureKind::alignment_leaning() -> Option<i16>`: hostile kinds −400, halflings and beggars +150, other peoples 0, animals (cattle, chickens, horses, wolves) `None`.

## Depends on

- `shared::creature` (`CreatureKind`).

## Used by

- `shared::sheet` (`Sheet::alignment`, `as_sent`), `shared::mob` (`Spawn::alignment`, `Regard::provoked`), `protocol` (`DevAlignment`), `server` (mobs, kills, dev tools, `server.alignment`), client `debug_menu` (the Alignment panel).

## Notes

- Hidden: `Sheet::as_sent` strips it from every `Sheet` the server sends (and `ListedCharacter`s); mobs' are kept on the server's `Mob` and never told. Only `ServerMessage::DevAlignment` (the debug menu's Alignment panel), Examine Mob's lines and the debug API (`server.alignment`, `server.mobs`) show it.
- Players: `Sheet::built` starts at 0, `Sheet::settled` gives 0 to a save from before alignments, and a permanent change of creature keeps it.
- Mobs roll it from their own hash (`mob::ALIGNMENT_ROLL`, or `own(62)` in the wilderness), apart from the level's shape, so no generator version changed. It isn't saved: rolled again from the spawn.
- Self-defence (owner): a kill costs nothing if the mob attacked that player, or was unfriendly to them (its kind's feeling, a trespass, or its memory), before their first blow (`mob::Memory::provoked`).
- No drift over time (owner): only deeds move it. PvP and visible effects are later (`todo-alignment-pvp`, `todo-alignment-effects`).
