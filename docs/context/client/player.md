# client: player

The player's position on the current view's grid (world map or dungeon): keyboard steps and travelling to a clicked point.

## Files

- `crates/client/src/player/mod.rs`: `PlayerPlugin`; spawns the player (with a default `Token`: white circle, `Icon` `monsters/humanarcher.png`, and a white `LightSource` of `LIGHT_RADIUS` 6 cells), `move_player` (arrows/WASD: one step on press, repeating every 50 ms while held; directions are screen directions, turned to grid steps with `common::ViewFacing`; in the dungeon view, only walkable cells can be stepped on), `travel` (constant-speed travel, sending `Arrived` at the end, `TRAVEL_SPEED` 60 units/s on the world map, `DUNGEON_TRAVEL_SPEED` 12 cells/s in the dungeon).

## Public API

- `Player` (component): marks the single player entity.
- `TravelTo { x, y }` (message): travel to a grid point at constant speed, always ending exactly on it. A new one replaces the current journey from wherever the player is.
- `Arrived { x, y }` (message): a journey (`TravelTo`) has ended there. Not sent for keyboard steps or other direct moves. Used by `site` to enter features, take stairs and leave by outside ground.
- `PlayerSystems` (system set): `Input` then `Travel`, chained. Systems that change `MapPosition` or send `TravelTo` go in `Input`; systems drawing the position run `.after(PlayerSystems::Travel)`.

## Depends on

- `common` (`Token`, `Icon`, `LightSource`, `ViewFacing`, `GameView`, `CurrentDungeon`, `MapPosition`, `SmoothPosition`: the player's position components, in `common` because other entities (world map features) have them too).

## Used by

- `site` (`Arrived`; sets the position on entering and changing levels), `view3d` (the camera follows the `Player`), `world_map` (centre of view, clicks), `dungeon_map` (sight and light), `ui` (position readout), `demo` (auto-walk), `dev_tools` (`GAME_POS`, `GAME_TRAVEL`), all with `common::MapPosition` / `SmoothPosition` on the `Player`.

## Notes

- A direct change to `MapPosition` (keyboard, demo walk, dev tools) cancels travel and jumps there: `travel` notices because `MapPosition` differs from the value it last set (private `Travel` component).
- `move_player` and `travel` don't run while `GameView::Connecting` (there is no world yet).
- Keys count if "pressed" **or** "just pressed": a tap released within one frame is otherwise lost.
- Movement is applied directly on the client; the only check is the dungeon's walkable cells (`shared::dungeon::Level::is_walkable`, the rule the server will use). Travel and other direct `MapPosition` changes don't check walls yet. Under the client/server architecture it becomes an intent sent to the server (see `architecture.md`); `TravelTo` is already intent-shaped.
- Systems use `Single<..., With<Player>>`, assuming one player entity. Multiplayer will need local vs remote players.
- A new native window takes keyboard focus, so typing elsewhere at that moment can move the player.
