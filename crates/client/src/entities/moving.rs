//! Others moving: where the server says they are (in units on a level or in
//! the Hall, points on the world map: `protocol::moves`) and the stretches
//! they go along, eased between at the speed each stretch takes.

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::protocol::{Stretch, cell_of_units};

use crate::common::{GameView, MapPosition, Other, SmoothPosition};

/// How fast others are shown moving when the server gives no time, in grid
/// units a second: the same as an ordinary player.
const WORLD_MAP_SPEED: f64 = 60.0;
const SITE_SPEED: f64 = shared::energy::SITE_CELLS_PER_SECOND;
/// Further behind than this, in grid units, they go straight there.
const TOO_FAR_BEHIND: f64 = 8.0;

/// Where someone else is going (in cells or points), and how fast, in grid
/// units a second (`None`: an ordinary walker's pace).
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(super) struct Goal {
    x: f64,
    y: f64,
    speed: Option<f64>,
}

impl Goal {
    /// Standing at `at`.
    pub(super) fn still(at: SmoothPosition) -> Self {
        Self {
            x: at.x,
            y: at.y,
            speed: None,
        }
    }
}

/// Where someone the server says is at `at` is shown: their cell (or point),
/// and exactly where.
pub(super) fn placed(on_level: bool, at: (i64, i64)) -> (MapPosition, SmoothPosition) {
    if on_level {
        let (x, y) = cell_of_units(at);
        (MapPosition { x, y }, SmoothPosition::of_units(at))
    } else {
        let position = MapPosition { x: at.0, y: at.1 };
        (position, SmoothPosition::from(position))
    }
}

/// Where a stretch takes someone, and how they are shown going there.
pub(super) fn stretch(on_level: bool, stretch: &Stretch) -> (MapPosition, Goal) {
    let (position, to) = placed(on_level, stretch.to);
    let (_, from) = placed(on_level, stretch.from);
    let seconds = f64::from(stretch.ticks) / ACTION_TICKS_PER_SECOND;
    let length = (to.x - from.x).hypot(to.y - from.y);
    // No time given: there at once (as good as).
    let speed = if seconds > 0.0 {
        length / seconds
    } else {
        f64::MAX
    };
    let goal = Goal {
        x: to.x,
        y: to.y,
        speed: Some(speed),
    };
    (position, goal)
}

/// `distance` towards `to`, stopping exactly on it.
pub(super) fn glide(from: Vec2d, to: Vec2d, distance: f64) -> Vec2d {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let gap = dx.hypot(dy);
    if gap <= distance || gap > TOO_FAR_BEHIND {
        to
    } else {
        (from.0 + dx / gap * distance, from.1 + dy / gap * distance)
    }
}

type Vec2d = (f64, f64);

/// Eases others towards where they are going, between the server's word.
pub(super) fn show_moving(
    time: Res<Time>,
    view: Res<State<GameView>>,
    mut others: Query<(&Goal, &mut SmoothPosition), With<Other>>,
) {
    let usual = match view.get() {
        GameView::Dungeon => SITE_SPEED,
        _ => WORLD_MAP_SPEED,
    };
    for (goal, mut smooth) in &mut others {
        let distance = goal.speed.unwrap_or(usual) * time.delta_secs_f64();
        let (x, y) = glide((smooth.x, smooth.y), (goal.x, goal.y), distance);
        if (x, y) != (smooth.x, smooth.y) {
            *smooth = SmoothPosition { x, y };
        }
    }
}
