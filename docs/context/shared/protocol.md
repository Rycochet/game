# shared: protocol

The messages between clients and the server, and the `Link` that carries them. Clients send intents; the server sends what the player may know. Design: "Map data" in `architecture.md`.

## Files

- `crates/shared/src/protocol/mod.rs`: `PROTOCOL_VERSION`, world map range constants, `ClientMessage`, `ServerMessage`, `WorldInfo`, `WorldChunk`. Tests: every message survives the link; closed links; report distance below the preload.
- `crates/shared/src/protocol/link.rs`: `Link`, `ClientLink`, `ServerLink`, `LinkError`, `local_pair`.

## Public API

- `PROTOCOL_VERSION` (2): bump whenever a message changes; the server refuses other versions.
- `WORLD_MAP_SIGHT` (300): how far (each way) the client may show anything on the world map. `WORLD_MAP_PRELOAD` (one chunk, 64): how much further the server sends. `WORLD_MAP_REPORT_DISTANCE` (16): how far the player moves before the client reports again; must stay below the preload.
- `ClientMessage::Hello { protocol }`: the first message on a connection. `ClientMessage::WorldMapPosition { x, y }`: TEMPORARY, until movement goes through the server: where the player is on the world map.
- `ServerMessage::Welcome(WorldInfo)`, `ServerMessage::Refused(reason)`, `ServerMessage::WorldChunk(WorldChunk)`.
- `WorldInfo { id, terrain_seed, world_map_version }`.
- `WorldChunk { chunk, feature: Option<Feature> }`: what the player knows of a world map chunk (no terrain: the client generates it). The `Feature` still includes its secret map seed, TEMPORARY until the server sends sites (step 3). Later: the chunk's changes and per-player state (visited, attitude).
- `local_pair() -> (ClientLink, ServerLink)`: an in-process connection. `Link::send(&msg) -> bool` (`false` once the other end has gone), `Link::receive() -> Result<Option<msg>, LinkError>` (`Closed`, `Garbled`); never blocks.

## Depends on

- `shared::rng` (`Seed`), `shared::world` (`WorldId`), `shared::feature` (`ChunkPos`, `Feature`, `CHUNK_SIZE`), `serde`, `postcard`, `std::sync::mpsc`.

## Used by

- `server` (`connections`, `embedded`), client `connection` (messages, `WORLD_MAP_REPORT_DISTANCE`), `common` (`WorldInfo`).

## Notes

- A `Link` carries postcard-encoded bytes, not the messages, also in single player: nothing can be shared between client and server by accident, and a network transport can later pass the same bytes to and from a socket.
- When adding a message, add one of it to the round-trip test (`every_message`).
- Compiles for wasm: the web client has the types, but no link until networking.
