//! A button showing a clipboard, for copying something beside it. Native only,
//! as `copy_to_clipboard` is.

use bevy::prelude::*;

use super::{BUTTON_BORDER, ButtonState, ButtonTooltip, TextSize};

/// A clipboard (U+1F4CB), from the icon font.
const COPY_ICON: &str = "\u{1F4CB}";

/// A clipboard button named `what` (such as "Copy the seed"), sized by `node`.
/// It does nothing by itself: make it `Pressable`, and copy when it is pressed.
pub fn spawn_copy_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    what: &str,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let state = ButtonState::Normal;
    let (background, border, text) = state.colours();
    let mut button = parent.spawn((
        state,
        ButtonTooltip(what.into()),
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((
        Text::new(COPY_ICON),
        // The emoji is only in the icon font.
        TextFont {
            font: crate::fonts::with_icons(),
            ..size.font()
        },
        TextLayout::justify(Justify::Center),
        TextColor(text),
    ));
    button
}
