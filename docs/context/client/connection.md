# client: connection

The client's connection to the server: joining, sending the player's steps and putting right the ones the server refused, storing world map chunks, and passing messages between other modules and the server. Native clients start the embedded single-player server (`server::start_embedded`) and talk to it through an in-process `Link`, or connect to a server named by `HALL_OF_WORLDS_SERVER` (`ws://...`); the web client connects over a WebSocket to the server it was loaded from. A lost connection is tried again every 2 s; `common::ServerStatus` says how it is going (the `ui` shows "Connecting..." in red). On quitting it hangs up and waits for the embedded server to save.

## Files

- `crates/client/src/connection/mod.rs`: `ConnectionPlugin`; `connect` (PreUpdate, exclusive, while there is no `ServerConnection` and the server hasn't refused: every `RETRY_EVERY` (2 s), opens the private `Connecting::target` (embedded, or a WebSocket), says `Hello` with the guest number straight away (the link holds it until the socket opens) and starts a fresh `Prediction`), `receive` (PreUpdate, after `connect`: `Welcome` sets `ServerStatus::Connected` and makes `common::CurrentWorld` (added after the loop, so the chunks that follow in the same batch go into it), puts the player where the server says and moves to `GameView::WorldMap` (if not there already), after checking `WORLD_MAP_VERSION`, and is passed on to `site`; `Refused` sets `ServerStatus::Refused` (no more tries); a lost connection removes `ServerConnection` (and the browser socket) and sets `ServerStatus::Connecting`; `WorldChunk` goes into `CurrentWorld::features`; `Position` goes through the `Prediction`, and moves the player if the result differs from where they are shown; `EnterLevel`, `LevelCells`, `OnWorldMap` and `Teleported` are passed on as `FromServer`; ), `send_steps` (Update after `PlayerSystems::Travel`: each `player::Stepped` becomes a `Step` message), `count_map_changes` (PostUpdate: `EnterLevel`, `OnWorldMap` and `Teleported` start a new map for steps), `send` (`Last`): sends every `ToServer` written this frame, in order. `hang_up_on_exit` (`Last`, after `send`): on `AppExit`, removes `ServerConnection`.
- `crates/client/src/connection/socket.rs`: `Target::{Embedded, Url}` and `choose()` (native: `HALL_OF_WORLDS_SERVER` or embedded; web: `ws(s)://<the page's host>/ws`), `open(url)` (native: a "socket" thread running `tungstenite`, plain `ws://` only, polling reads every 2 ms between sends), `BrowserSocket` (web: the browser `WebSocket`, binary messages as `ArrayBuffer`s so they stay in order; incoming go straight to the link, `pump()` sends outgoing once open; dropping it closes it), `guest()` (web: random, kept in local storage as `hall-of-worlds-guest`; native: new each run).
- `crates/client/src/connection/prediction.rs`: `Prediction` (resource: map count, last step number, steps not yet confirmed; `step`, `changed_map`, `confirmed(map, step, at) -> Option<predicted>`), `correction(shown, predicted, width)` (where to move the player, if anywhere; on the world map the same place on another copy of the planet counts, and a move stays on the player's copy). Tests: confirmed steps forgotten and later ones added, a refused step moves the player back, positions from a left map ignored, the world map wraps.

## Public API

- `ToServer(ClientMessage)` (message): send this to the server (at the end of the frame).
- `FromServer(ServerMessage)` (message): a map message from the server (`Welcome`, `EnterLevel`, `LevelCells`, `OnWorldMap`, `Teleported`), read by `site`.
- `ServerConnection` (resource): the link to the server, and the embedded server (native); removed while disconnected and on quitting.

## Depends on

- `common` (`CurrentWorld`, `GameView`, `MapPosition`), `player` (`Player`, `PlayerSystems`, `Stepped`), `shared::movement`, `shared::protocol`, `shared::world`; `common::ServerStatus`; native only: `server` (`start_embedded`, `EmbeddedServer`, `DEFAULT_WORLD_SEED`, `default_data_folder`), `tungstenite`; web only: `web-sys`, `js-sys`, `wasm-bindgen`.

## Used by

- `site` (`ToServer`, `FromServer`), `dev_tools`, `demo`, `world_map` and `dungeon_map` (`ToServer`: dev tools messages); other modules see `CurrentWorld` and `GameView`.

## Notes

- `receive` runs in `PreUpdate`, so the state changes on the same frame `Welcome` arrives, and `OnEnter(GameView::WorldMap)` already finds `CurrentWorld`. `dev_tools` starting positions apply after it (on leaving `Connecting`), so they win over the saved one.
- `ServerConnection`'s fields are dropped in order: the link first (the server sees its client leave, saves and stops), then the `EmbeddedServer`, whose drop waits for that. Swapping them would hang on quitting.
- Map changes are counted in `PostUpdate`, after `site` has applied them: steps taken earlier in that frame were still on the last map. Steps sent just before a map change are never confirmed; the server ignores them.
- Corrections cancel travel (any direct change to `MapPosition` does, `player`). With straight-line travel through walls (until pathfinding), a journey stops where the server refused the step.
- Reconnecting keeps the view on show: `Welcome` replaces `CurrentWorld` (feature tokens respawn), the player is put where the server says, and `EnterLevel` follows if they are in a site. Movement waits for `ServerStatus::Connected`.
- `dev_tools` start-up states run on leaving `GameView::Connecting`, which only happens on the first welcome.
- Native `wss://` isn't supported yet (no TLS in `tungstenite` here); browsers handle `wss://` themselves.
