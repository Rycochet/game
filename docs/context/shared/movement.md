# shared: movement

How characters move: one step at a time to one of the 8 neighbouring grid points, with the rules the client checks before showing a step and the server checks again before accepting it.

## Files

- `crates/shared/src/movement.rs`: `Step`, `step_on_level`, `step_cost`, `step_on_world_map`, `steps_between`. Tests: only single steps exist (also when decoded from a message), steps on a level need walkable cells, one cell of liquid is jumped, costs follow distance, steps between points add up.

## Public API

- `Step` (serde, as `(i8, i8)`): `dx` and `dy` each -1, 0 or 1, not both 0. `Step::new(dx, dy) -> Option<Step>` is the only way to make one; decoding anything else from a message fails (the link reports it garbled). `.dx()`, `.dy()`, `.from((x, y))`.
- `step_on_level(level, from, step) -> Option<(x, y)>`: onto a walkable cell (`Level::is_walkable`), or over one liquid cell onto a walkable one beyond it (a jump: two cells away, one `Step`, nobody stops on liquid); liquid that can't be crossed can't be stepped onto. Cutting a wall's corner diagonally is allowed.
- `STRAIGHT_COST` (70), `DIAGONAL_COST` (99), `step_cost(from, to)`: the time a step takes, by the distance it covers (a jump is two cells' worth). Pathfinding uses it now; walking speed and terrain costs are to use it too.
- `step_on_world_map(from, step) -> (x, y)`: anywhere, for now (no terrain stops anyone).
- `steps_between(from, to) -> Vec<Step>`: diagonal first, then straight (world map travel covering more than a step in a frame, dev tools).

## Depends on

- `shared::dungeon` (`Level`), `serde`.

## Used by

- `server` (`world_map`, `sites`), `shared::pathfinding`, `shared::protocol` (`ClientMessage::Step`), client `player` (keyboard and travel steps), `connection` (`prediction`), `demo`, `dev_tools`.

## Notes

- No speed limit yet: the server takes as many steps as arrive. A limit comes with game time (turns or a step budget), in `step_cost` units.
