//! Opening and closing the pad, showing it only on a map, remembering both
//! that and where it was dragged to (`Preferences::step_pad`), and reading
//! which arrows are held.

use bevy::prelude::*;

use super::spawn::{PadToggle, StepPad};
use crate::common::{GameView, PadButton, PadSteps, Preferences};
use crate::dom::Dom;
use crate::widgets::{ButtonPressed, ButtonState, HoldButton, INK, PanelMoved};

/// How strongly a disabled button's picture shows.
const DISABLED_ALPHA: f32 = 0.3;

/// The toggle button opens and closes the pad, remembered between runs.
pub(super) fn toggle_pad(
    mut pressed: MessageReader<ButtonPressed>,
    toggle: Single<Entity, With<PadToggle>>,
    mut preferences: ResMut<Preferences>,
) {
    if pressed.read().any(|press| press.0 == *toggle) {
        preferences.step_pad.closed = !preferences.step_pad.closed;
    }
}

/// Remembers where the pad was dragged to, as a share of the window, once
/// let go (so the preferences aren't saved on every frame of a drag).
pub(super) fn remember_place(
    mut moved: MessageReader<PanelMoved>,
    pad: Single<Entity, With<StepPad>>,
    mut preferences: ResMut<Preferences>,
) {
    if let Some(moved) = moved.read().filter(|moved| moved.panel == *pad).last() {
        let at = Some((moved.share.x, moved.share.y));
        if preferences.step_pad.at != at {
            preferences.step_pad.at = at;
        }
    }
}

/// Shows the toggle button on a map, and the pad too while open; the toggle
/// button is flat-edged while it is. Hidden, the pad keeps its place. (Told
/// to the elements only when it changes.)
pub(super) fn show_pad(
    preferences: Res<Preferences>,
    view: Res<State<GameView>>,
    mut toggle: Single<&mut ButtonState, With<PadToggle>>,
    mut dom: Dom,
    mut shown: Local<Option<(bool, bool)>>,
) {
    let open = !preferences.step_pad.closed;
    let on_a_map = matches!(view.get(), GameView::WorldMap | GameView::Dungeon);
    let now = (on_a_map, on_a_map && open);
    if *shown != Some(now) {
        *shown = Some(now);
        dom.set_flag("hud/step_pad", "hidden", !now.0);
        dom.set_flag("step_pad/panel", "hidden", !now.1);
    }
    let look = if open {
        ButtonState::Active
    } else {
        ButtonState::Normal
    };
    toggle.set_if_neq(look);
}

/// The pad's buttons and its toggle, with what is in them.
type PadButtons<'w, 's> = Query<
    'w,
    's,
    (&'static ButtonState, &'static Children),
    Or<(With<PadButton>, With<PadToggle>)>,
>;

/// The white pictures in ink, faint on disabled buttons.
pub(super) fn tint_icons(buttons: PadButtons, mut pictures: Query<&mut ImageNode>) {
    for (state, children) in &buttons {
        let alpha = if state.is_disabled() {
            DISABLED_ALPHA
        } else {
            1.0
        };
        for &child in children {
            if let Ok(mut picture) = pictures.get_mut(child) {
                let colour = INK.with_alpha(alpha);
                if picture.color != colour {
                    picture.color = colour;
                }
            }
        }
    }
}

/// The turn buttons are greyed out on the world map, where nobody faces a way
/// that matters (sight there is all round).
pub(super) fn grey_turns(
    view: Res<State<GameView>>,
    mut buttons: Query<(&PadButton, &mut ButtonState)>,
) {
    let on_world_map = *view.get() == GameView::WorldMap;
    for (button, mut state) in &mut buttons {
        if !matches!(button, PadButton::TurnLeft | PadButton::TurnRight) {
            continue;
        }
        if on_world_map {
            state.set_if_neq(ButtonState::Disabled);
        } else if state.is_disabled() {
            *state = ButtonState::Normal;
        }
    }
}

/// Which way the held arrows ask the player to step (`PadSteps`).
pub(super) fn read_pad(buttons: Query<(&PadButton, &HoldButton)>, mut steps: ResMut<PadSteps>) {
    let mut wanted = PadSteps::default();
    for (button, hold) in &buttons {
        if !hold.held() {
            continue;
        }
        match button {
            PadButton::StepUp => wanted.dy -= 1,
            PadButton::StepDown => wanted.dy += 1,
            PadButton::StepLeft => wanted.dx -= 1,
            PadButton::StepRight => wanted.dx += 1,
            PadButton::TurnLeft => wanted.turn -= 1,
            PadButton::TurnRight => wanted.turn += 1,
            PadButton::Up | PadButton::Down => {}
        }
    }
    steps.set_if_neq(wanted);
}
