//! Settings' Gameplay page: "Pause When Not in Focus", for single player. The
//! pausing itself is asked for with the menu's (`ask_for_pause`).

use bevy::prelude::*;

use super::rows::tip;
use super::{FONT, ROW_HEIGHT};
use crate::common::{IN_BROWSER, Preferences, ServerClock, Session};
use crate::widgets::{ButtonPressed, ButtonState, Pressable, UiId, spawn_check_button};

#[derive(Component)]
pub(in crate::system_menu) struct GameplayPage;

#[derive(Component)]
pub(in crate::system_menu) struct PauseButton;

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
        let node = Node {
            height: px(ROW_HEIGHT),
            align_self: AlignSelf::FlexStart,
            ..default()
        };
        spawn_check_button(page, state, Some(on), "Pause When Not in Focus", FONT, node).insert((
            PauseButton,
            Pressable,
            UiId::new("settings.gameplay", "pause_unfocused"),
            tip(PAUSE, why_not),
        ));
    });
}

pub(in crate::system_menu) fn use_gameplay_page(
    mut pressed: MessageReader<ButtonPressed>,
    buttons: Query<(), With<PauseButton>>,
    mut preferences: ResMut<Preferences>,
) {
    for press in pressed.read() {
        if buttons.contains(press.0) {
            preferences.pause_unfocused = !preferences.pause_unfocused;
        }
    }
}
