//! Deep water for good swimmers (`Gait::swims_deep`): water on a level can be
//! swum instead of jumped, and the sea near the coast crossed on the world map,
//! never the deep ocean. Everyone else goes by the plain rules.

use super::{Gait, Step, WorldConditions, WorldGround, step_on_level, step_on_world_map};
use crate::dungeon::{Cell, Level};

/// `step_on_level` for someone with this `gait`: a good swimmer steps into
/// water (`Cell::Water`, never lava) rather than jumping it.
pub fn step_on_level_by(
    level: &Level,
    from: (i64, i64),
    step: Step,
    gait: Gait,
) -> Option<(i64, i64)> {
    let to = step.from(from);
    if gait.swims_deep() && level.cell(to.0, to.1) == Cell::Water {
        return Some(to);
    }
    step_on_level(level, from, step)
}

/// `step_on_world_map` for someone with this `gait`: a good swimmer may also
/// step into the sea where it is no deeper than `Gait::deepest_sea`.
pub fn step_on_world_map_by(
    ground: &WorldGround,
    conditions: WorldConditions,
    from: (i64, i64),
    step: Step,
    gait: Gait,
) -> Option<(i64, i64)> {
    step_on_world_map(ground, conditions, from, step).or_else(|| {
        let (x, y) = step.from(from);
        ground.swimmable(x, y, conditions, gait).then_some((x, y))
    })
}

impl WorldGround {
    /// How deep the water is at a point, in height units: `None` on ground
    /// (frozen sea included) and past the ice.
    pub fn sea_depth(&self, x: i64, y: i64, conditions: WorldConditions) -> Option<f64> {
        if self.past_the_ice(y) || self.elevation(x, y, conditions).is_some() {
            return None;
        }
        Some(conditions.water - self.surface.sample(x, y).height)
    }

    /// Whether someone with this `gait` can swim into the sea at a point.
    pub fn swimmable(&self, x: i64, y: i64, conditions: WorldConditions, gait: Gait) -> bool {
        gait.swims_deep()
            && self
                .sea_depth(x, y, conditions)
                .is_some_and(|depth| depth <= gait.deepest_sea())
    }
}

#[cfg(test)]
mod tests;
