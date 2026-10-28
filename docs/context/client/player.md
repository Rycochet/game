# client: player

The player's position on the world grid: keyboard steps and travelling to a clicked point.

## Files

- `crates/client/src/player/mod.rs`: `PlayerPlugin`; spawns the player, `move_player` (arrows/WASD: one step on press, repeating every 50 ms while held), `travel` (constant-speed travel, `TRAVEL_SPEED` 60 units/s).

## Public API

- `Player` (component): marks the single player entity.
- `MapPosition { x: i64, y: i64 }` (component): grid position; `y` increases southwards (towards the viewer). While travelling it is the nearest grid point to `SmoothPosition`.
- `SmoothPosition { x: f64, y: f64 }` (component): exact position, fractional while travelling, equal to `MapPosition` when still. For smooth drawing only.
- `TravelTo { x, y }` (message): travel to a grid point at constant speed, always ending exactly on it. A new one replaces the current journey from wherever the player is.
- `PlayerSystems` (system set): `Input` then `Travel`, chained. Systems that change `MapPosition` or send `TravelTo` go in `Input`; systems drawing the position run `.after(PlayerSystems::Travel)`.

## Depends on

- Nothing beyond Bevy.

## Used by

- `world_map` (centre of view, clicks), `ui` (position readout), `demo` (auto-walk), `dev_tools` (`GAME_POS`, `GAME_TRAVEL`).

## Notes

- A direct change to `MapPosition` (keyboard, demo walk, dev tools) cancels travel and jumps there: `travel` notices because `MapPosition` differs from the value it last set (private `Travel` component).
- Keys count if "pressed" **or** "just pressed": a tap released within one frame is otherwise lost.
- Movement is applied directly on the client with no validation. Under the client/server architecture it becomes an intent sent to the server (see `architecture.md`); `TravelTo` is already intent-shaped.
- Systems use `Single<..., With<Player>>`, assuming one player entity. Multiplayer will need local vs remote players.
- A new native window takes keyboard focus, so typing elsewhere at that moment can move the player.
