# client: connection

The client's connection to the server: joining, reporting where the player is (on the world map and in sites), storing world map chunks, and passing messages between other modules and the server. Native clients start the embedded single-player server (`server::start_embedded`) and talk to it through an in-process `Link`; the web client has no server until networking exists, so it stays `GameView::Connecting` ("Connecting to the server..."). On quitting it hangs up and waits for the embedded server to save.

## Files

- `crates/client/src/connection/mod.rs`: `ConnectionPlugin`; `say_hello` (Startup), `receive` (PreUpdate: `Welcome` inserts `common::CurrentWorld`, puts the player where the server says and moves to `GameView::WorldMap`, after checking `WORLD_MAP_VERSION`; `WorldChunk` goes into `CurrentWorld::features`; `EnterLevel`, `LevelCells` and `LeftSite` are passed on as `FromServer`; `Refused` and a lost connection are logged), `report_world_map_position` (Update after `PlayerSystems::Travel`, world map only: sends `WorldMapPosition` on coming to the world map and every `WORLD_MAP_REPORT_DISTANCE`), `report_site_position` (in sites: `SitePosition` whenever the player moves to another cell); both TEMPORARY until movement goes through the server. `send` (`Last`): sends every `ToServer` written this frame, in order. `hang_up_on_exit` (`Last`, after `send`): on `AppExit`, reports the exact world map position, then removes `ServerConnection`.

## Public API

- `ToServer(ClientMessage)` (message): send this to the server (at the end of the frame).
- `FromServer(ServerMessage)` (message): a map message from the server (`EnterLevel`, `LevelCells`, `LeftSite`), read by `site`.
- `ServerConnection` (resource): the link to the server, and the embedded server (native); missing on the web, removed if the connection is lost and on quitting.

## Depends on

- `common` (`CurrentWorld`, `GameView`, `MapPosition`), `player` (`Player`, `PlayerSystems`), `shared::protocol`, `shared::world`; native only: `server` (`start_embedded`, `EmbeddedServer`, `DEFAULT_WORLD_SEED`, `default_data_folder`).

## Used by

- `site` (`ToServer`, `FromServer`), `dev_tools` (`ToServer`); other modules see `CurrentWorld` and `GameView`.

## Notes

- `receive` runs in `PreUpdate`, so the state changes on the same frame `Welcome` arrives, and `OnEnter(GameView::WorldMap)` already finds `CurrentWorld`. `dev_tools` starting positions apply after it (on leaving `Connecting`), so they win over the saved one.
- `ServerConnection`'s fields are dropped in order: the link first (the server sees its client leave, saves and stops), then the `EmbeddedServer`, whose drop waits for that. Swapping them would hang on quitting.
- The world map position is only reported every `WORLD_MAP_REPORT_DISTANCE`, so a killed game (not quit) saves up to that far behind.
