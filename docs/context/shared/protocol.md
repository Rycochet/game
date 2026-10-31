# shared: protocol

The messages between clients and the server, and the `Link` that carries them. Clients send intents; the server sends what the player may know. Design: "Map data" in `architecture.md`.

## Files

- `crates/shared/src/protocol/mod.rs`: `PROTOCOL_VERSION`, world map range constants, `ClientMessage`, `ServerMessage`, `WorldInfo`, `WorldChunk`. Tests: every message survives the link; closed links; report distance below the preload.
- `crates/shared/src/protocol/level.rs`: site levels as sent: `LEVEL_CHUNK_SIZE` (8), `ENTER_REACH` (2), `level_chunk(x, y) -> (chunk, bit)`, `LevelInfo`, `LevelChunk` (`positions()`, `cells()`), `RememberedLevel`, `SeenStairs`. Test: cells come back where they were.
- `crates/shared/src/protocol/link.rs`: `Link`, `ClientLink`, `ServerLink`, `LinkError`, `local_pair`, `Wire`, `wired`. Tests (in `mod.rs`): every message survives the link and a wire, garbled bytes and closed ends noticed.

`Welcome` carries the world's float clock; `Hour` says a new whole hour has begun (the tide); `Ping { sent }` is answered by `Pong { sent, clock }`.

## Public API

- `SOCKET_PATH` (`/ws`): where a server's WebSocket is.
- `PROTOCOL_VERSION` (13): bump whenever a message changes; the server refuses other versions.
- `WORLD_MAP_SIGHT` (300): how far (each way) the client may show anything on the world map. `WORLD_MAP_PRELOAD` (one chunk, 64): how much further the server sends.
- `ClientMessage::DevSnowLine(n)` (dev tools: answered with `SnowLine`). `ClientMessage::Hello { protocol, guest }`: the first message on a connection; `guest` names the character on servers without accounts (the embedded server ignores it). `ClientMessage::Step { map, number, step }`: a step (`movement::Step`) on the current map; `map` counts the maps the server has put the player on since `Welcome` (each `EnterLevel`, `OnWorldMap` and `Teleported`), so steps meant for the last map are ignored; `number` counts the client's steps. `EnterSite { chunk }` (the player must be within `ENTER_REACH` of its poi), `UseExit` (the stairs or edge ground the player stands on), Dev tools, only accepted by an embedded server (others ignore them; `is_dev_tool()`): `DevTeleport { x, y }` (to a point on the current map, in a site only a walkable cell; answered with `Teleported`), `DevBackToStart` (to where new characters start on the world map, out of any site; answered with `OnWorldMap`), `DevEnterSite { kind, seed, level }`.
- `ServerMessage::Hour(hour)` (a new game hour, which sets the tide), `SnowLine(n)` (the snow line has moved), `Welcome { world: WorldInfo, hour, snow_line, x, y }` (the world, its hour and snow line, and where the player is on its world map: where they left off; `EnterLevel` follows if they were in a site), `Refused(reason)`, `WorldChunk(WorldChunk)`, `EnterLevel(LevelInfo)` (the player is now on this level: kind, building or not, index of how many, bounds, arrival cell, and `entering_site`: come into the site or back on joining, rather than by its stairs), `Buildings(Vec<BuildingLook>)` (a town's, after `EnterLevel` and before its cells), `LevelCells(LevelChunk)`, `RememberedLevel { index, bounds, chunks }` (what the character remembers of another level of the site, with the neighbours that shape it, after `EnterLevel` when `entering_site`; for the maps), `OnWorldMap { x, y }` (on the world map: out of a site at the poi, or back to the start), `Teleported { x, y }` (dev tools: jumped to here on the current map), `Position { map, step, x, y }` (where the server has the player once it has dealt with step `step`, sent once a tick when steps arrived: the client adds its later steps and moves the player there if it shows them elsewhere).
- `WorldInfo { id, terrain_seed, world_map_version }`.
- `WorldChunk { chunk, poi: Option<SeenPoi> }`: what the player knows of a world map chunk (no terrain: the client generates it). `SeenPoi { kind, look, chunk, x, y }` (`From<Poi>`): never the map seed; `look` is `site::PoiLook`. A chunk's `poi` is `None` for a poi the character doesn't know of, as if there were none. Later: the chunk's changes and per-player state (visited, attitude).
- `LevelChunk { chunk, mask: u64, cells, stairs, remembered }`: some cells of an 8×8 chunk (bit `row × 8 + column`), as the player would see them (secret doors are wall), with the stairs on them; `remembered` for cells seen on an earlier visit, sent on arriving. `SeenStairs { x, y, direction, descends, style }`: where stairs lead stays with the server.
- `local_pair() -> (ClientLink, ServerLink)`: an in-process connection. `wired() -> (Link, Wire)`: a link whose bytes a `Wire` carries elsewhere (a WebSocket): `deliver(bytes)`, `take()`, or `into_parts()` (a sender for incoming bytes and a receiver of outgoing ones, for different threads); either side going is noticed by the other. `Link::send(&msg) -> bool` (`false` once the other end has gone), `Link::receive() -> Result<Option<msg>, LinkError>` (`Closed`, `Garbled`); never blocks.

## Depends on

- `shared::movement` (`Step`), `shared::rng` (`Seed`), `shared::world` (`WorldId`), `shared::poi` (`ChunkPos`, `Poi`, `PoiKind`, `CHUNK_SIZE`), `shared::dungeon` (`Cell`), `shared::site` (`StairsDirection`, `BuildingLook`), `serde`, `postcard`, `std::sync::mpsc`.

## Used by

- `server` (`connections`, `world_map`, `sites`, `embedded`), client `connection` (messages), `common` (`WorldInfo`, `SeenPoi`, `LevelInfo`, `LevelChunk`, `SeenStairs`), `site`, `dev_tools`, `dungeon_map` (`SeenStairs`), `world_map` (`SeenPoi`, `WORLD_MAP_SIGHT`).

## Notes

- A `Link` carries postcard-encoded bytes, not the messages, also in single player: nothing can be shared between client and server by accident, and over the network the same bytes are one WebSocket binary message each.
- When adding a message, add one of it to the round-trip test (`every_message`).
- Compiles for wasm: the web client uses the same types and `wired()` links over the browser's WebSocket.
