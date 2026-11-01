//! Boxes to type into. One has the keyboard at a time (`TextFocus`): a click or
//! tap on a box gives it the keyboard, and something else covering the screen
//! can give it to its own box.

use bevy::input::ButtonState as KeyState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::soft_keyboard;
use super::{ButtonPressed, INK, Pressable};

const CARET: &str = "_";
const SIDE_PADDING: f32 = 8.0;

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
}

/// The writing of a `TextBox`, whose entity it holds.
#[derive(Component)]
pub(super) struct TextBoxLabel(Entity);

/// Which box has the keyboard. Something that owns a box can set it.
#[derive(Resource, Default)]
pub struct TextFocus(pub Option<Entity>);

/// The player changed what a box says.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextEdited(pub Entity);

/// Return or Esc in the browser's hidden input, which the game's keys don't see.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SoftKeyPressed(pub soft_keyboard::Key);

/// A press happened this frame, before anything took it.
#[derive(Resource, Default)]
pub(super) struct PressedThisFrame(bool);

/// A place where a tap should raise the on-screen keyboard, other than a box.
#[derive(Component)]
pub(super) struct RaisesKeyboard;

/// A box of this kind, with `node` for its size and where it goes.
pub fn spawn_text_box<'a>(
    parent: &'a mut ChildSpawnerCommands,
    text_box: TextBox,
    font_size: f32,
    node: Node,
) -> EntityCommands<'a> {
    let mut holder = parent.spawn((
        text_box,
        Pressable,
        Node {
            border: UiRect::all(px(2)),
            padding: UiRect::horizontal(px(SIDE_PADDING)),
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..node
        },
        BorderColor::from(INK),
    ));
    let entity = holder.id();
    holder.with_child((
        TextBoxLabel(entity),
        Text::new(""),
        TextFont::from_font_size(font_size),
        TextColor(INK),
    ));
    holder
}

/// What of `typed` a box keeps.
fn allowed(typed: &str, digits_only: bool) -> String {
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

/// Who has the keyboard, and what they type.
#[allow(clippy::too_many_arguments)]
pub(super) fn take_text(
    mut focus: ResMut<TextFocus>,
    mut boxes: Query<&mut TextBox>,
    mut pressed: MessageReader<ButtonPressed>,
    noted: Res<PressedThisFrame>,
    mut typed: MessageReader<KeyboardInput>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut edited: MessageWriter<TextEdited>,
    mut soft_keys: MessageWriter<SoftKeyPressed>,
    mut told_soft_keyboard: Local<(Option<Entity>, String)>,
) {
    let on_box = pressed
        .read()
        .map(|press| press.0)
        .filter(|&entity| boxes.contains(entity))
        .last();
    let holds_keys = |focus: &TextFocus, boxes: &Query<&mut TextBox>| {
        focus
            .0
            .and_then(|entity| boxes.get(entity).ok())
            .is_some_and(|text_box| text_box.takes_keys)
    };
    if on_box.is_some() {
        focus.0 = on_box;
    } else if noted.0 && holds_keys(&focus, &boxes) {
        focus.0 = None;
    }
    let typed: Vec<KeyboardInput> = typed.read().cloned().collect();
    let soft: Vec<_> = soft_keyboard::take_keys();
    let gave_up = !soft.is_empty();
    for key in soft {
        soft_keys.write(SoftKeyPressed(key));
    }
    let Some(entity) = focus.0 else {
        return;
    };
    let Ok(mut text_box) = boxes.get_mut(entity) else {
        focus.0 = None;
        return;
    };

    let digits_only = text_box.digits_only;
    let mut text = text_box.text.clone();
    for key in typed.iter().filter(|key| key.state == KeyState::Pressed) {
        match &key.logical_key {
            // Opens and closes the debug menu.
            _ if key.key_code == KeyCode::Backquote => {}
            Key::Backspace => {
                text.pop();
            }
            Key::Space => text.push_str(&allowed(" ", digits_only)),
            Key::Character(typed) => text.push_str(&allowed(typed, digits_only)),
            _ => {}
        }
    }
    if text != text_box.text {
        // The browser's hidden input is told below.
    } else if let Some(typed) = soft_keyboard::typed(&text) {
        text = allowed(&typed, digits_only);
    }
    if text != text_box.text {
        text_box.text.clone_from(&text);
        edited.write(TextEdited(entity));
    }
    if told_soft_keyboard.0 != Some(entity) || told_soft_keyboard.1 != text {
        soft_keyboard::show_text(&text);
        *told_soft_keyboard = (Some(entity), text);
    }
    if text_box.takes_keys {
        let done = keys.just_pressed(KeyCode::Enter)
            || keys.just_pressed(KeyCode::NumpadEnter)
            || keys.just_pressed(KeyCode::Escape)
            || gave_up;
        if done {
            focus.0 = None;
        }
        keys.reset_all();
    }
}

/// Keeps what boxes say on show, with a caret in the one with the keyboard.
pub(super) fn show_text_boxes(
    focus: Res<TextFocus>,
    boxes: Query<&TextBox>,
    mut labels: Query<(&TextBoxLabel, &mut Text)>,
) {
    for (label, mut text) in &mut labels {
        let Ok(text_box) = boxes.get(label.0) else {
            continue;
        };
        let caret = if focus.0 == Some(label.0) { CARET } else { "" };
        let wanted = format!("{}{}{caret}", text_box.prefix, text_box.text);
        if text.0 != wanted {
            text.0 = wanted;
        }
    }
}

/// Tells the browser where a tap should raise the on-screen keyboard.
#[allow(clippy::type_complexity)]
pub(super) fn place_soft_keyboard(
    places: Query<
        (&ComputedNode, &UiGlobalTransform, &InheritedVisibility),
        Or<(With<TextBox>, With<RaisesKeyboard>)>,
    >,
) {
    let places = places
        .iter()
        .filter(|(.., visible)| visible.get())
        .map(|(node, transform, _)| {
            let scale = node.inverse_scale_factor;
            Rect::from_center_size(transform.translation * scale, node.size() * scale)
        })
        .collect();
    soft_keyboard::set_boxes(places);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_boxes_only_keep_digits() {
        assert_eq!(allowed("a1 b2", true), "12");
        assert_eq!(allowed("a1 b2", false), "a1 b2");
        assert_eq!(allowed("a\nb", false), "ab");
    }
}
