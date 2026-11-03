//! Logging out: back to the startup menu, with no connection and no world.

use bevy::prelude::*;

use super::{Connecting, hang_up};
use crate::common::{CurrentWorld, GameView, ServerStatus, Session};

/// Once the player has logged out (`Session::AtStart` while connected), hangs up
/// and forgets the world, so the next Play starts afresh. A native client's next
/// Play is single player again.
pub(super) fn log_out(
    mut commands: Commands,
    mut connecting: ResMut<Connecting>,
    mut status: ResMut<ServerStatus>,
    mut next_view: ResMut<NextState<GameView>>,
) {
    info!("Logged out");
    hang_up(&mut commands);
    commands.remove_resource::<CurrentWorld>();
    *status = ServerStatus::Connecting;
    connecting.retry = None;
    #[cfg(not(target_arch = "wasm32"))]
    {
        connecting.target = super::socket::Target::Embedded;
    }
    next_view.set(GameView::Connecting);
}

/// Whether the player has logged out while still connected.
pub(super) fn logged_out(session: Res<Session>) -> bool {
    !session.is_playing()
}
