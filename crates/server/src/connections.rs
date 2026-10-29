//! Connected clients: reading their messages and answering them.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerLink, ServerMessage};

use crate::ServerWorld;

/// The clients connected to this server.
#[derive(Resource, Default)]
pub struct Connections(Vec<ServerLink>);

impl Connections {
    /// A newly connected client.
    pub fn add(&mut self, link: ServerLink) {
        self.0.push(link);
    }
}

/// Answers every message that has arrived, and forgets clients that have
/// gone (or were refused).
pub(crate) fn handle_messages(mut connections: ResMut<Connections>, world: Res<ServerWorld>) {
    connections.0.retain(|link| {
        loop {
            match link.receive() {
                Ok(Some(message)) => {
                    if !answer(link, message, &world) {
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
fn answer(link: &ServerLink, message: ClientMessage, world: &ServerWorld) -> bool {
    match message {
        ClientMessage::Hello { protocol } if protocol != PROTOCOL_VERSION => {
            link.send(&ServerMessage::Refused(format!(
                "this server speaks protocol {PROTOCOL_VERSION}, not {protocol}"
            )));
            false
        }
        ClientMessage::Hello { .. } => link.send(&ServerMessage::Welcome(world.info())),
    }
}
