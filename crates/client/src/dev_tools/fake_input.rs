//! Fake input for the debug API (only with the `debug-api` feature): moving
//! the pointer, clicking, dragging, scrolling, tapping, keys and typing, as
//! Bevy's own input messages, so the game can't tell them from a person. The
//! pointer is `FakeCursor`, never the real one on the screen, and while it is
//! held the real mouse's presses and wheel are ignored.
//! Each request is queued, and one step is played per frame, so presses and
//! releases land on separate frames as they would by hand. See
//! `docs/context/debug-api.md`.

use bevy::input::InputSystems;
use bevy::prelude::*;
use server::debug_api::AddDebugMethod;

// Split into files (`cargo xtask lint` keeps files small).
mod keys;
mod mouse;
mod play;
mod real;
mod wait;

use keys::{key, type_text};
use mouse::{click, drag, move_to, one, scroll};
use play::{FakeInput, Step, play, queued};
use real::ignore_real_mouse;
use wait::{input, wait};

/// The id fake fingers use, well away from real ones.
const FINGER: u64 = 9_000_000;

pub(super) fn add(app: &mut App) {
    app.init_resource::<FakeInput>()
        .add_systems(
            PreUpdate,
            (play, ignore_real_mouse).chain().before(InputSystems),
        )
        .add_debug_method("client.move", move_to)
        .add_debug_method("client.click", click)
        .add_debug_method("client.drag", drag)
        .add_debug_method("client.scroll", scroll)
        .add_debug_method("client.key", key)
        .add_debug_method("client.type", type_text)
        .add_debug_method("client.wait", wait)
        .add_debug_method("client.input", input);
}

#[cfg(test)]
mod tests;
