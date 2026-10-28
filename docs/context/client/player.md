# client: player

The player's position on the current view's grid (world map or dungeon): keyboard steps and travelling to a clicked point.

## Files

- `crates/client/src/player/mod.rs`: `PlayerPlugin`; spawns the player (with a default `Token`: white circle, `Icon` `monsters/humanarcher.png`, and a white `LightSource` of `LIGHT_RADIUS` 6 cells), `move_player` (arrows/WASD: one step on press, repeating every 50 ms while held; directions are screen directions, turned to grid steps with `common::ViewFacing`; in the dungeon view, steps into walls are refused), `travel` (constant-speed travel, `TRAVEL_SPEED` 60 units/s on the world map, `DUNGEON_TRAVEL_SPEED` 12 cells/s in the dungeon).

## Public API

- `Player` (component): marks the single player entity.
- `MapPosition { x: i64, y: i64 }` (component): position on the current view's grid; `y` increases southwards (towards the viewer). While travelling it is the nearest grid point to `SmoothPosition`.
- `SmoothPosition { x: f64, y: f64 }` (component): exact position, fractional while travelling, equal to `MapPosition` when still. For smooth drawing only.
- `TravelTo { x, y }` (message): travel to a grid point at constant speed, always ending exactly on it. A new one replaces the current journey from wherever the player is.
- `PlayerSystems` (system set): `Input` then `Travel`, chained. Systems that change `MapPosition` or send `TravelTo` go in `Input`; systems drawing the position run `.after(PlayerSystems::Travel)`.

## Depends on

- `common` (`Token`, `Icon`, `LightSource`, `ViewFacing`, `GameView`, `CurrentDungeon`).

## Used by

- `view3d` (camera and token positions), `world_map` (centre of view, clicks, token heights), `dungeon_map` (sight and light), `ui` (position readout), `demo` (auto-walk, view switch), `dev_tools` (`GAME_POS`, `GAME_TRAVEL`).

## Notes

- A direct change to `MapPosition` (keyboard, demo walk, dev tools) cancels travel and jumps there: `travel` notices because `MapPosition` differs from the value it last set (private `Travel` component).
- Keys count if "pressed" **or** "just pressed": a tap released within one frame is otherwise lost.
- Movement is applied directly on the client; the only check is the dungeon's walls (`shared::dungeon::Level::is_open`, the rule the server will use). Travel and other direct `MapPosition` changes don't check walls yet. Under the client/server architecture it becomes an intent sent to the server (see `architecture.md`); `TravelTo` is already intent-shaped.
- Systems use `Single<..., With<Player>>`, assuming one player entity. Multiplayer will need local vs remote players.
- A new native window takes keyboard focus, so typing elsewhere at that moment can move the player.
