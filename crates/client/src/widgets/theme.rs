//! What all widgets share: their colours and sizes of text. Gaps and padding are
//! multiples of 4 logical pixels; the edges of things are exempt.

use bevy::prelude::*;

use crate::common::NO_QUALITY_NAME;

/// The inside of panels, the colour of parchment.
pub const PARCHMENT: [u8; 3] = [240, 225, 190];
/// Writing and lines on parchment.
pub const INK: Color = Color::srgb(0.25, 0.17, 0.1);

pub(super) const RAISED_BACKGROUND: Color = Color::srgb(0.75, 0.65, 0.48);
pub(super) const LIT_EDGE: Color = Color::srgb(0.95, 0.87, 0.7);
pub(super) const SHADED_EDGE: Color = Color::srgb(0.42, 0.32, 0.2);
pub(super) const SELECTED_TEXT: Color = LIT_EDGE;
pub(super) const DISABLED_BACKGROUND: Color = Color::srgb(0.62, 0.59, 0.53);
pub(super) const DISABLED_EDGE: Color = Color::srgb(0.5, 0.48, 0.44);
pub(super) const DISABLED_TEXT: Color = Color::srgb(0.42, 0.4, 0.37);

/// Dark, so that every quality's colour can be read on it.
pub(super) const TOOLTIP_BACKGROUND: Color = Color::srgb(0.13, 0.1, 0.08);
pub(super) const TOOLTIP_TEXT: Color = NO_QUALITY_NAME;

/// A main button's background.
pub(super) const MAIN_BACKGROUND: Color = Color::srgb(0.85, 0.62, 0.25);
/// The edge of a text box or slider under the pointer.
pub(super) const HOVER_EDGE: Color = Color::srgb(0.7, 0.4, 0.1);
/// What spreads from a press, at its strongest.
pub(super) const RIPPLE: Color = Color::srgba(1.0, 1.0, 1.0, 0.45);

/// What is outside a full-screen layer.
pub(super) const DARKENING: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);
const SHADOW: Color = Color::srgba(0.0, 0.0, 0.0, 0.45);
const SHADOW_DROP: f32 = 4.0;
const SHADOW_BLUR: f32 = 12.0;

pub const GRID_COLOUR: Color = Color::srgb(0.55, 0.46, 0.33);
pub const GRID_LINE: Color = Color::srgb(0.42, 0.34, 0.24);
pub const ITEM_EDGE: Color = Color::srgb(0.15, 0.1, 0.05);
/// Cells a drop could take.
pub const ALLOWED: Color = Color::srgba(0.2, 0.8, 0.25, 0.55);
/// Cells a drop couldn't take.
pub const REFUSED: Color = Color::srgba(0.85, 0.15, 0.15, 0.55);

/// How big writing is: all of it in the interface is one of these.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextSize {
    /// Where room is short: labels on items, the clothes panel.
    Small,
    /// Most writing.
    Medium,
    /// Menus.
    Large,
    /// The titles of panels.
    Title,
}

impl TextSize {
    pub const fn pixels(self) -> f32 {
        match self {
            Self::Small => 13.0,
            Self::Medium => 16.0,
            Self::Large => 18.0,
            Self::Title => 24.0,
        }
    }

    pub fn font(self) -> TextFont {
        TextFont::from_font_size(self.pixels())
    }
}

/// The soft shadow of what floats above everything behind it: panels, tooltips
/// and open lists.
pub(super) fn soft_shadow() -> BoxShadow {
    BoxShadow::new(SHADOW, px(0), px(SHADOW_DROP), px(0), px(SHADOW_BLUR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_sizes_go_up_in_order() {
        let sizes = [
            TextSize::Small,
            TextSize::Medium,
            TextSize::Large,
            TextSize::Title,
        ];
        assert!(
            sizes
                .windows(2)
                .all(|two| two[0].pixels() < two[1].pixels())
        );
    }
}
