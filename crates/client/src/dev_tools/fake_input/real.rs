//! While the fake cursor is held, the real mouse's presses and wheel are
//! ignored, so a person's hand on the mouse can't get in a test's way: only
//! the clicks fake input makes reach the game. Letting go of a button still
//! does, so nothing is left held down.

use bevy::ecs::message::{Message, MessageCursor};
use bevy::input::ButtonState;
use bevy::input::mouse::{MouseButtonInput, MouseWheel};
use bevy::prelude::*;

use super::{FakeInput, Step};
use crate::dev_tools::FakeCursor;

/// Runs after `play` and before Bevy reads the frame's input.
pub(super) fn ignore_real_mouse(
    fake: Res<FakeInput>,
    cursor: Res<FakeCursor>,
    mut presses: ResMut<Messages<MouseButtonInput>>,
    mut press_cursor: Local<MessageCursor<MouseButtonInput>>,
    mut wheel: ResMut<Messages<MouseWheel>>,
    mut wheel_cursor: Local<MessageCursor<MouseWheel>>,
) {
    let held = cursor.at.is_some();
    let played_press = matches!(fake.played, Some(Step::Mouse(..)));
    let played_wheel = matches!(fake.played, Some(Step::Wheel(_)));
    keep_only(
        &mut presses,
        &mut press_cursor,
        held,
        played_press,
        |press| press.state == ButtonState::Released,
    );
    keep_only(&mut wheel, &mut wheel_cursor, held, played_wheel, |_| false);
}

/// Of the messages new since last frame (the last of them fake if
/// `played_fake`), drops the real ones `keep` refuses while `held`. Dropping
/// one clears them all, so the others are written again: only new ones, which
/// nothing has read yet, so nothing sees one twice.
fn keep_only<M: Message + Clone>(
    messages: &mut Messages<M>,
    seen: &mut MessageCursor<M>,
    held: bool,
    played_fake: bool,
    keep: impl Fn(&M) -> bool,
) {
    let mut new: Vec<M> = seen.read(messages).cloned().collect();
    let fake = if played_fake { new.pop() } else { None };
    if !held || new.iter().all(&keep) {
        return;
    }
    messages.clear();
    let kept = new.into_iter().filter(|message| keep(message));
    messages.write_batch(kept.chain(fake));
    seen.clear(messages);
}
