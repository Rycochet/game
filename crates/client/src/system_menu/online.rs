//! "Online": a check button with Server Options' gear attached, opening this
//! game's own server to other players (native). The ports open once playing on
//! it (`connection` asks); while playing there, turning it either way asks
//! first. If it can't open, it goes back off and says why.
//!
//! On a networked server, for staff who may change it (`common::Lockdown`), it
//! shows whether the server is open to everyone, and turning it off locks it
//! down instead (owner, 2026-11-10): asked first, then `SetLockdown`. There is
//! no gear there, as Server Options are this game's own server's.

use bevy::prelude::*;
use shared::protocol::OnlineState;

use super::{Action, SystemMenu};
use crate::common::{Hosting, Lockdown, Preferences, ServerClock, Session};
use crate::connection::ToServer;
use crate::dom::Dom;
use crate::widgets::{ButtonPressed, ButtonTooltip, Checked};

/// The toggle itself.
#[derive(Component)]
pub(super) struct OnlineToggle;

/// What the toggle's tooltip says for this game's own server, and for a
/// networked one.
pub(super) const OWN_TIP: &str = "Let other players join this game";
const LOCKDOWN_TIP: &str = "Let everyone play here: off locks the server down, for its staff only";

/// The toggle's presses: at once unless playing on this game's own server, where
/// it asks first; on a networked server, asking first whether to lock it down
/// or open it again.
#[allow(clippy::too_many_arguments)]
pub(super) fn use_online(
    mut pressed: MessageReader<ButtonPressed>,
    actions: Query<&Action>,
    mut menu: ResMut<SystemMenu>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    lockdown: Res<Lockdown>,
    mut preferences: ResMut<Preferences>,
    mut to_server: MessageWriter<ToServer>,
) {
    let action = pressed
        .read()
        .filter_map(|press| actions.get(press.0).ok())
        .last()
        .copied();
    let remote = session.is_playing() && !clock.local_server;
    match (action, *menu) {
        (Some(Action::ToggleOnline), SystemMenu::Menu) if remote => {
            if let Some(locked) = lockdown.0 {
                *menu = if locked {
                    SystemMenu::Reopening
                } else {
                    SystemMenu::LockingDown
                };
            }
        }
        (Some(Action::ReallyToggleOnline), SystemMenu::LockingDown | SystemMenu::Reopening) => {
            let lock = *menu == SystemMenu::LockingDown;
            to_server.write(ToServer(shared::protocol::ClientMessage::SetLockdown(lock)));
            *menu = SystemMenu::Menu;
        }
        (Some(Action::ToggleOnline), SystemMenu::Menu) => {
            if session.is_playing() && clock.local_server {
                *menu = if preferences.online {
                    SystemMenu::GoingOffline
                } else {
                    SystemMenu::GoingOnline
                };
            } else {
                preferences.online = !preferences.online;
            }
        }
        (Some(Action::ReallyToggleOnline), SystemMenu::GoingOnline | SystemMenu::GoingOffline) => {
            preferences.online = *menu == SystemMenu::GoingOnline;
            *menu = SystemMenu::Menu;
        }
        _ => {}
    }
}

/// The toggle's box is ticked while on (owner, 2026-11-07): this game's own
/// server open to others, or a networked one open to everyone (not locked
/// down). The gear only for this game's own.
#[allow(clippy::type_complexity)]
pub(super) fn show_online(
    preferences: Res<Preferences>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    lockdown: Res<Lockdown>,
    mut toggles: Query<(&mut Checked, &mut ButtonTooltip), With<OnlineToggle>>,
    mut dom: Dom,
    mut gear_shown: Local<Option<bool>>,
) {
    let remote = session.is_playing() && !clock.local_server;
    let (on, tip) = match remote {
        true => (lockdown.0 == Some(false), LOCKDOWN_TIP),
        false => (preferences.online, OWN_TIP),
    };
    for (mut checked, mut tooltip) in &mut toggles {
        checked.set_if_neq(Checked(on));
        if tooltip.0 != tip {
            tooltip.0 = tip.into();
        }
    }
    // In a browser there are no Server Options.
    let gear = !remote && !cfg!(target_arch = "wasm32");
    if *gear_shown != Some(gear) {
        *gear_shown = Some(gear);
        dom.set_flag("menu/server_options", "hidden", !gear);
    }
}

/// If the server couldn't open (its port is taken, say), Online goes back off
/// and the menu says why.
pub(super) fn show_online_failure(
    hosting: Res<Hosting>,
    mut preferences: ResMut<Preferences>,
    mut menu: ResMut<SystemMenu>,
    mut dom: Dom,
) {
    if !hosting.is_changed() {
        return;
    }
    if let OnlineState::Failed(reason) = &hosting.0 {
        let words = format!("{reason}. Online is off again: change Server Options and try again.");
        dom.set_text("online_failed/words", words);
        preferences.online = false;
        *menu = SystemMenu::OnlineFailed;
    }
}
