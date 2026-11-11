//! Turning the mouse's buttons and wheel into events on what the pointer is
//! on. A finger's tap arrives as a click of the left button (`touch`), so it
//! is a `Click` here too.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;

use super::events::{Click, ContextMenu, Press, Release, Wheel};
use super::hit::{Hit, PointerCapture};
use crate::dom::parts::{Canvas, Element};

/// A second click this soon after the first, on the same thing, is a double
/// click. In seconds, by the real clock.
const DOUBLE_CLICK: f32 = 0.4;
/// How far the wheel turns for one notch, on a device that scrolls by pixels.
const PIXELS_PER_NOTCH: f32 = 40.0;

/// What the mouse's buttons did this frame, noted before anything takes a
/// press away from what is behind it.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Buttons {
    pub left_down: bool,
    pub left_up: bool,
    pub right_down: bool,
}

impl Buttons {
    pub fn of(mouse: &ButtonInput<MouseButton>) -> Self {
        Self {
            left_down: mouse.just_pressed(MouseButton::Left),
            left_up: mouse.just_released(MouseButton::Left),
            right_down: mouse.just_pressed(MouseButton::Right),
        }
    }
}

/// What a click is worked out from.
#[derive(Default)]
pub(super) struct Clicks {
    /// What the left button went down on, and all it is in.
    down: Vec<Entity>,
    /// The last thing clicked, and when.
    last: Option<(Entity, f32)>,
}

/// Sends this frame's events. A press on an element goes no further: the
/// game behind never sees it, but for a right press on a place its owner
/// draws in.
#[allow(clippy::too_many_arguments)]
pub(super) fn send_events(
    mut commands: Commands,
    hit: Res<Hit>,
    buttons: Res<Buttons>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time<Real>>,
    elements: Query<Has<Canvas>, With<Element>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut capture: ResMut<PointerCapture>,
    mut clicks: Local<Clicks>,
) {
    let at = hit.at.unwrap_or_default();
    let element = hit.target.and_then(|target| elements.get(target).ok());
    let on_element = element.is_some();
    // A canvas, or a grid or slot of things, is its owner's: a right press
    // there is the owner's to take (turning what is held).
    let owners = element == Some(true);
    if buttons.left_down {
        clicks.down.clone_from(&hit.chain);
        if let Some(entity) = hit.target {
            commands.trigger(Press { entity, at });
        }
        if on_element {
            mouse.clear_just_pressed(MouseButton::Left);
        }
    }
    if buttons.left_up {
        capture.set_if_neq(PointerCapture(None));
        if let Some(entity) = hit.target {
            commands.trigger(Release { entity, at });
        }
        // The nearest thing both the press and the letting go were on.
        let down = std::mem::take(&mut clicks.down);
        if let Some(entity) = down.into_iter().find(|held| hit.chain.contains(held)) {
            let now = time.elapsed_secs();
            let again = clicks
                .last
                .is_some_and(|(last, when)| last == entity && now - when <= DOUBLE_CLICK);
            // A third click starts again, so three aren't two double clicks.
            clicks.last = (!again).then_some((entity, now));
            let count = if again { 2 } else { 1 };
            commands.trigger(Click { entity, at, count });
        }
    }
    if buttons.right_down {
        if let Some(entity) = hit.target {
            commands.trigger(ContextMenu { entity, at });
        }
        if on_element && !owners {
            mouse.clear_just_pressed(MouseButton::Right);
        }
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_NOTCH,
    };
    if lines != 0.0
        && let Some(entity) = hit.target
    {
        commands.trigger(Wheel { entity, lines });
    }
}
