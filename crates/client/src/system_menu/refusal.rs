//! When a server won't be played on (it refuses, or its certificate isn't the
//! one kept for it): back to the startup menu, with a requester saying why.

use bevy::prelude::*;

use super::SystemMenu;
use crate::common::{ChosenServer, SavedServer, ServerStatus, Session};
use crate::dom::Dom;

/// The server chosen in Connect that just refused, if it was one: OK goes back
/// to Connect with it in the box (and its key in the padlock), as going back to
/// the startup menu forgets `ChosenServer`.
#[derive(Resource, Default)]
pub(super) struct TriedServer(pub(super) Option<SavedServer>);

/// A refusal while playing (or trying to) goes back to the startup menu to say
/// why. `connection` keeps the refusal until playing again.
pub(super) fn show_refusal(
    status: Res<ServerStatus>,
    chosen: Res<ChosenServer>,
    mut tried: ResMut<TriedServer>,
    mut session: ResMut<Session>,
    mut menu: ResMut<SystemMenu>,
    mut dom: Dom,
) {
    if !status.is_changed() || !session.is_playing() {
        return;
    }
    if let ServerStatus::Refused(reason) = &*status {
        dom.set_text("refused/words", reason.clone());
        tried.0.clone_from(&chosen.0);
        *session = Session::AtStart;
        *menu = SystemMenu::Refused;
    }
}
