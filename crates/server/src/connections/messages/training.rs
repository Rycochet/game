//! What steps cost by the player's movement skills, and the practice they
//! give: Running for every step on the world map, Climbing for steep ones there
//! and on levels' slopes, Swimming for wading, Jumping for jumps, Sneaking for
//! a slow pace in the wilderness under a wary eye, and Weight Training under a
//! heavy load (`shared::movement`'s training settings), at
//! most once per skill every 5 seconds, as blows are.

use shared::energy::level_move_energy;
use shared::movement::{
    Gait, JUMP_PRACTICE, LevelWay, MOVEMENT_SKILLS, PIONEER_PRACTICE, RIVER_SWIM_PRACTICE,
    RUN_PRACTICE, SNEAK_PRACTICE, SWIM_PRACTICE, Terrain, WildHeights, climb_practice, climbed,
    footing_on, weight_practice, world_step_time,
};
use shared::protocol::ServerMessage;
use shared::spot::Spot;

use crate::ServerWorld;
use crate::connections::Client;

/// What a step was like, for practice.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Moved {
    /// On the world map: practises Running.
    pub on_world_map: bool,
    /// Elevations climbed or descended (a world map step's, or 16ths a cell on
    /// a level's smooth ground, the same steepness).
    pub elevations: u32,
    /// Swimming practice from wading.
    pub swam: u32,
    /// A jump over something: practises Jumping.
    pub jumped: bool,
    /// At a sneaking pace in the wilderness, watched by something wary:
    /// practises Sneaking.
    pub sneaked: bool,
    /// The terrain it slowed over: practises its Pioneering skill.
    pub terrain: Option<Terrain>,
}

/// What a move on a level was like, worked out before it is taken: its energy
/// (before any sneaking pace) and what it practises.
#[derive(Clone, Copy, Debug)]
pub(super) struct Walked {
    pub energy: u32,
    /// Its length in units.
    pub length: u64,
    /// The cells it crossed (0 within one): practice comes only from those.
    pub cells: u32,
    /// 16ths of an elevation a cell, on the level's smooth ground.
    pub steepness: u32,
    pub wades: bool,
    pub jumped: bool,
    pub terrain: Option<Terrain>,
}

impl Walked {
    /// A move from `from` to `to` along `way` (the shared rule,
    /// `movement::LevelWay`).
    pub(super) fn of(way: &LevelWay, from: Spot, to: Spot, jumped: bool) -> Self {
        let (a, b) = (from.cell(), to.cell());
        let footing = footing_on(way.level, b, way.terrain);
        Self {
            energy: level_move_energy(way.cost(from, to, jumped)),
            length: from.distance(to),
            cells: (a.0 - b.0).abs().max((a.1 - b.1).abs()) as u32,
            steepness: way.steepness(from, to),
            wades: footing.wades(),
            jumped,
            terrain: footing.slowed_by(),
        }
    }
}

/// The wilderness's terrain at a cell (`WildHeights::terrain`).
pub(super) fn terrain_at(world: &ServerWorld, (x, y): (i64, i64)) -> Option<Terrain> {
    WildHeights::new(&world.ground, (x, y, 1, 1)).terrain(x, y)
}

impl Client {
    /// What a move on a level takes (`walked`, in the wilderness if `wild`): no
    /// quicker than a sneaking pace while sneaking. Moving into another cell
    /// practises Climbing by its steepness, Swimming by wading, Jumping and,
    /// slowly in the wilderness under a wary eye, Sneaking. Whether the client
    /// is still there, and the cost.
    pub(super) fn level_move(
        &mut self,
        world: &ServerWorld,
        walked: Walked,
        wild: bool,
    ) -> (bool, u32) {
        let cost = self.sneak_cost(walked.energy, walked.length);
        if walked.cells == 0 {
            return (true, cost);
        }
        let now = world.action_clock.ticks();
        let slow = self.learning.sneak.stepped(now, walked.cells);
        let sneaked = wild && slow && self.watched(world);
        let moved = Moved {
            on_world_map: false,
            elevations: walked.steepness,
            swam: if walked.wades { SWIM_PRACTICE } else { 0 },
            jumped: walked.jumped,
            sneaked,
            terrain: walked.terrain,
        };
        (self.practise_moving(moved, now), cost)
    }

