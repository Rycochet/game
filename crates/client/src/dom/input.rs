//! Input by the browser's rules: one search a frame for what the pointer is
//! on, and events sent to that and up through what holds it. Only the thing
//! in front is ever pointed at or pressed.

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;

mod dispatch;
mod events;
mod hit;
mod hover;

pub use events::{Click, ContextMenu, PointerEnter, PointerLeave, Press, Release, Wheel};
#[cfg(test)]
pub use hit::find_now;
pub use hit::{Finder, Hit, PointerCapture};

/// When the pointer and the keys are read, in `PreUpdate`, in this order.
/// Anything else taking clicks there runs after all of them.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DomInput {
    /// `Hit` is worked out: before anything reads what the pointer is on.
    Find,
    /// Presses on controls and panels are taken (`ButtonPressed`), so
    /// nothing under them sees them.
    Presses,
    /// Events are sent, once the controls have read the press.
    Send,
    /// As `Modal`, for what stays in front of everything that is in `Modal`
    /// (the system menu).
    Front,
    /// Something covering the screen reads its presses and keys, and takes
    /// all other input.
    Modal,
}

pub(super) fn add(app: &mut App) {
    app.init_resource::<Hit>()
        .init_resource::<PointerCapture>()
        .init_resource::<dispatch::Buttons>()
        .configure_sets(
            PreUpdate,
            (
                DomInput::Find.after(InputSystems).after(PointerSystems),
                DomInput::Presses,
                DomInput::Send,
                DomInput::Front,
                DomInput::Modal,
            )
                .chain(),
        )
        .add_systems(
            PreUpdate,
            (
                (hit::find_target, hover::follow_hover)
                    .chain()
                    .in_set(DomInput::Find),
                dispatch::send_events.in_set(DomInput::Send),
            ),
        );
}

#[cfg(test)]
mod tests;
