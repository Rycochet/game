//! The movement pad: a moveable panel, open by default, with buttons held to
//! step (screen directions, as the step keys) and Up and Down ladders for
//! stairs, exploring and leaving, and the small d-pad button at the bottom
//! right that opens and closes it. See `docs/context/client/step_pad.md`.

use bevy::prelude::*;

use crate::widgets::WidgetSystems;

// Split into files (`cargo xtask lint` keeps files small).
mod icons;
mod picture;
mod show;
mod spawn;

/// The pad's widget form.
const FORM: &str = "step_pad";
/// The button that opens and closes it, across.
const TOGGLE: f32 = 30.0;
/// Each of the pad's buttons, across: room for a finger (as its markup says).
#[cfg(test)]
const BUTTON: f32 = 44.0;
/// Between the pad's buttons, and between the pad and its toggle button.
const GAP: f32 = 4.0;
/// Between the toggle button and the window's corner.
const CORNER_GAP: f32 = 8.0;

pub struct StepPadPlugin;

impl Plugin for StepPadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn::spawn_pad)
            .add_systems(PreUpdate, show::read_pad.after(WidgetSystems::Presses))
            .add_systems(
                Update,
                (
                    spawn::bind_pad,
                    show::toggle_pad,
                    show::remember_place,
                    show::show_pad,
                    show::tint_icons,
                )
                    .chain(),
            );
    }
}
