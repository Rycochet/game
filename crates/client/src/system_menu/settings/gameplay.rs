//! Settings' Gameplay page: Difficulty (`difficulty`), "Shown As" while playing
//! for staff (`shown_role`), and "Pause When Not in Focus", for single player. The pausing itself is asked for with the menu's
//! (`ask_for_pause`).

use bevy::prelude::*;

use super::super::SystemMenu;
use super::difficulty::{Choices, DifficultyList, choices, show_difficulty_row};
use super::rows::tip_words;
use super::shown_role::{ShownAs, ShownList, offered, show_shown_row};
use crate::common::{IN_BROWSER, PlayDifficulty, Preferences, ServerClock, Session};
use crate::dom::Dom;
use crate::widgets::SwitchFlipped;
use shared::role::ShownRole;

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
/// choices, whether the Ironman requester was up, and Shown As.
type Shown = (bool, Option<&'static str>, Choices, bool, Option<ShownRole>);

/// Shows the page's rows again when what they show changes (and at the
/// start, once the page is built).
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn show_gameplay_page(
    preferences: Res<Preferences>,
    session: Res<Session>,
    clock: Res<ServerClock>,
    play: Res<PlayDifficulty>,
    menu: Res<SystemMenu>,
    shown_as: Res<ShownAs>,
    mut dom: Dom,
    mut lists: Query<&mut DifficultyList>,
    mut roles: Query<&mut ShownList>,
    mut shown: Local<Option<Shown>>,
) {
    let (Ok(mut list), Ok(mut roles)) = (lists.single_mut(), roles.single_mut()) else {
        return;
    };
    let why_not = why_not(&session, &clock);
    let difficulty = choices(*play, preferences.difficulty);
    // Again as the Ironman requester closes, putting back a choice not kept,
    // and on each word from the server, which may have refused one.
    let asking = *menu == SystemMenu::Ironman;
    let role = offered(&shown_as).filter(|_| session.is_playing()).cloned();
    let on = preferences.pause_unfocused;
    let now = (on, why_not, difficulty, asking, role);
    if shown.as_ref() == Some(&now) && !play.is_changed() {
        return;
    }
    show_difficulty_row(&mut dom, &mut list, &now.2, play.current.is_some());
    show_shown_row(&mut dom, &mut roles, now.4.as_ref());
    dom.set_flag("settings.gameplay/pause_unfocused", "checked", on);
    dom.set_flag(
        "settings.gameplay/pause_unfocused",
        "disabled",
        why_not.is_some(),
    );
    let tip = tip_words(PAUSE, why_not);
    dom.set("settings.gameplay/pause_unfocused_row", "title", &tip);
    *shown = Some(now);
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
