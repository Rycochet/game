//! Settings' Gameplay page: "Pause When Not in Focus", for single player. The
//! pausing itself is asked for with the menu's (`ask_for_pause`).

use bevy::prelude::*;

use super::rows::{spawn_switch_row, tip};
use crate::common::{IN_BROWSER, Preferences, ServerClock, Session};
use crate::widgets::{ButtonState, SwitchFlipped, UiId};

#[derive(Component)]
pub(in crate::system_menu) struct GameplayPage;

#[derive(Component)]
pub(in crate::system_menu) struct PauseSwitch;

const PAUSE: &str = "Pauses the game while another window is in front. Single player only.";

/// Why it can't be chosen now, if it can't: in a browser there is no single
/// player, and while playing on another computer's server nothing pauses.
fn why_not(session: &Session, clock: &ServerClock) -> Option<&'static str> {
    if IN_BROWSER {
        Some("Not in the browser, which always plays on a server.")
    } else if session.is_playing() && !clock.local_server {
        Some("Not while playing on another computer's server.")
    } else {
        None
    }
}

/// Builds the page again when what it shows changes (and at the start).
pub(in crate::system_menu) fn fill_gameplay_page(
    mut commands: Commands,
    preferences: Res<Preferences>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    page: Single<Entity, With<GameplayPage>>,
    mut shown: Local<Option<(bool, Option<&'static str>)>>,
) {
    let why_not = why_not(&session, &clock);
    let now = (preferences.pause_unfocused, why_not);
    if *shown == Some(now) {
        return;
    }
    *shown = Some(now);
    let on = preferences.pause_unfocused;
    let state = match (why_not.is_some(), on) {
        (false, _) => ButtonState::Normal,
        (true, false) => ButtonState::Disabled,
        (true, true) => ButtonState::DisabledSelected,
    };
    let page = *page;
    commands.entity(page).despawn_related::<Children>();
    commands.entity(page).with_children(|page| {
        let label = "Pause When Not in Focus";
        let made = spawn_switch_row(page, label, tip(PAUSE, why_not), state, on);
        page.commands().entity(made).insert((
            PauseSwitch,
            UiId::new("settings.gameplay", "pause_unfocused"),
        ));
    });
}

pub(in crate::system_menu) fn use_gameplay_page(
    mut flipped: MessageReader<SwitchFlipped>,
    switches: Query<(), With<PauseSwitch>>,
    mut preferences: ResMut<Preferences>,
) {
    for &SwitchFlipped(switch, on) in flipped.read() {
        if switches.contains(switch) {
            preferences.pause_unfocused = on;
        }
    }
}
