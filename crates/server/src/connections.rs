//! Connected clients: reading their messages and answering them, and
//! sending each player what comes into their sight.

use std::collections::BTreeSet;

use bevy::prelude::*;
use shared::feature::ChunkPos;
use shared::protocol::{
    ClientMessage, PROTOCOL_VERSION, ServerLink, ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT,
    WorldChunk,
};

use crate::ServerWorld;

/// The clients connected to this server.
#[derive(Resource, Default)]
pub struct Connections(Vec<Client>);

impl Connections {
    /// A newly connected client.
    pub fn add(&mut self, link: ServerLink) {
        self.0.push(Client {
            link,
            welcomed: false,
            world_chunks: BTreeSet::new(),
        });
    }
}

/// One connected client and what it has been sent.
pub struct Client {
    link: ServerLink,
    /// Whether it has said `Hello` (and been welcomed).
    welcomed: bool,
    /// The world map chunks it has been sent.
    world_chunks: BTreeSet<ChunkPos>,
}

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused).
pub(crate) fn handle_messages(mut connections: ResMut<Connections>, world: Res<ServerWorld>) {
    connections.0.retain_mut(|client| {
        loop {
            match client.link.receive() {
                Ok(Some(message)) => {
                    if !answer(client, message, &world) {
                        return false;
                    }
                }
                Ok(None) => return true,
                Err(error) => {
                    info!("A client left ({error:?})");
                    return false;
                }
            }
        }
    });
}

/// Answers one message; `false` if the client should be disconnected.
fn answer(client: &mut Client, message: ClientMessage, world: &ServerWorld) -> bool {
    let refuse = |reason: String| {
        client.link.send(&ServerMessage::Refused(reason));
        false
    };
    match message {
        ClientMessage::Hello { protocol } if protocol != PROTOCOL_VERSION => refuse(format!(
            "this server speaks protocol {PROTOCOL_VERSION}, not {protocol}"
        )),
        ClientMessage::Hello { .. } => {
            client.welcomed = true;
            client.link.send(&ServerMessage::Welcome(world.info()))
        }
        _ if !client.welcomed => refuse("say hello first".into()),
        ClientMessage::WorldMapPosition { x, y } => send_world_chunks(client, world, x, y),
    }
}

/// Sends the world map chunks within sight of `(x, y)`, plus a margin, that
/// the client hasn't been sent yet.
fn send_world_chunks(client: &mut Client, world: &ServerWorld, x: i64, y: i64) -> bool {
    let reach = WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD;
    let chunks = world
        .features
        .chunks_in_area((x - reach, y - reach), (x + reach, y + reach));
    for chunk in chunks {
        if client.world_chunks.insert(chunk) {
            let feature = world.features.in_chunk(chunk);
            if !client
                .link
                .send(&ServerMessage::WorldChunk(WorldChunk { chunk, feature }))
            {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use shared::protocol::local_pair;
    use shared::world::WorldSeed;

    use super::*;

    /// Sends `messages` from a new client and returns everything answered.
    fn exchange(messages: &[ClientMessage]) -> Vec<ServerMessage> {
        let (client_link, server_link) = local_pair();
        let world = ServerWorld::new(WorldSeed::new(1));
        let mut client = Client {
            link: server_link,
            welcomed: false,
            world_chunks: BTreeSet::new(),
        };
        for message in messages {
            answer(&mut client, message.clone(), &world);
        }
        std::iter::from_fn(|| client_link.receive().unwrap()).collect()
    }

    fn chunks(answers: &[ServerMessage]) -> Vec<ChunkPos> {
        answers
            .iter()
            .filter_map(|answer| match answer {
                ServerMessage::WorldChunk(chunk) => Some(chunk.chunk),
                _ => None,
            })
            .collect()
    }

    const HELLO: ClientMessage = ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    };

    #[test]
    fn chunks_in_sight_are_sent_once() {
        let at = ClientMessage::WorldMapPosition { x: 0, y: 0 };
        let first = chunks(&exchange(&[HELLO, at.clone()]));
        // Sight plus preload either way, 64 points per chunk: 11-12 chunks.
        let side = first.iter().filter(|c| c.y == 0).count();
        assert!((11..=13).contains(&side), "{side}");
        assert_eq!(first.len(), side * side);
        // Reporting the same place again sends nothing new.
        let twice = chunks(&exchange(&[HELLO, at.clone(), at]));
        assert_eq!(twice, first);
    }

    #[test]
    fn moving_sends_only_the_new_chunks() {
        let answers = exchange(&[
            HELLO,
            ClientMessage::WorldMapPosition { x: 0, y: 0 },
            ClientMessage::WorldMapPosition { x: 0, y: 64 },
        ]);
        let sent = chunks(&answers);
        let unique: BTreeSet<_> = sent.iter().collect();
        assert_eq!(unique.len(), sent.len());
    }

    #[test]
    fn positions_before_hello_are_refused() {
        let answers = exchange(&[ClientMessage::WorldMapPosition { x: 0, y: 0 }]);
        assert!(matches!(answers[..], [ServerMessage::Refused(_)]));
    }
}
