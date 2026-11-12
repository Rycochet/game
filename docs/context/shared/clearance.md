# shared: clearance (and spot)

Positions on a level in whole units (`Spot`, 240 a cell) and room to move for creatures of each size: what blocks them, the one rule for a move from one spot to another (with squeezing and jumps), sliding along what is in the way, and where each size has room. Free positions, PR 2 of 9 (`docs/context/free-positions.md`). Since PR 3, positions on levels are `Spot`s end to end and a player's move that isn't a step between two cells' middles is checked by `check_segment` (`movement::move_on_level`); since PR 4 the client walks freely by the same rule (`slide`, `check_segment`); since PR 5 sizes are in play (players by their kind, mobs' steps and distance maps by theirs) and creatures are in each other's way (`crowd`). Routes by size are in `pathfinding` (`find_sized_path`).

## Files

- `crates/shared/src/spot.rs`: `Spot`, `CELL_SPAN`, `HALF_SPAN`, `root`.
- `crates/shared/src/clearance.rs` and `clearance/`: `blocker.rs` (`Blocker`, `Shape`, how close a move passes one), `cell_blockers.rs` (what each cell puts in the way; `blockers_near`), `footing.rs` (`has_footing`, `has_footing_by`), `segment.rs` (`check_segment`, `check_segment_by`, `Passage`, `segment_is_legal`, `segment_cost`, `JUMP_REACH`), `slide.rs` (`slide`, `slide_by`), `squeeze.rs`, `room_map.rs` (`RoomMap`), `crowd.rs` (`Crowding`, `Other`, `crowd_per_mille`, `check_among`, `crowded_at`; tests in `crowd_tests.rs`). Golden tests: radii, a one-cell corridor and a doorway by size, walls touching at a corner, trunks, jumps, fences, sliding, room maps.

## Public API

- `Spot { x, y }` (serde, i64 units, `x` east, `y` south): `stands_on(cell)` (within `UNDERFOOT`, half a cell, of its middle: stairs underfoot), `middle_of(cell)` (cell `(x, y)`'s middle at `(x, y) × 240`, as `relief::Relief::at`), `cell()` (a spot on a cell's edge is in the cell east or south), `offset`, `distance_squared` (exact, `u128`), `distance` (rounded down), `towards(other, n, d)`. On the Hall's levels `x` is the Hall's own, unwrapped, as its cells are (`hall::wrap_x` brings a cell round). `root(n)`: whole-number square root.
- `Size::radius()` (in `creature`): Tiny 20, Small 60, Medium 80 (today's token base, 2/3 cell across), Large 120, Huge 180, Giant 240. Drawn base = blocking circle.
- `Blocker { shape, fence }`, `Shape::{Box { low, high }, Post { at, radius }}`, `.clearance(a, b)`: how far a move passes it (0 touching). `cell_blockers(level, x, y, &mut out)`, `blockers_near(level, low, high, reach)`:
  - walls (and building doors, blind windows, closed secret doors), windows and closed doors fill the cell, less a `CORNER_CUT` (60) notch at each corner with no wall, window or doorway beside it on either side;
  - an open or broken door (an open secret door too) leaves two frame posts, the doorway `2 × DOOR_HALF_GAP` (192 units, 0.8 cell, as `dungeon_map` draws it) wide, the full cell deep; leaves don't block (owner);
  - a fence is two bars `FENCE_HALF` (12) thick, shaped as a wall's (`WallShape::reach` × 6), running to fences and field gates beside it, and can be jumped (`fence: Some(cell)`);
  - a tree is its trunk, a `TRUNK_RADIUS` (30) post at the middle.
- `has_footing(level, x, y)`: where a middle may stand: walkable cells, and fence and tree cells (beside the bar or trunk). Liquid and holes can only be jumped. `has_footing_by(level, swims, x, y)`: water too for a good swimmer (`Gait::swims_deep`), never lava (free positions PR 6).
- `check_segment(level, size, from, to) -> Option<Passage { squeeze_per_mille, jumped }>`: the rule. Both ends need footing; the middle's line may cross the inside of no cell without footing, unless it is a jump: over exactly one jumpable cell (liquid, hole, or a fence whose bar it crosses), at most `JUMP_REACH` (679, two cells diagonally). The swept circle keeps at least half its radius from every blocker but the jumped fence. `segment_is_legal` (yes or no), `segment_cost` (length × squeeze / 1000). `check_segment_by(level, size, swims, from, to)` and `slide_by`: the same with a swimmer's footing, water swum rather than jumped (players' moves on both sides and their routes, `pathfinding::find_sized_path`; mobs still go by `check_segment`).
- `squeeze_per_mille(radius, clearance)` (owner, 2026-11-10): clearance ≥ radius is free (1000); down to half the radius (a gap half its width), slower by (radius / clearance)², at most `MOST_SQUEEZE_PER_MILLE` (4000); less is `None`.
- `slide(level, size, from, to) -> Spot`: for the client's movement. `to` if the move is a legal walk (squeezed or not); otherwise as far along as it goes, then on along east-west or north-south, whichever gets further, never more squeezed than at `from` (so pushing into a wall doesn't press into it). Never jumps. `from` if it has no room there.
- Creatures in the way (owner, 2026-11-10): `Other { at, size, crowding }`, `Crowding::{Pass, Squeeze, Block}` (players pass through players; anyone else but an enemy is squeezed past, "excuse me"; enemies block). `crowd_per_mille(size, from, to, others)`: circles overlapping along the move; squeezing past goes by the room left round the mover's middle as for a gap but never refuses (at worst `MOST_SQUEEZE_PER_MILLE`); an enemy's circle can't be entered, and one already overlapping (stairs, a shove, god mode turned off) may only be moved away from (the move's nearest point to it is where it starts). `check_among`: `check_segment` and that together, the slower squeeze. `crowded_at(size, at, others)`: overlapping anyone not passed through, for choosing where to stand.
- `RoomMap::new(level, size)`: whether that size can stand at each cell's middle and how squeezed (`squeeze_at`, `has_room`, `size`). Rebuild when doors change. `RoomMap::new_by(level, size, swims)`: the same for a good swimmer, who can stand in water too (`swims()`), for their routes.

