//! Logging out, or going back after a refusal: back to the startup menu, with no
//! connection and no world; and starting afresh on Play.

use bevy::prelude::*;

use super::{Connecting, hang_up};
use crate::common::{ChosenServer, CurrentWorld, GameView, PlayFor, ServerStatus, Session};

/// Once back at the startup menu (`Session::AtStart`), hangs up if connected and
/// forgets the world. A refusal is kept, for the menu to say why.
/// `HOW_SERVER` no longer counts after that: a native client's next Continue is
/// single player. A server chosen in Connect is forgotten (`ChosenServer`), as
/// Continue and Characters are for this computer's own (owner, 2026-11-10).
#[allow(clippy::too_many_arguments)]
pub(super) fn log_out(
    mut commands: Commands,
    mut connecting: ResMut<Connecting>,
    mut status: ResMut<ServerStatus>,
    view: Res<State<GameView>>,
    mut next_view: ResMut<NextState<GameView>>,
    connection: Option<Res<super::ServerConnection>>,
    mut play_for: ResMut<PlayFor>,
    mut chosen: ResMut<ChosenServer>,
) {
    chosen.set_if_neq(ChosenServer(None));
    // Connect's next server is for Play, unless Characters says otherwise.
    play_for.set_if_neq(PlayFor::Latest);
    if connection.is_some() {
        info!("Logged out");
        hang_up(&mut commands);
        connecting.environment = false;
    }
    commands.remove_resource::<CurrentWorld>();
    if !matches!(*status, ServerStatus::Refused(_)) {
        *status = ServerStatus::Connecting;
    }
    connecting.retry = None;
    if *view.get() != GameView::Connecting {
        next_view.set(GameView::Connecting);
    }
}

/// On Play: a refusal from before is forgotten, so it tries again.
pub(super) fn start_playing(mut status: ResMut<ServerStatus>) {
    if matches!(*status, ServerStatus::Refused(_)) {
        *status = ServerStatus::Connecting;
    }
}

/// Whether Play has just been pressed.
pub(super) fn now_playing(session: Res<Session>) -> bool {
    session.is_changed() && session.is_playing()
}

/// Whether the player has just gone back to the startup menu.
pub(super) fn back_at_start(session: Res<Session>) -> bool {
    session.is_changed() && !session.is_added() && !session.is_playing()
}