    /// What a world map step from `from` to `to` (before any wrapping) costs,
    /// with the elevations at each end: its slope, any road, wading a river
    /// off the road or swimming the sea, and the player's skills. Whether the
    /// client is still there, and the cost.
    pub(super) fn world_step(
        &mut self,
        world: &mut ServerWorld,
        (from, to): ((i64, i64), (i64, i64)),
        ends: (Option<i32>, Option<i32>),
    ) -> (bool, u32) {
        world.river_points.cover(&world.rivers, to);
        let road = world.road_steps.kind_of(from, to);
        // Swimming the sea, or wading a river.
        let on_river = world.river_points.contains(to);
        let footing = world.ground.footing(to, world.conditions(), road, on_river);
        let river = footing.wades();
        let gait = Gait::of(&self.sheet.skills);
        let cost = world_step_time(from, to, ends, footing, gait);
        let moved = Moved {
            on_world_map: true,
            elevations: climbed(ends),
            swam: if river && road.is_none() {
                RIVER_SWIM_PRACTICE
            } else {
                0
            },
            jumped: false,
            sneaked: false,
            terrain: footing.slowed_by(),
        };
        let now = world.action_clock.ticks();
        (self.practise_moving(moved, now), cost)
    }

    /// Whether something wary on the player's level watches them where they
    /// stand (`entities::watched_warily`).
    fn watched(&self, world: &ServerWorld) -> bool {
        let Some(visit) = &self.visit else {
            return false;
        };
        let kind = self.inventory.creature;
        let character = self.character_id.as_ref();
        crate::entities::watched_warily(
            world,
            visit.population(),
            visit.position(),
            kind,
            character,
        )
    }

    /// Practice from a step: Running on the world map, Climbing by its
    /// steepness, Swimming from wading, and Weight Training while carrying more
    /// than half of what can be carried, on any map. The client is sent its
    /// `Sheet` only when that changes how it moves or what it can carry (or
    /// something rusted), as its steps are paced by it. `false` if the client
    /// has gone.
    pub(super) fn practise_moving(&mut self, moved: Moved, now: u64) -> bool {
        let size = self.inventory.creature.def().size;
        let before = (Gait::of(&self.sheet.skills), self.sheet.capacity(size));
        let load = self.sheet.load(&self.inventory);
        let played = self.activity.played;
        let skills = &*MOVEMENT_SKILLS;
        let running = if moved.on_world_map { RUN_PRACTICE } else { 0 };
        let mut rusted = false;
        if let Some(terrain) = moved.terrain {
            let skill = terrain.skill();
            let amount = self.learning.skills.take(skill, PIONEER_PRACTICE, now);
            if amount > 0 {
                self.sheet.skills.practise(skill, amount);
                rusted |= self.sheet.skills.used(skill, played);
                self.unsaved = true;
            }
        }
        for (skill, amount) in [
            (skills.running, running),
            (skills.climbing, climb_practice(moved.elevations)),
            (skills.swimming, moved.swam),
            (skills.jumping, if moved.jumped { JUMP_PRACTICE } else { 0 }),
            (
                skills.sneaking,
                if moved.sneaked { SNEAK_PRACTICE } else { 0 },
            ),
            (skills.weight_training, weight_practice(load)),
        ] {
            if amount == 0 {
                continue;
            }
            let amount = self.learning.skills.take(skill, amount, now);
            if amount == 0 {
                continue;
            }
            self.sheet.skills.practise(skill, amount);
            rusted |= self.sheet.skills.used(skill, played);
            self.unsaved = true;
        }
        let after = (Gait::of(&self.sheet.skills), self.sheet.capacity(size));
        if !rusted && after == before {
            return true;
        }
        // Carrying more may lighten the load, and so change the speed.
        if after.1 != before.1 {
            self.update_speed(now);
        }
        self.link.send(&ServerMessage::Sheet(self.sheet.clone()))
    }
}

#[cfg(test)]
mod tests;
