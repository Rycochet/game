//! What steps cost by the player's movement skills, and the practice they
//! give: Running for every step on the world map, Climbing for steep ones there
//! and in the wilderness, Swimming for wading, Jumping for jumps, Sneaking for
//! a slow pace in the wilderness under a wary eye, and Weight Training under a
//! heavy load (`shared::movement`'s training settings), at
//! most once per skill every 5 seconds, as blows are.

use shared::energy::level_step_energy;
use shared::movement::{
    Gait, JUMP_PRACTICE, MOVEMENT_SKILLS, RIVER_SWIM_PRACTICE, RUN_PRACTICE, SNEAK_PRACTICE,
    SWIM_PRACTICE, WildHeights, climb_practice, climbed, is_jump, weight_practice, world_step_time,
};
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::Client;

/// What a step was like, for practice.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Moved {
    /// On the world map: practises Running.
    pub on_world_map: bool,
    /// Elevations climbed or descended (a world map step's, or 16ths a cell in
    /// the wilderness, the same steepness).
    pub elevations: u32,
    /// Swimming practice from wading.
    pub swam: u32,
    /// A jump over something: practises Jumping.
    pub jumped: bool,
    /// At a sneaking pace in the wilderness, watched by something wary:
    /// practises Sneaking.
    pub sneaked: bool,
}

impl Client {
    /// What a step on a level from `from` to `to` (where it landed before any
    /// wrapping round the planet) costs: in the wilderness (`wild`) by its slope
    /// and the player's Climbing, and slower if `wading`, less so by Swimming;
    /// no quicker than a sneaking pace while sneaking. Whether the client is
    /// still there, and the cost.
    pub(super) fn level_step(
        &mut self,
        world: &ServerWorld,
        (from, to): ((i64, i64), (i64, i64)),
        wild: bool,
        wading: bool,
    ) -> (bool, u32) {
        let ends = wild.then(|| {
            let corner = (from.0.min(to.0), from.1.min(to.1));
            let size = ((from.0 - to.0).abs() + 1, (from.1 - to.1).abs() + 1);
            let flats = self.visit.as_ref().map(|visit| visit.flats());
            let heights = WildHeights::new(&world.ground, (corner.0, corner.1, size.0, size.1))
                .with_flats(flats.unwrap_or_default());
            (heights.at(from.0, from.1), heights.at(to.0, to.1))
        });
        let gait = Gait::of(&self.sheet.skills);
        // 16ths of an elevation a cell: elevations per world map point, as
        // steepness counts there.
        let cells = (from.0 - to.0).abs().max((from.1 - to.1).abs()).max(1) as u32;
        let cost = self.sneak_cost(level_step_energy(from, to, ends, wading, gait), cells);
        let now = world.action_clock.ticks();
        let slow = self.learning.sneak.stepped(now, cells);
        let sneaked = wild && slow && self.watched(world);
        let moved = Moved {
            on_world_map: false,
            elevations: ends.map_or(0, |(a, b)| a.abs_diff(b) / cells),
            swam: if wading { SWIM_PRACTICE } else { 0 },
            jumped: is_jump(from, to),
            sneaked,
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
        let river = ends.1.is_none() || world.river_points.contains(to);
        let gait = Gait::of(&self.sheet.skills);
        let cost = world_step_time(from, to, ends, road, river, gait);
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
