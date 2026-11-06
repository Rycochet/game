# client: abilities

The Character panel's Abilities page (B): every ability the character knows (`shared::ability::known`, for the body they are in), as ability widgets flowing in rows, Attack first, with a Skills button above them that stays disabled until the Skills page exists. The action bar comes here next (PR 5). Not the panel or its tabs: those are `inventory`'s, which spawns the page as a `common::AbilitiesPage` node. Native and web.

## Files

- `crates/client/src/abilities/mod.rs`: `AbilitiesPlugin`.
- `crates/client/src/abilities/page.rs`: `fill_pages` (fills each new `AbilitiesPage`, and again when the body changes: `DrawnFor`), `ability_id` (widget ids `abilities/<id>`).
- `crates/client/src/abilities/icons.rs`: `icon_path`: each ability's game-icons.net icon in `assets/icons/` (listed in `xtask/icons.txt`; a test checks each is there).

## Public API

- `AbilitiesPlugin`.

## Depends on

- `common` (`AbilitiesPage`), `inventory` (`Carried`, for the body), `widgets` (`spawn_ability`, buttons), `shared::ability`.

## Used by

- Nothing.

## Notes

- Ids: `abilities/skills` (the Skills button, disabled; the Skills page task enables it and binds K, kept free for it), `abilities/<ability>` (`attack`, `night_vision`, ...). Abilities can't be sorted by dragging; filtering and searching come later (owner).
- Passive abilities are drawn round and never greyed (they are never done); active ones the player can't use now are greyed with a dull red frame.
