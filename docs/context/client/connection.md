# client: connection

The client's connection to the server: joining, sending the player's steps and putting right the ones the server refused, storing world map chunks, and passing messages between other modules and the server. Native clients start the embedded single-player server (`server::start_embedded`) and talk to it through an in-process `Link`; the web client has no server until networking exists, so it stays `GameView::Connecting` ("Connecting to the server..."). On quitting it hangs up and waits for the embedded server to save.

## Files

- `crates/client/src/connection/mod.rs`: `ConnectionPlugin`; `say_hello` (Startup), `receive` (PreUpdate: `Welcome` makes `common::CurrentWorld` (added after the loop, so the chunks that follow in the same batch go into it), puts the player where the server says and moves to `GameView::WorldMap`, after checking `WORLD_MAP_VERSION`; `WorldChunk` goes into `CurrentWorld::features`; `Position` goes through the `Prediction`, and moves the player if the result differs from where they are shown; `EnterLevel`, `LevelCells` and `OnWorldMap` are passed on as `FromServer`; `Refused` and a lost connection are logged), `send_steps` (Update after `PlayerSystems::Travel`: each `player::Stepped` becomes a `Step` message), `count_map_changes` (PostUpdate: `EnterLevel` and `OnWorldMap` start a new map for steps), `send` (`Last`): sends every `ToServer` written this frame, in order. `hang_up_on_exit` (`Last`, after `send`): on `AppExit`, removes `ServerConnection`.
- `crates/client/src/connection/prediction.rs`: `Prediction` (resource: map count, last step number, steps not yet confirmed; `step`, `changed_map`, `confirmed(map, step, at) -> Option<predicted>`), `correction(shown, predicted, width)` (where to move the player, if anywhere; on the world map the same place on another copy of the planet counts, and a move stays on the player's copy). Tests: confirmed steps forgotten and later ones added, a refused step moves the player back, positions from a left map ignored, the world map wraps.

## Public API

- `ToServer(ClientMessage)` (message): send this to the server (at the end of the frame).
- `FromServer(ServerMessage)` (message): a map message from the server (`EnterLevel`, `LevelCells`, `OnWorldMap`), read by `site`.
- `ServerConnection` (resource): the link to the server, and the embedded server (native); missing on the web, removed if the connection is lost and on quitting.

## Depends on

- `common` (`CurrentWorld`, `GameView`, `MapPosition`), `player` (`Player`, `PlayerSystems`, `Stepped`), `shared::movement`, `shared::protocol`, `shared::world`; native only: `server` (`start_embedded`, `EmbeddedServer`, `DEFAULT_WORLD_SEED`, `default_data_folder`).

## Used by

- `site` (`ToServer`, `FromServer`), `dev_tools` (`ToServer`); other modules see `CurrentWorld` and `GameView`.

## Notes

- `receive` runs in `PreUpdate`, so the state changes on the same frame `Welcome` arrives, and `OnEnter(GameView::WorldMap)` already finds `CurrentWorld`. `dev_tools` starting positions apply after it (on leaving `Connecting`), so they win over the saved one.
- `ServerConnection`'s fields are dropped in order: the link first (the server sees its client leave, saves and stops), then the `EmbeddedServer`, whose drop waits for that. Swapping them would hang on quitting.
- Map changes are counted in `PostUpdate`, after `site` has applied them: steps taken earlier in that frame were still on the last map. Steps sent just before a map change are never confirmed; the server ignores them.
- Corrections cancel travel (any direct change to `MapPosition` does, `player`). With straight-line travel through walls (until pathfinding), a journey stops where the server refused the step.
