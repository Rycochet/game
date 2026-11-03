//! Raised and sunken boxes, panels with their frames and titles, and tooltips.

use bevy::prelude::*;

use super::theme::{INK, PARCHMENT, TextSize};
use super::theme::{
    LIT_EDGE, RAISED_BACKGROUND, SHADED_EDGE, TOOLTIP_BACKGROUND, TOOLTIP_TEXT, soft_shadow,
};
use super::{BUTTON_PADDING, PANEL_EDGE, TOOLTIP_EDGE, TOOLTIP_FONT};

/// Lit along its top and left, in shade along its bottom and right.
pub(super) fn raised_border() -> BorderColor {
    BorderColor {
        top: LIT_EDGE,
        left: LIT_EDGE,
        right: SHADED_EDGE,
        bottom: SHADED_EDGE,
    }
}

/// Shaded along its top and left, lit along its bottom and right.
pub(super) fn sunken_border() -> BorderColor {
    BorderColor {
        top: SHADED_EDGE,
        left: SHADED_EDGE,
        right: LIT_EDGE,
        bottom: LIT_EDGE,
    }
}

/// A box standing up from what is behind it. Put its contents in as children.
pub fn raised_box() -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(PANEL_EDGE)),
            padding: BUTTON_PADDING,
            ..default()
        },
        BackgroundColor(RAISED_BACKGROUND),
        raised_border(),
    )
}

/// A parchment panel, placed by absolute position, with a soft shadow. Its parts
/// go in as children. Give its `Node` a border of `PANEL_EDGE`, or a `panel_frame`
/// if its children are placed by numbers that shouldn't allow for one.
pub fn panel() -> impl Bundle {
    let [red, green, blue] = PARCHMENT;
    (
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(red, green, blue)),
        raised_border(),
        soft_shadow(),
    )
}

/// A panel's edge, as a child covering it: spawn it first, to be behind the rest.
pub fn panel_frame() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            border: UiRect::all(px(PANEL_EDGE)),
            ..default()
        },
        raised_border(),
    )
}

/// The writing for a panel's title. Add a `Node` to place it.
pub fn panel_title(title: impl Into<String>) -> impl Bundle {
    (Text::new(title), TextSize::Title.font(), TextColor(INK))
}

/// A dark box naming something, such as what is under the pointer; `marker` goes on its text,
/// to find it and change what it says. Hide the holder to hide it.
pub fn spawn_tooltip(
    holder: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    text: impl Into<String>,
) {
    holder
        .spawn((
            Node {
                border: UiRect::all(px(TOOLTIP_EDGE)),
                padding: BUTTON_PADDING,
                ..default()
            },
            BackgroundColor(TOOLTIP_BACKGROUND),
            BorderColor::from(SHADED_EDGE),
            soft_shadow(),
        ))
        .with_child((
            marker,
            Text::new(text),
            TOOLTIP_FONT.font(),
            TextColor(TOOLTIP_TEXT),
        ));
}
