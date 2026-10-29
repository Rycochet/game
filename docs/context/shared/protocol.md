# shared: protocol

The messages between clients and the server, and the `Link` that carries them. Clients send intents; the server sends what the player may know. Design: "Map data" in `architecture.md`.

## Files

- `crates/shared/src/protocol/mod.rs`: `PROTOCOL_VERSION`, world map range constants, `ClientMessage`, `ServerMessage`, `WorldInfo`, `WorldChunk`. Tests: every message survives the link; closed links; report distance below the preload.
- `crates/shared/src/protocol/level.rs`: site levels as sent: `LEVEL_CHUNK_SIZE` (8), `LEVEL_PRELOAD_STEPS` (2), `ENTER_REACH` (2), `level_chunk(x, y) -> (chunk, bit)`, `LevelInfo`, `LevelChunk` (`positions()`, `cells()`), `SeenStairs`. Test: cells come back where they were.
- `crates/shared/src/protocol/link.rs`: `Link`, `ClientLink`, `ServerLink`, `LinkError`, `local_pair`.

## Public API

- `PROTOCOL_VERSION` (3): bump whenever a message changes; the server refuses other versions.
- `WORLD_MAP_SIGHT` (300): how far (each way) the client may show anything on the world map. `WORLD_MAP_PRELOAD` (one chunk, 64): how much further the server sends. `WORLD_MAP_REPORT_DISTANCE` (16): how far the player moves before the client reports again; must stay below the preload.
- `ClientMessage::Hello { protocol }`: the first message on a connection. `ClientMessage::WorldMapPosition { x, y }` and `SitePosition { x, y }`: TEMPORARY, until movement goes through the server: where the player is (the server accepts only walkable site cells). `EnterSite { chunk }` (the player must be within `ENTER_REACH` of its feature), `UseExit` (the stairs or edge ground the player stands on), `DevEnterSite { kind, seed, level }` (only an embedded server accepts it).
- `ServerMessage::Welcome(WorldInfo)`, `Refused(reason)`, `WorldChunk(WorldChunk)`, `EnterLevel(LevelInfo)` (the player is now on this level: kind, building or not, index of how many, bounds, arrival cell), `LevelCells(LevelChunk)`, `LeftSite { x, y }` (back on the world map, at the feature).
- `WorldInfo { id, terrain_seed, world_map_version }`.
- `WorldChunk { chunk, feature: Option<SeenFeature> }`: what the player knows of a world map chunk (no terrain: the client generates it). `SeenFeature { kind, chunk, x, y }` (`From<Feature>`): never the map seed. Later: the chunk's changes and per-player state (visited, attitude).
- `LevelChunk { chunk, mask: u64, cells, stairs, remembered }`: some cells of an 8×8 chunk (bit `row × 8 + column`), as the player would see them (secret doors are wall), with the stairs on them; `remembered` for cells seen on an earlier visit, sent on arriving. `SeenStairs { x, y, direction, descends }`: where stairs lead stays with the server.
- `local_pair() -> (ClientLink, ServerLink)`: an in-process connection. `Link::send(&msg) -> bool` (`false` once the other end has gone), `Link::receive() -> Result<Option<msg>, LinkError>` (`Closed`, `Garbled`); never blocks.

## Depends on

- `shared::rng` (`Seed`), `shared::world` (`WorldId`), `shared::feature` (`ChunkPos`, `Feature`, `FeatureKind`, `CHUNK_SIZE`), `shared::dungeon` (`Cell`), `shared::site` (`StairsDirection`), `serde`, `postcard`, `std::sync::mpsc`.

## Used by

- `server` (`connections`, `world_map`, `sites`, `embedded`), client `connection` (messages, `WORLD_MAP_REPORT_DISTANCE`), `common` (`WorldInfo`, `SeenFeature`, `LevelInfo`, `LevelChunk`, `SeenStairs`), `site`, `dev_tools`, `dungeon_map` (`SeenStairs`), `world_map` (`SeenFeature`, `WORLD_MAP_SIGHT`).

## Notes

- A `Link` carries postcard-encoded bytes, not the messages, also in single player: nothing can be shared between client and server by accident, and a network transport can later pass the same bytes to and from a socket.
- When adding a message, add one of it to the round-trip test (`every_message`).
- Compiles for wasm: the web client has the types, but no link until networking.
