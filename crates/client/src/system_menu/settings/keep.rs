//! "Keep these settings?": after Apply on the Video page the window shows the
//! new settings, and goes back to the old ones after `KEEP_SECONDS` unless Keep
//! is pressed, so a mode the screen can't show doesn't leave it black.

use bevy::prelude::*;

use super::super::{ABOVE_MENU_LAYER, Action, BUTTON_ROW, FONT, Layer, MENU_WIDTH, SystemMenu};
use crate::common::{DisplayChoice, DisplayInUse, Preferences, RememberWindow};
use crate::widgets::{
    ButtonPressed, ButtonState, ButtonStyle, INK, Pressable, spawn_button, spawn_dialog,
};

const KEEP_SECONDS: f64 = 15.0;
/// Above Settings.
const POPUP_LAYER: i32 = ABOVE_MENU_LAYER + 1;

/// Settings applied but not yet kept.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct VideoTrial(Option<Trial>);

#[derive(Clone, Copy)]
struct Trial {
    /// What to go back to.
    before: DisplayChoice,
    /// When, in real seconds since starting.
    until: f64,
}

impl VideoTrial {
    pub(super) fn start(&mut self, before: DisplayChoice, now: f64) {
        self.0 = Some(Trial {
            before,
            until: now + KEEP_SECONDS,
        });
    }
}

#[derive(Component)]
pub(in crate::system_menu) struct Countdown;

#[derive(Component)]
pub(in crate::system_menu) struct KeepButton;

pub(in crate::system_menu) fn spawn_keep_requester(commands: &mut Commands) {
    let dialog = spawn_dialog(commands, POPUP_LAYER, "Video");
    commands
        .entity(dialog.layer)
        .insert((Layer(&[SystemMenu::KeepVideo]), Action::Back));
    commands.entity(dialog.content).with_children(|content| {
        content.spawn((
            Text::new("Keep these settings?"),
            FONT.font(),
            TextColor(INK),
        ));
        content.spawn((Countdown, Text::new(""), FONT.font(), TextColor(INK)));
    });
    commands.entity(dialog.buttons).with_children(|row| {
        let node = Node {
            width: px(MENU_WIDTH / 2.0),
            height: px(BUTTON_ROW),
            ..default()
        };
        spawn_button(row, ButtonState::Normal, "Keep", FONT, node.clone()).insert((
            KeepButton,
            Action::KeepVideo,
            ButtonStyle::Main,
            Pressable,
        ));
        spawn_button(row, ButtonState::Normal, "Go back", FONT, node).insert((
            Action::Back,
            ButtonStyle::Plain,
            Pressable,
        ));
    });
}

/// Keeps the settings on Keep; goes back to the old ones when the requester
/// closes any other way, or the time is up.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn run_trial(
    mut pressed: MessageReader<ButtonPressed>,
    keeps: Query<(), With<KeepButton>>,
    mut trial: ResMut<VideoTrial>,
    mut in_use: ResMut<DisplayInUse>,
    remember: Res<RememberWindow>,
    mut preferences: ResMut<Preferences>,
    mut menu: ResMut<SystemMenu>,
    time: Res<Time<Real>>,
    mut countdown: Single<&mut Text, With<Countdown>>,
) {
    let kept = pressed.read().any(|press| keeps.contains(press.0));
    let Some(Trial { before, until }) = trial.0 else {
        return;
    };
    if kept {
        if remember.0 {
            preferences.window.keep(in_use.0);
        }
        trial.0 = None;
        return;
    }
    let now = time.elapsed_secs_f64();
    if *menu != SystemMenu::KeepVideo || now >= until {
        in_use.0 = before;
        trial.0 = None;
        if *menu == SystemMenu::KeepVideo {
            *menu = SystemMenu::Settings;
        }
        return;
    }
    let left = format!("Going back in {} s.", (until - now).ceil() as u32);
    if countdown.0 != left {
        countdown.0 = left;
    }
}
