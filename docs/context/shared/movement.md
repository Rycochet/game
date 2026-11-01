# shared: movement

How characters move: one step at a time to one of the 8 neighbouring grid points, with the rules the client checks before showing a step and the server checks again before accepting it.

## Files

- `crates/shared/src/movement.rs`: `Step`, `step_on_level`, `step_cost`, `WorldGround`, `step_on_world_map`, `world_step_cost`, `steps_between`. Tests: only single steps exist (also when decoded from a message), steps on a level need walkable cells, one cell of liquid is jumped, costs follow distance, water stops world map steps but not out of it, slopes cost extra, steps between points add up.

## Public API

- `Step` (serde, as `(i8, i8)`): `dx` and `dy` each -1, 0 or 1, not both 0. `Step::new(dx, dy) -> Option<Step>` is the only way to make one; decoding anything else from a message fails (the link reports it garbled). `.dx()`, `.dy()`, `.from((x, y))`.
- `step_on_level(level, from, step) -> Option<(x, y)>`: onto a walkable cell (`Level::is_walkable`), or over one liquid, hole or fence cell onto a walkable one beyond it (a jump: two cells away, one `Step`, nobody stops there); what can't be crossed can't be stepped onto. Cutting a wall's corner diagonally is allowed.
- `STRAIGHT_COST` (70), `DIAGONAL_COST` (99), `step_cost(from, to)`: the time a step takes, by the distance it covers (a jump is two cells' worth). Pathfinding uses it now; walking speed and terrain costs are to use it too.
- `WorldConditions { water, snow_line }`: what decides where can be walked now, as the server says (the hour's `WorldParams::water_level`, the weather).
- `WorldGround::new(&WorldParams)`, `.elevation(x, y, conditions) -> Option<i32>`: in whole quarters of `Surface::broad_height`, `None` for open water (ice counts as ground).
- `step_on_world_map(ground, conditions, from, step) -> Option<(x, y)>`: onto dry ground or ice; anyone standing in open water may step anywhere, so nobody caught by the tide, a thaw or put there is stuck.
- `world_step_cost(from, to, elevations) -> u32`: `step_cost` plus `SLOPE_COST` (45) per elevation climbed or descended; water counts as flat.
- `steps_between(from, to) -> Vec<Step>`: diagonal first, then straight (dev tools, tests).

## Depends on

- `shared::dungeon` (`Level`), `shared::surface`, `shared::world`, `serde`.

## Used by

- `server` (`world_map`, `sites`), `shared::pathfinding`, `shared::protocol` (`ClientMessage::Step`), client `common` (`CurrentWorld::ground`), `player` (keyboard and travel steps), `connection` (`prediction`), `debug_menu`, `dev_tools`.

## Notes

- No speed limit yet: the server takes as many steps as arrive. A limit comes with game time (turns or a step budget), in `step_cost` units. So slopes only steer routes and slow what the client shows of click travel; keyboard steps aren't slowed.
- Walkable ground follows the tide, by whole game hours, matching what the client draws. If an `Hour` message crosses a step in flight, that step may be refused and put right.
- Climbs count on broad height, not the full surface: tried on real mountains, a higher flat cost or one growing with steepness on the full surface barely changed routes, because small bumps cost the same on any route and drown out the mountain. Tune with `SLOPE_COST` (its comment records what was seen at each value).
