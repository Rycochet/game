# client: character

What the character is like: the Character panel's Character tab, which `inventory` opens, in words for now: gender, kind and size, the nine attributes a row (Body, Mind, Spirit) to a line, Health and Mana (most of each and where it comes from), carrying and speed, looks. Skills are on the Skills page (`skills_page`). Abilities have their own tab (`abilities`). Native and web.

## Files

- `crates/client/src/character/mod.rs`: `CharacterPlugin`, `CharacterSheet`, `describe` (what the panel says; unit tested).
- `crates/client/src/character/listed.rs`: `describe_listed(ListedCharacter)`: what the tab would say of a character offered to choose from, as far as their save tells (gender, kind, attributes, carrying, difficulty, looks; no Health, Mana or speed), for `character_select`.
- `crates/client/src/character/vitals_lines.rs`: "Health 31 (Toughness 26, +5 bonuses)": the most from the last `Vitals`, its parts from the sheet with its effects' bonuses (`Sheet::totals`, no worn items yet); playtime isn't sent, so its part ("play") is what the others leave. The server sends `Sheet` again when practice or effects do, so the parts stay right.

## Public API

- `CharacterPlugin`.
- `CharacterSheet(Sheet)` (resource): what the server last said.
- `describe_listed(&ListedCharacter) -> String`.

## Depends on

- `common` (`PlayerSpeed`, `PlayerGait`, `CarryLimit`, `OwnVitals`), `connection` (`FromServer`), `inventory` (`Carried`), `widgets`, `shared::sheet`, `shared::item`, `shared::protocol`.

## Used by

- `character_select` (`describe_listed`).

## Notes

- It writes `common::PlayerSpeed` for `player`, from the sheet and what is carried, and `common::PlayerGait` (`movement::Gait` from the skills), and `common::CarryLimit` (the most that can be carried, `Sheet::capacity`) for `inventory`, which shows it under the inventory's grid, so `inventory` needn't depend on `character`.
- The panel shows only the most the character can carry (`Sheet::capacity`); what is carried now is shown under the inventory (owner's choice). Speed is worked out here from `Carried`, so it follows a move before the server answers.
- Appearance is shown here as words; the token's picture (in `entities`) is the kind's body in colours from it.
- `publish_field` keeps `common::PlayerField` (`Field::of` the kind carried and Observation's value), for the player's cone.
