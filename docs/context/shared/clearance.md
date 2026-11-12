# shared: clearance (and spot)

Positions on a level in whole units (`Spot`, 240 a cell) and room to move for creatures of each size: what blocks them, the one rule for a move from one spot to another (with squeezing and jumps), sliding along what is in the way, and where each size has room. Free positions, PR 2 of 9 (`docs/context/free-positions.md`): **nothing in play calls these yet**; levels still move in whole-cell `movement::Step`s. Routes by size are in `pathfinding` (`find_sized_path`).

## Files

- `crates/shared/src/spot.rs`: `Spot`, `CELL_SPAN`, `HALF_SPAN`, `root`.
- `crates/shared/src/clearance.rs` and `clearance/`: `blocker.rs` (`Blocker`, `Shape`, how close a move passes one), `cell_blockers.rs` (what each cell puts in the way; `blockers_near`), `footing.rs` (`has_footing`), `segment.rs` (`check_segment`, `Passage`, `segment_is_legal`, `segment_cost`, `JUMP_REACH`), `slide.rs`, `squeeze.rs`, `room_map.rs` (`RoomMap`). Golden tests: radii, a one-cell corridor and a doorway by size, walls touching at a corner, trunks, jumps, fences, sliding, room maps.

## Public API

- `Spot { x, y }` (serde, i64 units, `x` east, `y` south): `middle_of(cell)` (cell `(x, y)`'s middle at `(x, y) × 240`, as `relief::Relief::at`), `cell()` (a spot on a cell's edge is in the cell east or south), `offset`, `distance_squared` (exact, `u128`), `distance` (rounded down), `towards(other, n, d)`. On the Hall's levels `x` is the Hall's own, unwrapped, as its cells are (`hall::wrap_x` brings a cell round). `root(n)`: whole-number square root.
- `Size::radius()` (in `creature`): Tiny 20, Small 60, Medium 80 (today's token base, 2/3 cell across), Large 120, Huge 180, Giant 240. Drawn base = blocking circle.
- `Blocker { shape, fence }`, `Shape::{Box { low, high }, Post { at, radius }}`, `.clearance(a, b)`: how far a move passes it (0 touching). `cell_blockers(level, x, y, &mut out)`, `blockers_near(level, low, high, reach)`:
  - walls (and building doors, blind windows, closed secret doors), windows and closed doors fill the cell, less a `CORNER_CUT` (60) notch at each corner with no wall, window or doorway beside it on either side;
  - an open or broken door (an open secret door too) leaves two frame posts, the doorway `2 × DOOR_HALF_GAP` (192 units, 0.8 cell, as `dungeon_map` draws it) wide, the full cell deep; leaves don't block (owner);
  - a fence is two bars `FENCE_HALF` (12) thick, shaped as a wall's (`WallShape::reach` × 6), running to fences and field gates beside it, and can be jumped (`fence: Some(cell)`);
  - a tree is its trunk, a `TRUNK_RADIUS` (30) post at the middle.
- `has_footing(level, x, y)`: where a middle may stand: walkable cells, and fence and tree cells (beside the bar or trunk). Liquid and holes can only be jumped.
- `check_segment(level, size, from, to) -> Option<Passage { squeeze_per_mille, jumped }>`: the rule. Both ends need footing; the middle's line may cross the inside of no cell without footing, unless it is a jump: over exactly one jumpable cell (liquid, hole, or a fence whose bar it crosses), at most `JUMP_REACH` (679, two cells diagonally). The swept circle keeps at least half its radius from every blocker but the jumped fence. `segment_is_legal` (yes or no), `segment_cost` (length × squeeze / 1000).
- `squeeze_per_mille(radius, clearance)` (owner, 2026-11-10): clearance ≥ radius is free (1000); down to half the radius (a gap half its width), slower by (radius / clearance)², at most `MOST_SQUEEZE_PER_MILLE` (4000); less is `None`.
- `slide(level, size, from, to) -> Spot`: for the client's movement. `to` if the move is a legal walk (squeezed or not); otherwise as far along as it goes, then on along east-west or north-south, whichever gets further, never more squeezed than at `from` (so pushing into a wall doesn't press into it). Never jumps. `from` if it has no room there.
- `RoomMap::new(level, size)`: whether that size can stand at each cell's middle and how squeezed (`squeeze_at`, `has_room`, `size`). Rebuild when doors change.

## What sizes get through (measured by the tests)

- Doorway (192): Tiny to Medium free; Large 1.56× slower; Huge 3.52× (it needs 0.75 cell, 180 units; the doorway is 0.8); Giant never (it needs a whole cell).
- One-cell corridor: up to Large free; Huge 2.25×; Giant 4×.
- Between two walls touching at a corner (42 units either side): Tiny free, Small 2.04×, Medium 3.63×, Large never.

## Depends on

- `dungeon` (`Level`, `Cell`, `WallShape`, `CELL_UNITS`, `line_hits_box`, crate-only), `creature` (`Size`), `relief` (`UNITS_PER_CELL`), `serde`.

## Used by

- `pathfinding` (`find_sized_path`, `straighten_spots`). Nothing in play yet: PR 3 (positions in units end to end), PR 4 (free movement on the client: `slide`), PR 5 (sizes in play: room maps, squeezing, mobs).

## Notes

- Integers only; distances by `u128::isqrt`, rounded down, so both sides agree exactly.
- Squeezing goes by clearance round the middle (half a gap's width when centred in it), so walking close beside one wall squeezes too: routes avoid it by cost, `slide` by never squeezing further.
- Not yet here: mobs' rule against World Gates' cells, deep water for good swimmers (`movement::step_on_level_by`), slopes in costs (`relief::Relief::at`), creatures blocking each other (PR 5).
- Natural windows' columns are drawn 0.04 cell apart, too narrow for anyone, so windows stay whole cells.
