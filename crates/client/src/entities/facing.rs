//! Which way others face (`common::Facing`): the way each stretch they move
//! goes, or as the server says when they turn in place (`Turned`).

use bevy::prelude::*;
use shared::facing::Heading;
use shared::protocol::ServerMessage;

use super::Others;
use crate::common::Facing;
use crate::connection::FromServer;

pub(super) fn read_turns(
    mut from_server: MessageReader<FromServer>,
    others: Res<Others>,
    mut facings: Query<&mut Facing>,
) {
    let mut face = |id, heading| {
        if let Some(&entity) = others.0.get(&id)
            && let Ok(mut facing) = facings.get_mut(entity)
        {
            facing.set_if_neq(Facing(heading));
        }
    };
    for FromServer(message) in from_server.read() {
        match message {
            ServerMessage::Moves(stretches) => {
                for stretch in stretches {
                    let (dx, dy) = (stretch.to.0 - stretch.from.0, stretch.to.1 - stretch.from.1);
                    if let Some(heading) = Heading::of(dx, dy) {
                        face(stretch.id, heading);
                    }
                }
            }
            ServerMessage::Turned(turns) => {
                for &(id, heading) in turns {
                    face(id, heading);
                }
            }
            _ => {}
        }
    }
}
