# player

The player-controlled character: spawning it and moving it from keyboard input. Placeholder for now (a coloured square).

## Files

- `src/player/mod.rs`: `PlayerPlugin`; `spawn_player` (Startup), `move_player` (Update, arrows/WASD at `PLAYER_SPEED` px/s).

## Public API

- `PlayerPlugin`: registered in `main.rs`.
- `Player` (component): marks the single player entity. Other modules can query `With<Player>` to find it.

## Depends on

- `common`: assumes the camera exists.

## Used by

- Nothing yet.

## Notes

- `move_player` uses `Single<...>`, so it assumes exactly one `Player` entity. Multiplayer will need to revisit this (local player vs. remote players).
- Movement is free pixel movement, applied directly on the client. It's a placeholder: under the client/server architecture, movement becomes integer grid steps sent to the server as intents and validated there (see `architecture.md`); the client only predicts.