## What sizes get through (measured by the tests)

- Doorway (192): Tiny to Medium free; Large 1.56× slower; Huge 3.52× (it needs 0.75 cell, 180 units; the doorway is 0.8); Giant never (it needs a whole cell).
- One-cell corridor: up to Large free; Huge 2.25×; Giant 4×.
- Between two walls touching at a corner (42 units either side): Tiny free, Small 2.04×, Medium 3.63×, Large never.

## Depends on

- `dungeon` (`Level`, `Cell`, `WallShape`, `CELL_UNITS`, `line_hits_box`, crate-only), `creature` (`Size`), `relief` (`UNITS_PER_CELL`), `serde`.

## Used by

- `pathfinding` (`find_sized_path`, `straighten_spots`, sized `DistanceMap`s), `movement::move_on_level` (players' moves) and `mob_step_sized` (mobs' steps), `protocol` (`Spot` in messages, `JUMP_REACH`), `shop::beside_door`, `energy::site_move_cost`, `server` (visits, the Hall, saves, dev teleports' room to stand). Client `player` (free walking: `check_segment`, `slide`; routes: `RoomMap`). Server `connections::messages::moves` (`crowd_per_mille` against the mobs about, squeezes charged) and `entities::shove`; client `player` (`crowd_per_mille` in walking, steps and move costs).

## Notes

- Integers only; distances by `u128::isqrt`, rounded down, so both sides agree exactly.
- Squeezing goes by clearance round the middle (half a gap's width when centred in it), so walking close beside one wall squeezes too: routes avoid it by cost, `slide` by never squeezing further.
- Slopes, wading and terrain are priced by `movement::LevelWay`, not here: a passage gives only its squeeze. Mobs keep to the cell rule's World Gate check (`mob_step_sized`, `mob_can_stand`).
- Natural windows' columns are drawn 0.04 cell apart, too narrow for anyone, so windows stay whole cells.
