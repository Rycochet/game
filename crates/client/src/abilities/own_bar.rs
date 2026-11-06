//! The player's action bar as the client has it: what the server last said, plus
//! any change asked for since (checked by the same `ActionBar::apply` first, so a
//! change the server would refuse is never sent). The server answers every change
//! with the bar, which puts right any it refused.

use bevy::prelude::*;
use shared::ability::{ActionBar, BarChange};
use shared::creature::CreatureKind;
use shared::protocol::{ClientMessage, ServerMessage};

use crate::connection::{FromServer, ToServer};

/// The player's action bar.
#[derive(Resource, Default)]
pub(super) struct OwnBar(pub ActionBar);

/// Takes the bar the server sends.
pub(super) fn take_bar(mut from_server: MessageReader<FromServer>, mut bar: ResMut<OwnBar>) {
    for FromServer(message) in from_server.read() {
        if let ServerMessage::ActionBar(new) = message {
            bar.0 = new.clone();
        }
    }
}

impl OwnBar {
    /// Makes the change here and asks the server for it, if the rules allow it for
    /// a body of kind `kind`.
    pub(super) fn change(
        &mut self,
        change: BarChange,
        kind: CreatureKind,
        to_server: &mut MessageWriter<ToServer>,
    ) {
        match self.0.apply(change, kind) {
            Ok(()) => {
                to_server.write(ToServer(ClientMessage::ActionBar(change)));
            }
            Err(why) => debug!("Not changing the action bar ({change:?}): {why:?}"),
        }
    }
}
