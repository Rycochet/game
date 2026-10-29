# shared: protocol

The messages between clients and the server, and the `Link` that carries them. Clients send intents; the server sends what the player may know. Design: "Map data" in `architecture.md`.

## Files

- `crates/shared/src/protocol/mod.rs`: `PROTOCOL_VERSION`, `ClientMessage`, `ServerMessage`, `WorldInfo`. Tests: every message survives the link; closed links.
- `crates/shared/src/protocol/link.rs`: `Link`, `ClientLink`, `ServerLink`, `LinkError`, `local_pair`.

## Public API

- `PROTOCOL_VERSION` (1): bump whenever a message changes; the server refuses other versions.
- `ClientMessage::Hello { protocol }`: the first message on a connection.
- `ServerMessage::Welcome(WorldInfo)`, `ServerMessage::Refused(reason)`.
- `WorldInfo { id, terrain_seed, world_map_version, features_seed }`. `features_seed` is TEMPORARY (and secret): until the server sends features (step 2 of the map data plan), the client still places features and generates sites itself.
- `local_pair() -> (ClientLink, ServerLink)`: an in-process connection. `Link::send(&msg) -> bool` (`false` once the other end has gone), `Link::receive() -> Result<Option<msg>, LinkError>` (`Closed`, `Garbled`); never blocks.

## Depends on

- `shared::rng` (`Seed`), `shared::world` (`WorldId`), `serde`, `postcard`, `std::sync::mpsc`.

## Used by

- `server` (`connections`, `embedded`), client `connection`, `common` (`WorldInfo`).

## Notes

- A `Link` carries postcard-encoded bytes, not the messages, also in single player: nothing can be shared between client and server by accident, and a network transport can later pass the same bytes to and from a socket.
- When adding a message, add one of it to the round-trip test (`every_message`).
- Compiles for wasm: the web client has the types, but no link until networking.
