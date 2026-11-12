# shared: facing

Which way creatures face, and the field of view that goes with it (owner, 2026-11-12, free positions PR 9): a cone ahead plus a small circle all round, the same rule for players and mobs on both sides. Only the angle: walls and distance are `dungeon::Sight`'s and the callers'.

## Files

- `crates/shared/src/facing.rs`: re-exports.
- `facing/heading.rs`: `Heading` (`u16`, `TURN` steps a turn, 0 east growing towards south: `EAST`, `SOUTH`, `WEST`, `NORTH`), `of(dx, dy)` and `towards(from, to)` (by a binary search on a whole-number sine), `turned`, `apart` (the short way), `unit` (a vector of about `UNIT`), `radians` / `from_radians` (client drawing and input only). `EIGHTH` (a tap of Turn Left/Right).
- `facing/field.rs`: `Field { half, near, further }` (`of(kind, observation)`: the kind's `field_of_view` plus up to 40° and 3 cells of reach from Observation at 100%; `ALL_ROUND`; `is_all_round`), `NEAR` (a cell, in units), `observation_skill()` (`survival/observation`).
- `facing/cone.rs`: `Cone { at, heading, field }`: `sees(spot)` (within the near circle or within half the width of the heading), `sees_cell`.

## Public API

- `Heading`, `TURN`, `EIGHTH`, `UNIT`, `Field`, `NEAR`, `observation_skill`, `Cone` (above).

## Depends on

- `shared::spot` (`Spot`, `CELL_SPAN`), `shared::creature` (`CreatureKind::field_of_view`), `shared::skill` (`SkillId`, `VALUE_SCALE`), `serde`.

## Used by

- `shared::dungeon` (`Sight::add_view_in`, `add_glimpses_in`), `protocol` (`EntityLook::heading`, `ClientMessage::Face`, `ServerMessage::Turned`), `server` (players' and mobs' headings, sight, noticing, Observation practice), client `common` (`Facing`, `PlayerField`, `PlayerCone`), `player`, `entities`, `view3d`, `dungeon_map`, `character`.

## Notes

- Integers only, the same everywhere: the sine is Bhaskara's formula (within 0.2%), finer for finding a heading so small angles come out right; a test checks a heading and its vector agree within half a degree.
- Shared on purpose, so a future ability can draw an enemy's arc (owner). Height doesn't block sight (the owner's TODO). The world map has no cone.
- Widths are first guesses (owner): tune `CreatureKind::field_of_view`, `WIDER_AT_FULL`, `FURTHER_AT_FULL` and `NEAR` after playing.
