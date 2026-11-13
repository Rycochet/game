//! Pinning a row's tip beside its "?", as fingers can't hover.

use bevy::prelude::*;

use super::HelpMark;
use crate::dom::input::Hit;
use crate::widgets::{Pinned, PinnedTip};

/// A press on a "?" pins its row's tip beside it (or lets it go, if it was
/// already); any other press, or any key, lets a pinned tip go.
pub(in crate::dom) fn pin_tips(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    hit: Res<Hit>,
    marks: Query<&HelpMark>,
    mut pinned: ResMut<PinnedTip>,
) {
    let pressed = mouse.get_just_pressed().next().is_some();
    if !pressed && keys.get_just_pressed().next().is_none() {
        return;
    }
    let on_mark = |target: Entity| {
        let HelpMark(row) = marks.get(target).ok()?;
        Some(Pinned {
            tip_of: *row,
            beside: target,
        })
    };
    let pressed_mark = mouse
        .just_pressed(MouseButton::Left)
        .then(|| hit.target.and_then(on_mark))
        .flatten();
    let wanted = match pressed_mark {
        Some(mark) if pinned.0 != Some(mark) => Some(mark),
        _ => None,
    };
    if pinned.0 != wanted {
        pinned.0 = wanted;
    }
}
