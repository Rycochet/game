# client: character

What the character is like: the left of the Character panel, which `inventory` opens, in words for now. Native and web.

## Files

- `crates/client/src/character/mod.rs`: `CharacterPlugin`, `CharacterSheet`, `describe` (what the panel says; unit tested).

## Public API

- `CharacterPlugin`.
- `CharacterSheet(Sheet)` (resource): what the server last said.

## Depends on

- `common` (`SheetText`, `PlayerSpeed`), `connection` (`FromServer`), `inventory` (`Carried`), `widgets`, `shared::sheet`, `shared::item`, `shared::protocol`.

## Used by

- Nothing directly.

## Notes

- It writes `common::PlayerSpeed` for `player`, from the sheet and what is carried.
- The weight and load (`Sheet::load`) are worked out here from `Carried`, so they follow a move before the server answers.
- Appearance is only shown as words: the token's picture doesn't change with it yet, nor with the creature's kind. That comes when entities have their looks.
