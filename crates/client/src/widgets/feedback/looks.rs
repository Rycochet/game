//! How a button looks by state, style and touch, and its ripples.

use bevy::prelude::*;

use super::super::theme::{MAIN_BACKGROUND, PARCHMENT, RIPPLE};
use super::super::{ButtonState, INK, raised_border, sunken_border};
use super::{DARKER_WHEN_PRESSED, LIGHTER_WHEN_HOVERED, RIPPLE_SECONDS};

/// How much a button stands out; leave it off for an ordinary one.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Plain,
    /// The main thing to do here: one to an area.
    Main,
    /// Something less often wanted: no box until the pointer is on it.
    Quiet,
    /// A tab: when chosen, of a piece with the panel below it.
    Folder,
}

/// The name of a button that only shows a picture, for hovering over it.
#[derive(Component, Clone, Debug)]
pub struct ButtonTooltip(pub String);

/// What the pointer is doing to a button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Touch {
    Away,
    Hovered,
    Pressed,
}

/// What the pointer is on.
#[derive(Resource, Default)]
pub(in crate::widgets) struct Pointed {
    /// The front-most widget under it, unless disabled.
    pub(in crate::widgets) hovered: Option<Entity>,
    /// When `hovered` last changed, in seconds.
    pub(super) since: f32,
    /// The widget the button being held went down on.
    pub(in crate::widgets) pressed: Option<Entity>,
}

/// A spreading circle in a button, in a holder that cuts it to the button.
#[derive(Component)]
pub(in crate::widgets) struct Ripple {
    pub(super) age: f32,
    pub(super) centre: Vec2,
    pub(super) reach: f32,
}

/// A button's background, border and text colours.
pub(super) fn look(
    state: ButtonState,
    style: ButtonStyle,
    touch: Touch,
) -> (Color, BorderColor, Color) {
    let (mut background, mut border, mut text) = state.colours();
    if state == ButtonState::Selected && style == ButtonStyle::Folder {
        let [red, green, blue] = PARCHMENT;
        background = Color::srgb_u8(red, green, blue);
        border = BorderColor {
            bottom: background,
            ..raised_border()
        };
        text = INK;
    }
    if state == ButtonState::Normal {
        match style {
            ButtonStyle::Plain | ButtonStyle::Folder => {}
            ButtonStyle::Main => background = MAIN_BACKGROUND,
            ButtonStyle::Quiet if touch == Touch::Away => {
                background = Color::NONE;
                border = BorderColor::from(Color::NONE);
            }
            ButtonStyle::Quiet => {}
        }
    }
    if state.is_disabled() {
        return (background, border, text);
    }
    match touch {
        Touch::Away => {}
        Touch::Hovered => background = background.lighter(LIGHTER_WHEN_HOVERED),
        Touch::Pressed => {
            background = background.darker(DARKER_WHEN_PRESSED);
            // Pushed in: the light now falls on its other edges.
            if state == ButtonState::Normal {
                border = sunken_border();
            }
        }
    }
    (background, border, text)
}

/// How far across a ripple is and how strongly it shows, `age` seconds on, out of
/// `reach` across at its widest.
pub(super) fn ripple_shape(age: f32, reach: f32) -> (f32, f32) {
    let done = (age / RIPPLE_SECONDS).clamp(0.0, 1.0);
    (reach * done, (1.0 - done) * RIPPLE.alpha())
}
