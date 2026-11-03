//! Areas that scroll by themselves, slowly down and back up again, until
//! someone scrolls them by hand (for credits and the like).

use bevy::prelude::*;

use super::scroll::{ScrollArea, furthest};

/// How long to wait at the top before going down, and at the bottom before
/// coming back up.
const WAIT_AT_TOP: f32 = 2.0;
const WAIT_AT_BOTTOM: f32 = 4.0;
/// How fast it goes, in interface pixels a second.
const SPEED: f32 = 30.0;

/// Put this on a scroll area (from `spawn_scroll_area`) to have it scroll by
/// itself while it has more than it can show. Scrolling it by hand in any way
/// removes this; put it back (with its position at the top) to start again.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct AutoScroll {
    phase: Phase,
    /// Seconds waited so far, while waiting.
    waited: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Phase {
    #[default]
    WaitingAtTop,
    Down,
    WaitingAtBottom,
    Up,
}

impl AutoScroll {
    /// After `seconds` more, at `position` in an area that can go as far as
    /// `furthest`: what it is doing, and where the area is.
    fn after(self, seconds: f32, position: f32, furthest: f32) -> (Self, f32) {
        if furthest <= 0.0 {
            return (Self::default(), 0.0);
        }
        let waited = self.waited + seconds;
        let waiting = |phase, waited| Self { phase, waited };
        match self.phase {
            Phase::WaitingAtTop if waited >= WAIT_AT_TOP => (waiting(Phase::Down, 0.0), position),
            Phase::WaitingAtBottom if waited >= WAIT_AT_BOTTOM => {
                (waiting(Phase::Up, 0.0), position)
            }
            Phase::WaitingAtTop | Phase::WaitingAtBottom => (waiting(self.phase, waited), position),
            Phase::Down => {
                let moved = (position + SPEED * seconds).min(furthest);
                let phase = if moved >= furthest {
                    Phase::WaitingAtBottom
                } else {
                    Phase::Down
                };
                (waiting(phase, 0.0), moved)
            }
            Phase::Up => {
                let moved = (position - SPEED * seconds).max(0.0);
                let phase = if moved <= 0.0 {
                    Phase::WaitingAtTop
                } else {
                    Phase::Up
                };
                (waiting(phase, 0.0), moved)
            }
        }
    }
}

/// Moves areas that scroll by themselves. By the real clock, as the game's stops
/// while the system menu pauses it.
pub(super) fn auto_scroll(
    time: Res<Time<Real>>,
    mut areas: Query<
        (
            &mut AutoScroll,
            &ComputedNode,
            &InheritedVisibility,
            &mut ScrollPosition,
        ),
        With<ScrollArea>,
    >,
) {
    for (mut auto, node, visible, mut position) in &mut areas {
        if !visible.get() {
            continue;
        }
        let (next, at) = auto.after(time.delta_secs(), position.y, furthest(node));
        auto.set_if_neq(next);
        if position.y != at {
            position.y = at;
        }
    }
}

#[cfg(test)]
mod tests;
