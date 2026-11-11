//! "Keep these settings?": after Apply on the Graphics page the window shows the
//! new settings, and goes back to the old ones after `KEEP_SECONDS` unless Keep
//! is pressed, so a mode the screen can't show doesn't leave it black.

use bevy::prelude::*;

use super::super::SystemMenu;
use crate::common::{DisplayChoice, DisplayInUse, Preferences, RememberWindow};
use crate::dom::Dom;
use crate::widgets::ButtonPressed;

const KEEP_SECONDS: f64 = 15.0;

/// Settings applied but not yet kept.
#[derive(Resource, Default)]
pub(in crate::system_menu) struct GraphicsTrial(Option<Trial>);

#[derive(Clone, Copy)]
struct Trial {
    /// What to go back to.
    before: DisplayChoice,
    /// When, in real seconds since starting.
    until: f64,
}

impl GraphicsTrial {
    pub(super) fn start(&mut self, before: DisplayChoice, now: f64) {
        self.0 = Some(Trial {
            before,
            until: now + KEEP_SECONDS,
        });
    }
}

#[derive(Component)]
pub(in crate::system_menu) struct KeepButton;

/// Keeps the settings on Keep; goes back to the old ones when the requester
/// closes any other way, or the time is up.
#[allow(clippy::too_many_arguments)]
pub(in crate::system_menu) fn run_trial(
    mut pressed: MessageReader<ButtonPressed>,
    keeps: Query<(), With<KeepButton>>,
    mut trial: ResMut<GraphicsTrial>,
    mut in_use: ResMut<DisplayInUse>,
    remember: Res<RememberWindow>,
    mut preferences: ResMut<Preferences>,
    mut menu: ResMut<SystemMenu>,
    time: Res<Time<Real>>,
    mut dom: Dom,
    mut shown: Local<String>,
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
    if *menu != SystemMenu::KeepGraphics || now >= until {
        in_use.0 = before;
        trial.0 = None;
        if *menu == SystemMenu::KeepGraphics {
            *menu = SystemMenu::Settings;
        }
        return;
    }
    let left = format!("Going back in {} s.", (until - now).ceil() as u32);
    if *shown != left {
        dom.set_text("settings.keep_graphics/countdown", left.clone());
        *shown = left;
    }
}
