//! Settings' Gameplay page: Difficulty (`difficulty`), and "Pause When Not in
//! Focus", for single player. The pausing itself is asked for with the menu's
//! (`ask_for_pause`).

use bevy::prelude::*;

use super::super::SystemMenu;
use super::difficulty::{Choices, choices, spawn_difficulty_row};
use super::rows::{spawn_switch_row, tip};
use crate::common::{IN_BROWSER, PlayDifficulty, Preferences, ServerClock, Session};
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

/// What the page last showed: Pause's switch and why not, Difficulty's
/// choices, and whether the Ironman requester was up.
type Shown = (bool, Option<&'static str>, Choices, bool);

/// Builds the page again when what it shows changes (and at the start).
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn fill_gameplay_page(
    mut commands: Commands,
    preferences: Res<Preferences>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    play: Res<PlayDifficulty>,
    menu: Res<SystemMenu>,
    page: Single<Entity, With<GameplayPage>>,
    mut shown: Local<Option<Shown>>,
) {
    let why_not = why_not(&session, &clock);
    let difficulty = choices(*play, preferences.difficulty);
    // Again as the Ironman requester closes, putting back a choice not kept,
    // and on each word from the server, which may have refused one.
    let asking = *menu == SystemMenu::Ironman;
    let now = (preferences.pause_unfocused, why_not, difficulty, asking);
    if shown.as_ref() == Some(&now) && !play.is_changed() {
        return;
    }
    let difficulty = now.2.clone();
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
        spawn_difficulty_row(page, &difficulty, play.current.is_some());
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
