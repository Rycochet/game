# client: connection

The client's connection to the server: joining, then (on the world map) reporting where the player is and storing the chunks the server sends. Native clients start the embedded single-player server (`server::start_embedded`) and talk to it through an in-process `Link`; the web client has no server until networking exists, so it stays `GameView::Connecting` ("Connecting to the server...").

## Files

- `crates/client/src/connection/mod.rs`: `ConnectionPlugin`; `say_hello` (Startup), `receive` (PreUpdate: `Welcome` inserts `common::CurrentWorld` and moves to `GameView::WorldMap`, after checking `WORLD_MAP_VERSION`; `WorldChunk` goes into `CurrentWorld::features`; `Refused` and a lost connection are logged), `report_world_map_position` (Update after `PlayerSystems::Travel`, world map only: sends `WorldMapPosition` on coming to the world map and every `WORLD_MAP_REPORT_DISTANCE`; TEMPORARY until movement goes through the server).

## Public API

- `ServerConnection` (resource): the link to the server; missing on the web, removed if the connection is lost.

## Depends on

- `common` (`CurrentWorld`, `GameView`, `MapPosition`), `player` (`Player`, `PlayerSystems`), `shared::protocol`, `shared::world`; native only: `server` (`start_embedded`, `DEFAULT_WORLD_SEED`).

## Used by

- Nothing directly; other modules see `CurrentWorld` and `GameView`. The server's map and entity messages will arrive through here.

## Notes

- `receive` runs in `PreUpdate`, so the state changes on the same frame `Welcome` arrives, and `OnEnter(GameView::WorldMap)` already finds `CurrentWorld`.
