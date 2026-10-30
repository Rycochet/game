//! How characters move: one step at a time, to one of the 8 grid points
//! around them. The client checks a step before showing it and the server
//! checks it again before accepting it, both with these rules.

use serde::{Deserialize, Serialize};

use crate::dungeon::Level;

/// One step: `dx` and `dy` each -1, 0 or 1, not both 0 (`y` south). Made
/// only through `Step::new`, also when read from a message, so a step is
/// never longer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "(i8, i8)", into = "(i8, i8)")]
pub struct Step {
    dx: i8,
    dy: i8,
}

impl Step {
    /// The step by `(dx, dy)`, if that is a step.
    pub fn new(dx: i64, dy: i64) -> Option<Self> {
        let fits = (-1..=1).contains(&dx) && (-1..=1).contains(&dy);
        (fits && (dx, dy) != (0, 0)).then_some(Self {
            dx: dx as i8,
            dy: dy as i8,
        })
    }

    pub fn dx(self) -> i64 {
        self.dx.into()
    }

    pub fn dy(self) -> i64 {
        self.dy.into()
    }

    /// Where it leads from `(x, y)`.
    pub fn from(self, (x, y): (i64, i64)) -> (i64, i64) {
        (x + self.dx(), y + self.dy())
    }
}

impl TryFrom<(i8, i8)> for Step {
    type Error = &'static str;

    fn try_from((dx, dy): (i8, i8)) -> Result<Self, Self::Error> {
        Self::new(dx.into(), dy.into()).ok_or("not a step")
    }
}

impl From<Step> for (i8, i8) {
    fn from(step: Step) -> Self {
        (step.dx, step.dy)
    }
}

/// Where `step` from `(x, y)` leads on a site level, if it can be taken: onto
/// a walkable cell, or over one cell of liquid onto a walkable cell beyond it
/// (a jump, landing two cells away; nobody stops on liquid). (Cutting a wall's
/// corner diagonally is allowed.)
pub fn step_on_level(level: &Level, from: (i64, i64), step: Step) -> Option<(i64, i64)> {
    let (x, y) = step.from(from);
    if level.is_walkable(x, y) {
        return Some((x, y));
    }
    let (far_x, far_y) = step.from((x, y));
    (level.cell(x, y).is_liquid() && level.is_walkable(far_x, far_y)).then_some((far_x, far_y))
}

/// What a step costs per cell it covers, in units of time: walking is timed by
/// distance, so a diagonal takes about 1.414 times as long as a straight step.
/// Whole numbers, because game state is (99 / 70 is within 0.01% of the root).
pub const STRAIGHT_COST: u32 = 70;
pub const DIAGONAL_COST: u32 = 99;

/// What moving from one point to another by a single step costs: a jump over
/// liquid covers two cells, so it costs two steps' worth. Only for a step
/// `step_on_level` has allowed. (Terrain costs will go here.)
pub fn step_cost(from: (i64, i64), to: (i64, i64)) -> u32 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let per_cell = if dx != 0 && dy != 0 {
        DIAGONAL_COST
    } else {
        STRAIGHT_COST
    };
    per_cell * dx.abs().max(dy.abs()) as u32
}

/// Where `step` from `(x, y)` leads on the world map: anywhere, for now
/// (no terrain stops anyone yet).
pub fn step_on_world_map(from: (i64, i64), step: Step) -> (i64, i64) {
    step.from(from)
}

/// The steps from one point to another: diagonally first, then straight.
/// None if they are the same point.
pub fn steps_between(from: (i64, i64), to: (i64, i64)) -> Vec<Step> {
    let (mut x, mut y) = from;
    let mut steps = Vec::new();
    while (x, y) != to {
        let step = Step::new((to.0 - x).signum(), (to.1 - y).signum()).expect("not there yet");
        (x, y) = step.from((x, y));
        steps.push(step);
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dungeon::Cell;

    #[test]
    fn only_single_steps_exist() {
        assert!(Step::new(1, -1).is_some());
        for (dx, dy) in [(0, 0), (2, 0), (0, -2), (5, 5)] {
            assert!(Step::new(dx, dy).is_none(), "{dx}, {dy}");
        }
        // Not even when read from a message.
        let long = postcard::to_allocvec(&(2i8, 0i8)).unwrap();
        assert!(postcard::from_bytes::<Step>(&long).is_err());
        let fine = postcard::to_allocvec(&Step::new(-1, 1).unwrap()).unwrap();
        assert_eq!(
            postcard::from_bytes::<Step>(&fine),
            Ok(Step::new(-1, 1).unwrap())
        );
    }

    #[test]
    fn steps_on_a_level_need_walkable_cells() {
        let mut level = Level::test_room();
        let east = Step::new(1, 0).unwrap();
        assert_eq!(step_on_level(&level, (0, 0), east), Some((1, 0)));
        level.set(1, 0, Cell::Wall);
        assert_eq!(step_on_level(&level, (0, 0), east), None);
    }

    #[test]
    fn one_cell_of_liquid_is_jumped() {
        let mut level = Level::test_room();
        let east = Step::new(1, 0).unwrap();
        for liquid in [Cell::Water, Cell::Lava] {
            level.set(1, 0, liquid);
            assert_eq!(step_on_level(&level, (0, 0), east), Some((2, 0)));
        }
        // Diagonally too, across a corner.
        let north_west = Step::new(-1, -1).unwrap();
        level.set(3, 1, Cell::Water);
        assert_eq!(step_on_level(&level, (4, 2), north_west), Some((2, 0)));
        // Two wide can't be crossed, so it can't be stepped onto at all.
        level.set(2, 0, Cell::Water);
        assert_eq!(step_on_level(&level, (0, 0), east), None);
        // Nor with a wall to land on.
        level.set(2, 0, Cell::Wall);
        assert_eq!(step_on_level(&level, (0, 0), east), None);
    }

    #[test]
    fn steps_cost_the_distance_they_cover() {
        assert_eq!(step_cost((0, 0), (1, 0)), STRAIGHT_COST);
        assert_eq!(step_cost((0, 0), (-1, 1)), DIAGONAL_COST);
        assert_eq!(step_cost((0, 0), (0, -2)), 2 * STRAIGHT_COST);
        assert_eq!(step_cost((0, 0), (2, 2)), 2 * DIAGONAL_COST);
    }

    #[test]
    fn steps_between_points_add_up() {
        let steps = steps_between((0, 0), (3, -1));
        assert_eq!(steps.len(), 3);
        let end = steps.iter().fold((0, 0), |at, step| step.from(at));
        assert_eq!(end, (3, -1));
        assert!(steps_between((4, 4), (4, 4)).is_empty());
    }
}
