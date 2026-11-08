//! How characters move: one step at a time, to one of the 8 grid points
//! around them. The client checks a step before showing it and the server
//! checks it again before accepting it, both with these rules.

use serde::{Deserialize, Serialize};

use crate::climate::Climate;
use crate::dungeon::{Cell, Level};
use crate::road::RoadKind;
use crate::surface::Surface;
use crate::world::WorldParams;

mod gait;
mod training;

pub use gait::{CLIMB_SAVING_PER_MILLE, FULL, Gait, MOVEMENT_SKILLS, MovementSkills, RUN_TENTHS};
pub use training::{
    CLIMB_PRACTICE_PER_ELEVATION, MOST_CLIMB_PRACTICE, RUN_PRACTICE, climb_practice,
};

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
/// a walkable cell, or over one cell of liquid or hole onto a walkable cell
/// beyond it (a jump, landing two cells away; nobody stops there). (Cutting a
/// wall's corner diagonally is allowed.)
pub fn step_on_level(level: &Level, from: (i64, i64), step: Step) -> Option<(i64, i64)> {
    let (x, y) = step.from(from);
    if level.is_walkable(x, y) {
        return Some((x, y));
    }
    let (far_x, far_y) = step.from((x, y));
    (level.cell(x, y).is_jumpable() && level.is_walkable(far_x, far_y)).then_some((far_x, far_y))
}

/// Whether a mob can stand on a cell: walkable, and not a World Gate's, as
/// only players go through gates (to the Hall of Worlds).
pub fn mob_can_stand(level: &Level, x: i64, y: i64) -> bool {
    level.is_walkable(x, y) && level.cell(x, y) != Cell::WorldGate
}

/// `step_on_level` for a mob: never onto a World Gate's cell.
pub fn mob_step_on_level(level: &Level, from: (i64, i64), step: Step) -> Option<(i64, i64)> {
    step_on_level(level, from, step).filter(|&(x, y)| mob_can_stand(level, x, y))
}

/// What a step costs per cell it covers, in units of time: walking is timed by
/// distance, so a diagonal takes about 1.414 times as long as a straight step.
/// Whole numbers, because game state is (99 / 70 is within 0.01% of the root).
pub const STRAIGHT_COST: u32 = 70;
pub const DIAGONAL_COST: u32 = 99;

/// What moving from one point to another by a single step costs: a jump
/// covers two cells, so it costs two steps' worth. Only for a step
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

/// What decides where can be walked on the world map right now, as the server
/// says.
#[derive(Clone, Copy, Debug)]
pub struct WorldConditions {
    /// How high the water is (`WorldParams::water_level`, for the hour).
    pub water: f64,
    /// Whether frozen sea counts as ground: always for walking, never for
    /// laying roads.
    pub ice: bool,
}

/// The world map's ground, as far as walking on it goes.
pub struct WorldGround {
    surface: Surface,
    /// Grid units around the equator: the poles are a quarter of it away.
    width: i64,
    /// Where the sea is frozen, to walk on.
    climate: Climate,
}

impl WorldGround {
    pub fn new(world: &WorldParams) -> Self {
        Self {
            width: Surface::new(world).width(),
            surface: Surface::new(world),
            climate: Climate::new(world),
        }
    }

    /// How high it counts as for walking here, in whole
    /// `ELEVATIONS_PER_HEIGHT`ths of the surface's broad height (without small
    /// bumps, which are the same whichever way round a route goes and would
    /// drown out the mountains). `None` if it is open water; ice can be walked.
    pub fn elevation(&self, x: i64, y: i64, conditions: WorldConditions) -> Option<i32> {
        if self.past_the_ice(y) {
            return None;
        }
        let WorldConditions { water, ice } = conditions;
        let sample = self.surface.sample(x, y);
        let frozen = || ice && sample.is_frozen(water, self.climate.sample(x, y, sample.height));
        (sample.height > water || frozen())
            .then(|| (self.surface.broad_height(x, y) * ELEVATIONS_PER_HEIGHT).floor() as i32)
    }
}

impl WorldGround {
    /// Whether `y` is past `climate::ICE_LATITUDE` (80°), where the ice caps
    /// can't be crossed (owner: the map stretches badly near the poles). Not
    /// in the showcase's plain beyond the pole.
    pub fn past_the_ice(&self, y: i64) -> bool {
        past_the_ice(self.width, y)
    }
}

/// `WorldGround::past_the_ice` for a planet `width` points round, without
/// making its ground.
pub fn past_the_ice(width: i64, y: i64) -> bool {
    // 80° of the 90° to a pole, a quarter of the width away: 2/9 of it.
    !crate::showcase::in_plain(y) && y.abs() * 9 >= width * 2
}

/// How finely heights count for walking. Whole numbers, because game state is.
const ELEVATIONS_PER_HEIGHT: f64 = 4.0;

/// The extra time a step takes per elevation it climbs or descends. Measured by
/// drawing routes over real mountains: at 9 they go straight over summits, at
/// 20 they only shave them, at 45 they go round a mountain about 4 chunks across
/// (a bigger one's detour costs more than its climb), and at 90 detours get wide.
pub(crate) const SLOPE_COST: u32 = 45;

/// Where `step` from `(x, y)` leads on the world map, if it can be taken: onto
/// dry ground or ice. Anyone standing in open water may step anywhere, so that
/// nobody caught there (by the rising tide, thawing ice, a new world's start, a
/// teleport) is stuck; but never past the ice (`WorldGround::past_the_ice`).
pub fn step_on_world_map(
    ground: &WorldGround,
    conditions: WorldConditions,
    from: (i64, i64),
    step: Step,
) -> Option<(i64, i64)> {
    let (x, y) = step.from(from);
    let allowed = (ground.elevation(from.0, from.1, conditions).is_none()
        || ground.elevation(x, y, conditions).is_some())
        && !ground.past_the_ice(y);
    allowed.then_some((x, y))
}

/// What a step on the world map costs, given the elevation at each end and the
/// road it runs along, if any (`RoadSteps::kind_of`): its distance, plus the
/// slope (less with `gait`'s Climbing), divided by the road's speed. Water
/// counts as flat. Routes are found by this; the time a step takes is
/// `world_step_time`.
pub fn world_step_cost(
    from: (i64, i64),
    to: (i64, i64),
    elevations: (Option<i32>, Option<i32>),
    road: Option<RoadKind>,
    gait: Gait,
) -> u32 {
    let cost = step_cost(from, to) + gait.climb(climbed(elevations) * SLOPE_COST);
    match road {
        Some(kind) => cost * 100 / kind.speed_percent(),
        None => cost,
    }
}

/// How long a step on the world map takes: `world_step_cost`, quicker with
/// `gait`'s Running. Running is left out of routes, as it speeds every step
/// alike and would make the search's estimate too high.
pub fn world_step_time(
    from: (i64, i64),
    to: (i64, i64),
    elevations: (Option<i32>, Option<i32>),
    road: Option<RoadKind>,
    gait: Gait,
) -> u32 {
    gait.run(world_step_cost(from, to, elevations, road, gait))
}

/// How many elevations a step climbs or descends: none to or from water.
pub fn climbed(elevations: (Option<i32>, Option<i32>)) -> u32 {
    match elevations {
        (Some(a), Some(b)) => a.abs_diff(b),
        _ => 0,
    }
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
mod tests;
