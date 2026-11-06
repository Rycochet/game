//! Finding out what items written into text are (`common::KnownItems`): what the
//! player carries is known at once, and the rest is asked of the server in
//! batches (`LookUpItems`), whose answers are kept.

use bevy::prelude::*;
use shared::protocol::{ClientMessage, MOST_LOOKED_UP, ServerMessage};

use crate::common::{KnownItem, KnownItems};
use crate::connection::{FromServer, ToServer};
use crate::inventory::Carried;

pub struct ItemTextPlugin;

impl Plugin for ItemTextPlugin {
    fn build(&self, app: &mut App) {
        // Last, after whatever shows them has said which it wants.
        app.add_systems(PostUpdate, look_up_items)
            .add_systems(Update, read_answers);
    }
}

/// Fills in what is carried, and asks the server after the rest.
fn look_up_items(
    mut known: ResMut<KnownItems>,
    carried: Res<Carried>,
    time: Res<Time<Real>>,
    mut to_server: MessageWriter<ToServer>,
) {
    let wanted = known.take_wanted();
    if wanted.is_empty() {
        return;
    }
    let now = time.elapsed_secs_f64();
    let mut ask = Vec::new();
    for id in wanted {
        match carried.0.find_within(id) {
            Some(item) => known.set(id, KnownItem::Found(Box::new(item.alone()), now)),
            None => {
                known.set(id, KnownItem::Asked(now));
                ask.push(id);
            }
        }
    }
    for batch in ask.chunks(MOST_LOOKED_UP) {
        to_server.write(ToServer(ClientMessage::LookUpItems(batch.to_vec())));
    }
}

/// Keeps the server's answers.
fn read_answers(
    mut from_server: MessageReader<FromServer>,
    mut known: ResMut<KnownItems>,
    time: Res<Time<Real>>,
) {
    let now = time.elapsed_secs_f64();
    for FromServer(message) in from_server.read() {
        if let ServerMessage::ItemsLookedUp { found, unknown } = message {
            for item in found {
                known.set(item.id, KnownItem::Found(Box::new(item.clone()), now));
            }
            for &id in unknown {
                known.set(id, KnownItem::Unknown(now));
            }
        }
    }
}
