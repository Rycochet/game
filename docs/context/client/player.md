# client: player

The player's position on the current view's grid (world map or dungeon): keyboard steps and travelling to a clicked point.

## Files

- `crates/client/src/player/mod.rs`: `PlayerPlugin`; spawns the player (with a default `Token`: white circle, `Icon` `monsters/humanarcher.png`, and a white `LightSource` of `LIGHT_RADIUS` 6 cells), `move_player` (arrows/WASD: a `TakeStep` on press, repeating every 50 ms while held; directions are screen directions, turned to grid steps with `common::ViewFacing`), `take_steps` (in `Travel`, before `travel`: takes this frame's `TakeStep`s in order, in the dungeon view only onto walkable cells or over liquid to one (`shared::movement::step_on_level`), on the world map not into water (`step_on_world_map`), announcing each as `Stepped`), `travel` (travel along a found route, at constant speed except that world map slopes are slower by what they cost; announcing each grid point it moves to as `Stepped` (several if a frame covers more), sending `Arrived` at the end, and showing jumps, `TRAVEL_SPEED` 60 units/s on the world map, `DUNGEON_TRAVEL_SPEED` 12 cells/s in the dungeon).

## Public API

- `Player` (component): marks the single player entity.
- `TravelTo { x, y }` (message): travel to a grid point by the quickest route, or to the nearest point it can reach: in the dungeon through cells the player has seen, on the world map round water. A new one replaces the current journey from wherever the player is.
- `Arrived { x, y }` (message): a journey (`TravelTo`) has ended there, where it was asked to go. Not sent for keyboard steps or other direct moves, nor when it stopped short (so clicking a wall beside stairs doesn't take them). Used by `site` to enter features, take stairs and leave by outside ground.
- `TakeStep(Step)` (message): ask for a step (keyboard, demo walk, dev tools).
- `Stepped { step, moved }` (message): the player took a step (by `TakeStep` or travelling), which moved them by `moved` (two cells for a jump); `connection` sends it to the server. Not sent when the position is set from elsewhere (the server's word: arriving on a level, corrections).
- `PlayerSystems` (system set): `Input` then `Travel`, chained. Systems that send `TakeStep` or `TravelTo` go in `Input`; systems drawing the position or reading `Stepped` run `.after(PlayerSystems::Travel)`.

## Depends on

- `shared::movement`, `shared::pathfinding`; `common` (`Token`, `Icon`, `LightSource`, `ViewFacing`, `GameView`, `CurrentDungeon`, `CurrentWorld`, `MapPosition`, `SmoothPosition`: the player's position components, in `common` because other entities (world map features) have them too).

## Used by

- `connection` (`Stepped`; sets the position on joining and on corrections), `site` (`Arrived`; sets the position on entering and changing levels), `view3d` (the camera follows the `Player`), `world_map` (centre of view, clicks), `dungeon_map` (sight and light), `ui` (position readout), `demo` (auto-walk: `TakeStep`), `dev_tools` (`GAME_MOVE`: `TakeStep`; `GAME_TRAVEL`), all with `common::MapPosition` / `SmoothPosition` on the `Player`.

## Notes

- Any other change to `MapPosition` (a step, the server putting the player somewhere) cancels travel and jumps there: `travel` notices because `MapPosition` differs from the value it last set (private `Travel` component).
- `move_player` and `travel` don't run while `GameView::Connecting` (there is no world yet), nor while `common::ServerStatus` isn't `Connected`.
- Keys count if "pressed" **or** "just pressed": a tap released within one frame is otherwise lost.
- Movement is predicted: steps are shown at once and the server checks them again (`connection` moves the player back if it refused one). Unknown cells count as wall here, so the client is stricter than the server. World map routes use the same rules as the server (`CurrentWorld::ground`, public terrain, the water at `GameClock`'s hour), so they aren't limited to what has been seen. If the tide cuts a route, a new one is found from there to the same goal (private `Travel::goal`). In the dungeon a route uses only `CurrentDungeon::remembered` cells, so it gives nothing away, and each step is checked again when taken: if the cells have changed, the journey ends there. The route isn't looked at again as more of the level is seen.
- Jumps over liquid: `MapPosition` goes straight to the cell landed on and `SmoothPosition` glides there at travel speed. Until the jump is over, steps are ignored and a new `TravelTo` waits. A journey jumps on leaving the cell before the liquid and carries on from the edge of the landing cell; a keyboard jump ends on the landing cell's centre.
- Systems use `Single<..., With<Player>>`, assuming one player entity. Multiplayer will need local vs remote players.
- A new native window takes keyboard focus, so typing elsewhere at that moment can move the player.
