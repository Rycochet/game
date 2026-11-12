# shared: gender

Every creature's gender, players and mobs alike, animals too: Male, Female or Neutral, with a small bonus to one row of the attribute grid. Later text takes its pronouns from it. Not how anyone looks: models don't change with it.

## Files

- `crates/shared/src/gender.rs`: `Gender`, `GENDER_BONUS`. Tests.

## Public API

- `Gender::{Male, Female, Neutral}` (serde, carried by its place: new ones go at the end; Neutral is the `Default`): `ALL`, `name()`, `row()` (Male → Body, Female → Mind, Neutral → Spirit), `bonus()` (an `Attributes` with `GENDER_BONUS` on its row's three), `rolled(hash)` (a mob's: 45% Male, 45% Female, 10% Neutral).
- `GENDER_BONUS` (2, *to balance*): added to each of the three attributes of its row, so every gender adds the same 6.

## Depends on

- `shared::sheet` (`Row`, `Attributes`).

## Used by

- `shared::sheet` (`Sheet::gender`, `Attributes::for_kind(kind, gender, build)`), `shared::mob` (`Spawn::gender`, `EntityLook::gender`), `protocol` (`NewCharacter::gender`), `server` (new characters, mobs, what others are told), client `character_create` (the toggle row), `character` and `character_select` (shown beside the kind).

## Notes

- Owner (2026-11-12): "a subtle nudge" until skills make it mostly irrelevant. The bonus is added beside the kind's in `for_kind`, rounded once, and kept through shapeshifting (a man shapeshifted into a horse keeps his Body bonus).
- It is on the `Sheet`, not `Appearance`, because it affects the rules.
- Saves from before genders load as Neutral and keep their saved attributes with no bonus added (owner's decision): nothing is worked out again for them.
- Mobs roll it from their own hash (`mob::GENDER_ROLL`, or `own(61)` in the wilderness), apart from the level's shape, so no generator version changed.
