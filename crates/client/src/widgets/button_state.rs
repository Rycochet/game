//! Widget system order, button presses, and how a button looks by state.

use bevy::prelude::*;

use super::raised_border;
use super::theme::INK;
use super::theme::{
    DISABLED_BACKGROUND, DISABLED_EDGE, DISABLED_TEXT, RAISED_BACKGROUND, SELECTED_TEXT,
    SHADED_EDGE,
};

/// Where widgets take what they use of the mouse and fingers, in `PreUpdate`,
/// so nothing underneath them sees it. Anything else taking clicks there runs
/// after these.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WidgetSystems {
    /// Presses on widgets become `ButtonPressed`.
    Presses,
    /// As `Modal`, for what stays in front of everything that is in `Modal`.
    Front,
    /// Something covering the screen reads them, and takes all other input.
    Modal,
}

/// Brings this moveable panel to the front.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaisePanel(pub Entity);

/// A button marked `Pressable` was clicked or tapped, unless it was disabled.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonPressed(pub Entity);

/// Put this on a button (from `spawn_button`), or any node, for clicks on it to
/// be sent as `ButtonPressed` and go no further.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(crate::dom::Solid)]
pub struct Pressable;

/// How a button looks; change it to change the button.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[require(crate::dom::Solid)]
pub enum ButtonState {
    /// Standing up from what is behind it.
    #[default]
    Normal,
    /// The one chosen: darker and flat.
    Selected,
    /// Not chosen but special in some way, such as the floor the player is on
    /// while the map shows another: as normal, but with a flat border.
    Active,
    /// Can't be used now: grey, and clicks do nothing.
    Disabled,
    /// As `Disabled`, but with the keyboard on it: a dark border.
    DisabledSelected,
}

impl ButtonState {
    /// Whether clicks on it do nothing.
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled | Self::DisabledSelected)
    }

    /// Its background, border and text colours.
    pub(super) fn colours(self) -> (Color, BorderColor, Color) {
        match self {
            Self::Normal => (RAISED_BACKGROUND, raised_border(), INK),
            Self::Selected => (SHADED_EDGE, BorderColor::from(INK), SELECTED_TEXT),
            Self::Active => (RAISED_BACKGROUND, BorderColor::from(INK), INK),
            Self::Disabled => (
                DISABLED_BACKGROUND,
                BorderColor::from(DISABLED_EDGE),
                DISABLED_TEXT,
            ),
            Self::DisabledSelected => (DISABLED_BACKGROUND, BorderColor::from(INK), DISABLED_TEXT),
        }
    }
}
