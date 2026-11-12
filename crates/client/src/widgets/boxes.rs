//! Raised and sunken boxes, panels with their frames and titles, and tooltips.

use bevy::prelude::*;

use super::theme::{INK, PARCHMENT, TextSize};
use super::theme::{
    LIT_EDGE, RAISED_BACKGROUND, SHADED_EDGE, TOOLTIP_BACKGROUND, TOOLTIP_TEXT, soft_shadow,
};
use super::{BUTTON_PADDING, PANEL_EDGE, TITLE_LIFT, TOOLTIP_EDGE, TOOLTIP_FONT};

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
/// go in as children. Give its `Node` a border of `PANEL_EDGE`.
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

/// The writing for a panel's title. Add a `Node` to place it. Drawn a little
/// higher than its place, as Lexend leaves more room above capitals than below,
/// so a centred title looks low.
pub fn panel_title(title: impl Into<String>) -> impl Bundle {
    (
        Text::new(title),
        TextSize::Title.font(),
        TextColor(INK),
        UiTransform::from_translation(Val2::px(0.0, -TITLE_LIFT)),
    )
}

/// The dark box of a tooltip, with `node`'s layout, for more than a name in it.
pub fn tooltip_box(node: Node) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(TOOLTIP_EDGE)),
            padding: BUTTON_PADDING,
            ..node
        },
        BackgroundColor(TOOLTIP_BACKGROUND),
        BorderColor::from(SHADED_EDGE),
        soft_shadow(),
    )
}

/// A tooltip box's background and writing colours.
pub fn tooltip_colours() -> (Color, Color) {
    (TOOLTIP_BACKGROUND, TOOLTIP_TEXT)
}

/// How wide a tooltip box's edge is.
pub const TOOLTIP_BOX_EDGE: f32 = TOOLTIP_EDGE;

/// A tooltip box's edge colour and shadow, for a node laid out some other way.
pub fn tooltip_trim() -> impl Bundle {
    (BorderColor::from(SHADED_EDGE), soft_shadow())
}

/// A dark box naming something, such as what is under the pointer; `marker` goes on its text,
/// to find it and change what it says. Hide the holder to hide it.
pub fn spawn_tooltip(
    holder: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    text: impl Into<String>,
) {
    holder.spawn(tooltip_box(Node::default())).with_child((
        marker,
        Text::new(text),
        TOOLTIP_FONT.font(),
        TextColor(TOOLTIP_TEXT),
    ));
}
