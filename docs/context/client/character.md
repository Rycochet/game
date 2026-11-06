# client: character

What the character is like: the left of the Character panel, which `inventory` opens, in words for now: kind and size, the nine attributes a row (Body, Mind, Spirit) to a line, Health and Mana (most of each and where it comes from), carrying and speed, skills, abilities, looks. Native and web.

## Files

- `crates/client/src/character/mod.rs`: `CharacterPlugin`, `CharacterSheet`, `describe` (what the panel says; unit tested).
- `crates/client/src/character/skill_lines.rs`: `skill_lines`: a line per known skill with no known skill under it, by path, with what a check counts ("Athletics › Climbing 25.8"), until the Skills page shows the tree. `vitals_lines.rs`: "Health 31 (Toughness 26, +5 for now)": the most from the last `Vitals`, its parts from the sheet; playtime isn't sent, so its part ("play") is what the others leave. The server sends `Sheet` again when practice or changes do, so the parts stay right.

## Public API

- `CharacterPlugin`.
- `CharacterSheet(Sheet)` (resource): what the server last said.

## Depends on

- `common` (`SheetText`, `PlayerSpeed`, `CarryLimit`, `OwnVitals`), `connection` (`FromServer`), `inventory` (`Carried`), `widgets`, `shared::sheet`, `shared::skill`, `shared::item`, `shared::protocol`.

## Used by

- Nothing directly.

## Notes

- It writes `common::PlayerSpeed` for `player`, from the sheet and what is carried, and `common::CarryLimit` (the most that can be carried, `Sheet::capacity`) for `inventory`, which shows it under the inventory's grid, so `inventory` needn't depend on `character`.
- The panel shows only the most the character can carry (`Sheet::capacity`); what is carried now is shown under the inventory (owner's choice). Speed is worked out here from `Carried`, so it follows a move before the server answers.
- Appearance is shown here as words; the token's picture (in `entities`) is the kind's body in colours from it.
