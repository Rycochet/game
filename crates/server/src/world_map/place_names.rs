//! Places' names on the world map (towns', castles', forts'): sent with a
//! chunk only once the character has learned the name, by going into the
//! place or (later) being told of it.

use shared::poi::ChunkPos;
use shared::protocol::{ServerMessage, WorldChunk};

use super::known_poi;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::place_name_of;

/// A chunk as this client may know it: its poi, if they know of it, and the
/// poi's name, if they have learned it.
pub(super) fn chunk_message(
    client: &Client,
    world: &ServerWorld,
    chunk: ChunkPos,
) -> ServerMessage {
    let poi = known_poi(client, world, chunk);
    let named = client
        .character
        .pois
        .get(&chunk)
        .is_some_and(|known| known.knows_name());
    let name = poi
        .filter(|_| named)
        .and_then(|poi| place_name_of(world, &poi));
    ServerMessage::WorldChunk(WorldChunk {
        chunk,
        poi: poi.map(Into::into),
        name,
    })
}

/// The character learns the name of the place in `chunk` (going into it, or
/// later being told of it by a quest), and the client is sent it at once if
/// it has that chunk. `false` if the client has gone.
pub(crate) fn learn_place_name(client: &mut Client, world: &ServerWorld, chunk: ChunkPos) -> bool {
    let known = client.character.pois.entry(chunk).or_default();
    if known.named {
        return true;
    }
    known.named = true;
    client.unsaved = true;
    !client.world_chunks.contains(&chunk) || client.link.send(&chunk_message(client, world, chunk))
}
