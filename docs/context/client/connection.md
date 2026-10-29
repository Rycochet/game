# client: connection

The client's connection to the server: joining, reporting where the player is (on the world map and in sites), storing world map chunks, and passing messages between other modules and the server. Native clients start the embedded single-player server (`server::start_embedded`) and talk to it through an in-process `Link`; the web client has no server until networking exists, so it stays `GameView::Connecting` ("Connecting to the server...").

## Files

- `crates/client/src/connection/mod.rs`: `ConnectionPlugin`; `say_hello` (Startup), `receive` (PreUpdate: `Welcome` inserts `common::CurrentWorld` and moves to `GameView::WorldMap`, after checking `WORLD_MAP_VERSION`; `WorldChunk` goes into `CurrentWorld::features`; `EnterLevel`, `LevelCells` and `LeftSite` are passed on as `FromServer`; `Refused` and a lost connection are logged), `report_world_map_position` (Update after `PlayerSystems::Travel`, world map only: sends `WorldMapPosition` on coming to the world map and every `WORLD_MAP_REPORT_DISTANCE`), `report_site_position` (in sites: `SitePosition` whenever the player moves to another cell); both TEMPORARY until movement goes through the server. `send` (`Last`): sends every `ToServer` written this frame, in order.

## Public API

- `ToServer(ClientMessage)` (message): send this to the server (at the end of the frame).
- `FromServer(ServerMessage)` (message): a map message from the server (`EnterLevel`, `LevelCells`, `LeftSite`), read by `site`.
- `ServerConnection` (resource): the link to the server; missing on the web, removed if the connection is lost.

## Depends on

- `common` (`CurrentWorld`, `GameView`, `MapPosition`), `player` (`Player`, `PlayerSystems`), `shared::protocol`, `shared::world`; native only: `server` (`start_embedded`, `DEFAULT_WORLD_SEED`).

## Used by

- `site` (`ToServer`, `FromServer`), `dev_tools` (`ToServer`); other modules see `CurrentWorld` and `GameView`.

## Notes

- `receive` runs in `PreUpdate`, so the state changes on the same frame `Welcome` arrives, and `OnEnter(GameView::WorldMap)` already finds `CurrentWorld`.
