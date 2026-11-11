//! Boxes holding a key binding (Settings, Controls): sunken like a text box,
//! showing the binding ("Shift + G") or nothing. A click or tap makes one wait
//! ("Press key") for the next key, with any modifiers held (`capture`); Esc or
//! a click elsewhere cancels, and Backspace empties it. One waits at a time.
//! (A gamepad's box, later, would wait for a button the same way.)

mod capture;
mod show;

pub(super) use capture::{KeyWaiting, take_keys};
pub(super) use show::show_key_boxes;

use bevy::prelude::*;

use super::text_box::line_height;
use super::theme::SUNKEN_BACKGROUND;
use super::{ButtonState, INK, Pressable, TextSize, sunken_border};
use crate::common::Binding;

const SIDE_PADDING: f32 = 8.0;
const BORDER: f32 = 2.0;

/// A box holding a binding. Its owner sets `binding` to show another, and
/// `warned` to show its edge as wrong (a binding just taken from it). Disable it
/// with `ButtonState::Disabled`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct KeyBox {
    pub binding: Option<Binding>,
    pub warned: bool,
}

/// The player chose a binding for a box (`None`: emptied it with Backspace).
/// The box already shows it.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyChosen(pub Entity, pub Option<Binding>);

/// The writing of a `KeyBox`, whose entity it holds.
#[derive(Component)]
pub(super) struct KeyBoxLabel(Entity);

/// A box of this kind, with `node` for its size and where it goes, and
/// `state` `Normal` or `Disabled`.
pub fn spawn_key_box<'a>(
    parent: &'a mut ChildSpawnerCommands,
    key_box: KeyBox,
    state: ButtonState,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let min_height = match node.min_height {
        Val::Auto => px(line_height(size) + BORDER * 2.0),
        given => given,
    };
    let mut holder = parent.spawn((
        key_box,
        state,
        Pressable,
        Node {
            border: UiRect::all(px(BORDER)),
            padding: UiRect::horizontal(px(SIDE_PADDING)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            overflow: Overflow::clip(),
            min_height,
            ..node
        },
        BackgroundColor(SUNKEN_BACKGROUND),
        sunken_border(),
    ));
    let entity = holder.id();
    holder.with_child((
        KeyBoxLabel(entity),
        Text::new(""),
        size.font(),
        TextColor(INK),
    ));
    holder
}

#[cfg(test)]
mod tests;
