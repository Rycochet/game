# shared: pathfinding

Finding a way across one level of a site or the world map: the quickest route of legal steps between two points, for the player's clicks now and monsters and spells later. Not between floors.

## Files

- `crates/shared/src/pathfinding.rs`: `Path`, `find_path` (A*), `straighten`, `find_world_path`, `WORLD_ROUTE_REACH`. Tests: straight lines, walls, jumps over liquid and holes, nearest cell when unreachable, allowed cells, limit, determinism, straightening; on the world map: round water, clicks on water, the reach, starting in water.

## Public API

- `Path { steps, cells, cost, reached }`: the `Step`s to take, where each lands (not including the start), the time it takes in `movement::step_cost` units, and whether it ends on the goal.
- `find_path(level, from, to, max_cost, allowed) -> Path`: the quickest route, landing only on cells `allowed` accepts. If the goal can't be reached (or is past `max_cost`), goes to the reachable cell nearest it in a straight line: check `reached`.
- `straighten(level, from, &path, allowed) -> Path`: the same route, as quick and as legal, with its diagonals spread along each stretch instead of bunched. Separate because only looks care: skip it for anything that recalculates often.
- `find_world_path(ground, conditions, from, to) -> Path`: the same on the world map (`movement::WorldGround`, `WorldConditions`), with slopes costing extra, searching only within `WORLD_ROUTE_REACH` (320 points each way, more than the view shows). Otherwise goes to the nearest point it can reach: check `reached`.

## Depends on

- `shared::dungeon` (`Level`), `shared::movement` (`Step`, `step_on_level`, `step_cost`, `WorldGround`, `world_step_cost`, the costs).

## Used by

- Client `player` (journeys in both views).

## Notes

- Moves come from `step_on_level` and costs from `step_cost`, so paths follow the movement rules by construction (corner cutting, jumps) and new rules arrive by themselves. Costs are time: straight 70, diagonal 99, a jump its distance.
- Deterministic: integers only, fixed direction order, ties settled by position, no hash maps.
- The estimate (`estimate`) must never exceed the real cost. Slower terrain is fine; terrain faster than floor would have to be assumed by the estimate.
- Asking for an unreachable goal searches everything reachable (within `max_cost`), so set a limit for monsters and spells. For many monsters chasing one goal, a distance map flooded from the goal (not built) would be cheaper than one search each; with terrain costs it must count the cost of walking towards the goal.
- World map search time (test build): 30-50 ms for a long click across hills (the slope cost makes the estimate a weaker guide); about 150 ms when the goal can't be reached (a click on water), which searches the whole area. Each point is sampled once per search, sampling being the slow part. `find_world_path` repeats `step_on_world_map`'s rule to use those samples: change them together. World routes aren't straightened: slopes make equally quick lines rare.
- `straighten` tries a line to the furthest point of each stretch and accepts it only if every step is legal and allowed and the cost is exactly the same. Jumps stay where they are.
