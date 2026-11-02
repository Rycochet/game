# client: character

The character panel: what the character is like, from `C` or the person button at the bottom left. Words only, for now. Native and web.

## Files

- `crates/client/src/character/mod.rs`: `CharacterPlugin`, `CharacterSheet`, `describe` (what the panel says; unit tested), the person icon (drawn in code).

## Public API

- `CharacterPlugin`.
- `CharacterSheet(Sheet)` (resource): what the server last said. `OpenCharacterPanel` (message): opens or closes it, as `C` does (`dev_tools`).

## Depends on

- `common` (`GameView`), `connection` (`FromServer`), `inventory` (`Carried`), `widgets`, `shared::sheet`, `shared::item`, `shared::protocol`.

## Used by

- `dev_tools`.

## Notes

- It writes `common::PlayerSpeed` for `player`, from the sheet and what is carried.
- The weight and load (`Sheet::load`) are worked out here from `Carried`, so they follow a move before the server answers.
- Appearance is only shown as words: the token's picture doesn't change with it yet, nor with the creature's kind. That comes when entities have their looks.
