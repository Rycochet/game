//! Input by the browser's rules: one search a frame for what the pointer is
//! on, and events sent to that and up through what holds it. Only the thing
//! in front is ever pointed at or pressed.

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::widgets::WidgetSystems;

mod dispatch;
mod events;
mod hit;
mod hover;

pub use events::{Click, ContextMenu, PointerEnter, PointerLeave, Press, Release, Wheel};
#[cfg(test)]
pub use hit::find_now;
pub use hit::{Finder, Hit, PointerCapture};

/// When the search and the events happen, in `PreUpdate`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DomInput {
    /// `Hit` is worked out: before anything reads what the pointer is on.
    Find,
    /// Events are sent: after the old widgets have read the press.
    Send,
}

pub(super) fn add(app: &mut App) {
    app.init_resource::<Hit>()
        .init_resource::<PointerCapture>()
        .init_resource::<dispatch::Buttons>()
        // What the pointer is on is found before any widget reads it, and
        // events are sent once the old widgets have read presses.
        .configure_sets(
            PreUpdate,
            (
                DomInput::Find
                    .after(InputSystems)
                    .after(PointerSystems)
                    .before(WidgetSystems::Presses),
                DomInput::Send
                    .after(WidgetSystems::Presses)
                    .before(WidgetSystems::Front),
            ),
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
