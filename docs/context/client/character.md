# client: character

What the character is like: the left of the Character panel, which `inventory` opens, in words for now: kind and size, the nine attributes a row (Body, Mind, Spirit) to a line, carrying and speed, skills, abilities, looks. Native and web.

## Files

- `crates/client/src/character/mod.rs`: `CharacterPlugin`, `CharacterSheet`, `describe` (what the panel says; unit tested).

## Public API

- `CharacterPlugin`.
- `CharacterSheet(Sheet)` (resource): what the server last said.

## Depends on

- `common` (`SheetText`, `PlayerSpeed`, `CarryLimit`), `connection` (`FromServer`), `inventory` (`Carried`), `widgets`, `shared::sheet`, `shared::item`, `shared::protocol`.

## Used by

- Nothing directly.

## Notes

- It writes `common::PlayerSpeed` for `player`, from the sheet and what is carried, and `common::CarryLimit` (the most that can be carried, `Sheet::capacity`) for `inventory`, which shows it under the inventory's grid, so `inventory` needn't depend on `character`.
- The panel shows only the most the character can carry (`Sheet::capacity`); what is carried now is shown under the inventory (owner's choice). Speed is worked out here from `Carried`, so it follows a move before the server answers.
- Appearance is shown here as words; the token's picture (in `entities`) is the kind's body in colours from it.
