//! Practice from moving: Running for every step on the world map, Climbing for
//! steep ones there and in the wilderness (`shared::movement`'s training
//! settings), at most once per skill every 5 seconds, as blows are.

use shared::energy::wild_step_energy;
use shared::movement::{Gait, MOVEMENT_SKILLS, RUN_PRACTICE, WildHeights, climb_practice};
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::Client;

impl Client {
    /// What a wilderness step from `from` to `to` (where it landed before any
    /// wrapping round the planet) costs, by its slope and the player's
    /// Climbing, which a steep one practises: whether the client is still
    /// there, and the cost.
    pub(super) fn wild_step(
        &mut self,
        world: &ServerWorld,
        from: (i64, i64),
        to: (i64, i64),
    ) -> (bool, u32) {
        let corner = (from.0.min(to.0), from.1.min(to.1));
        let size = ((from.0 - to.0).abs() + 1, (from.1 - to.1).abs() + 1);
        let flats = self.visit.as_ref().map(|visit| visit.flats());
        let heights = WildHeights::new(&world.ground, (corner.0, corner.1, size.0, size.1))
            .with_flats(flats.unwrap_or_default());
        let ends = (heights.at(from.0, from.1), heights.at(to.0, to.1));
        let cost = wild_step_energy(from, to, ends, Gait::of(&self.sheet.skills));
        // 16ths of an elevation a cell: elevations per world map point, as
        // steepness counts there.
        let cells = (from.0 - to.0).abs().max((from.1 - to.1).abs()).max(1) as u32;
        let steepness = ends.0.abs_diff(ends.1) / cells;
        let now = world.action_clock.ticks();
        (self.practise_moving(false, steepness, now), cost)
    }

    /// Practice from a step that climbed or descended `elevations` (a world map
    /// step's, or as many 16ths a cell in the wilderness), and Running if it
    /// was `on_world_map`. The client is sent its `Sheet` only when that
    /// changes how it moves (or something rusted), as its steps are paced by
    /// it. `false` if the client has gone.
    pub(super) fn practise_moving(
        &mut self,
        on_world_map: bool,
        elevations: u32,
        now: u64,
    ) -> bool {
        let before = Gait::of(&self.sheet.skills);
        let played = self.activity.played;
        let skills = &*MOVEMENT_SKILLS;
        let mut rusted = false;
        for (skill, amount) in [
            (skills.running, if on_world_map { RUN_PRACTICE } else { 0 }),
            (skills.climbing, climb_practice(elevations)),
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
        if !rusted && Gait::of(&self.sheet.skills) == before {
            return true;
        }
        self.link.send(&ServerMessage::Sheet(self.sheet.clone()))
    }
}

#[cfg(test)]
mod tests;
