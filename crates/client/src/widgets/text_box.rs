//! Boxes to type into. One has the keyboard at a time (`TextFocus`): a click or
//! tap on a box gives it the keyboard, and something else covering the screen
//! can give it to its own box. The caret can be anywhere in the writing, and
//! some of it selected, by keys, clicks or a finger (`edit` has the rules).

use bevy::prelude::*;
use bevy::text::LineBreak;

use super::item_text::InlineHeight;
use super::theme::SUNKEN_BACKGROUND;
use super::{INK, Pressable, TextSize, sunken_border};
use crate::common::TakesItemText;

mod caret;
mod clicks;
mod dropped;
mod edit;
mod keys;
mod layout;
mod selection;
mod show;
mod units;

pub(super) use caret::show_carets;
pub(super) use clicks::{drag_to_select, select_by_pointer};
pub(super) use dropped::take_dropped_items;
pub(super) use keys::{Held, take_text};
pub(super) use selection::show_selections;
pub(super) use show::{place_soft_keyboard, show_text_boxes};

const SIDE_PADDING: f32 = 8.0;
const BORDER: f32 = 2.0;

/// A box to type into. Change `text` to change what it says.
#[derive(Component, Clone, Debug, Default)]
pub struct TextBox {
    pub text: String,
    /// Written before the text, and not part of it.
    pub prefix: &'static str,
    pub digits_only: bool,
    /// While it has the keyboard, nothing else gets any keys (Return and Esc give
    /// the keyboard up). Off for a box in something that reads the keys it wants
    /// and takes the rest itself.
    pub takes_keys: bool,
    /// What it says is wrong: its edge shows it, whoever has the keyboard.
    pub error: bool,
    /// Written after the text (and the caret) in grey, and not part of it.
    pub suffix: String,
    /// Items can be dropped on it, and items written in (`${item:<uuid>}`) show
    /// as pictures and count as one place each.
    pub accepts_items: bool,
}

/// Where a box's caret is and what is selected, kept apart from `TextBox` so
/// that its owners can still make one by its fields.
#[derive(Component, Clone, Debug, Default)]
pub(super) struct Cursor {
    pub caret: usize,
    pub anchor: usize,
    /// What the box said when this was last set: if its owner changes the
    /// text, the caret goes to the end.
    pub text: String,
    /// How far the writing is slid left, in interface pixels, to keep the
    /// caret in sight.
    pub scroll: f32,
    /// When the caret last moved or the writing changed, by the real clock: it
    /// stays solid a while after.
    pub moved_at: f32,
}

/// The writing of a `TextBox`, whose entity it holds. Its own text is the
/// prefix; each character typed is a span after it, then the suffix.
#[derive(Component)]
pub(super) struct TextBoxLabel(pub Entity);

/// One character of a box's writing, by its place.
#[derive(Component)]
pub(super) struct TextBoxChar(pub usize);

/// The grey writing after it (`TextBox::suffix`), the label's last span.
#[derive(Component)]
pub(super) struct TextBoxSuffix(pub Entity);

/// The line showing where typing goes, in the box it holds.
#[derive(Component)]
pub(super) struct TextBoxCaret(pub Entity);

/// Which box has the keyboard. Something that owns a box can set it.
#[derive(Resource, Default)]
pub struct TextFocus(pub Option<Entity>);

/// The player changed what a box says.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextEdited(pub Entity);

/// Return or Esc in the browser's hidden input, which the game's keys don't see.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoftKeyPressed(pub super::soft_keyboard::Key);

/// A press happened this frame, before anything took it.
#[derive(Resource, Default)]
pub(super) struct PressedThisFrame(pub(super) bool);

/// A place where a tap should raise the on-screen keyboard, other than a box.
#[derive(Component)]
pub(super) struct RaisesKeyboard;

/// A box of this kind, with `node` for its size and where it goes.
pub fn spawn_text_box<'a>(
    parent: &'a mut ChildSpawnerCommands,
    text_box: TextBox,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    // Empty writing has no height: the box keeps a line's, unless given one.
    let min_height = match node.min_height {
        Val::Auto => px(line_height(size) + BORDER * 2.0),
        given => given,
    };
    let end = edit::Edited::at_end(&text_box.text, text_box.accepts_items).caret;
    let cursor = Cursor {
        caret: end,
        anchor: end,
        text: text_box.text.clone(),
        ..default()
    };
    let takes_items = text_box.accepts_items;
    let mut holder = parent.spawn((
        text_box,
        cursor,
        Pressable,
        Node {
            border: UiRect::all(px(BORDER)),
            padding: UiRect::horizontal(px(SIDE_PADDING)),
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            // Writing slid along is cut at the padding, not the edge.
            overflow_clip_margin: OverflowClipMargin::content_box(),
            min_height,
            ..node
        },
        BackgroundColor(SUNKEN_BACKGROUND),
        sunken_border(),
    ));
    let entity = holder.id();
    holder.with_children(|holder| {
        // First, so behind the writing.
        holder.spawn((
            selection::TextBoxSelection(entity),
            Node {
                position_type: PositionType::Absolute,
                height: px(line_height(size)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
        ));
        holder.spawn((
            TextBoxLabel(entity),
            InlineHeight(line_height(size)),
            Text::new(""),
            TextLayout {
                linebreak: LineBreak::NoWrap,
                ..default()
            },
            size.font(),
            TextColor(INK),
            // Placed over the box rather than sizing it, so long writing
            // slides left (`Node::left`) instead of widening the box.
            Node {
                position_type: PositionType::Absolute,
                left: px(SIDE_PADDING),
                ..default()
            },
        ));
        holder.spawn((
            TextBoxCaret(entity),
            Node {
                position_type: PositionType::Absolute,
                width: px(caret::WIDTH),
                height: px(line_height(size)),
                ..default()
            },
            BackgroundColor(INK),
            Visibility::Hidden,
        ));
    });
    if takes_items {
        // On its parts too: whichever the pointer is on counts as the box.
        holder.insert(TakesItemText(entity));
        let commands = holder.commands_mut();
        commands.queue(move |world: &mut World| {
            let parts = world.get::<Children>(entity).map(|c| c.to_vec());
            for part in parts.unwrap_or_default() {
                world.entity_mut(part).insert(TakesItemText(entity));
            }
        });
    }
    holder
}

/// How tall a line of writing this size is: Bevy's own line height, 1.2 times
/// the font's size.
pub(super) fn line_height(size: TextSize) -> f32 {
    size.pixels() * 1.2
}

/// The words naming a box, to go above it.
pub fn field_label(words: impl Into<String>) -> impl Bundle {
    (Text::new(words), TextSize::Small.font(), TextColor(INK))
}

/// What of `typed` a box keeps.
pub(super) fn allowed(typed: &str, digits_only: bool) -> String {
    typed
        .chars()
        .filter(|c| !c.is_control() && (!digits_only || c.is_ascii_digit()))
        .collect()
}

pub(super) fn note_press(
    mouse: Res<ButtonInput<MouseButton>>,
    mut noted: ResMut<PressedThisFrame>,
) {
    noted.0 = mouse.just_pressed(MouseButton::Left);
}

#[cfg(test)]
mod tests;
