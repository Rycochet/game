# client: player

The player's position on the world grid and keyboard movement.

## Files

- `crates/client/src/player/mod.rs`: `PlayerPlugin`; spawns the player, `move_player` (arrows/WASD: one step on press, repeating every 50 ms while held).

## Public API

- `Player` (component): marks the single player entity.
- `MapPosition { x: i64, y: i64 }` (component): grid position; `y` increases southwards (towards the viewer).

## Depends on

- Nothing beyond Bevy.

## Used by

- `slice_view`, `mesh_view` (centre of view), `ui` (position readout), `dev_tools` (`GAME_POS`).

## Notes

- Keys count if "pressed" **or** "just pressed": a tap released within one frame is otherwise lost.
- Movement is applied directly on the client with no validation. Under the client/server architecture it becomes an intent sent to the server (see `architecture.md`).
- Systems use `Single<..., With<Player>>`, assuming one player entity. Multiplayer will need local vs remote players.
- A new native window takes keyboard focus, so typing elsewhere at that moment can move the player.
