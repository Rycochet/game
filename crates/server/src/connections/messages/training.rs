//! Practice from moving: Running for every step on the world map, Climbing for
//! steep ones (`shared::movement`'s training settings), at most once per skill
//! every 5 seconds, as blows are.

use shared::movement::{Gait, MOVEMENT_SKILLS, RUN_PRACTICE, climb_practice};
use shared::protocol::ServerMessage;

use crate::connections::Client;

impl Client {
    /// Practice from a world map step that climbed or descended `elevations`.
    /// The client is sent its `Sheet` only when that changes how it moves (or
    /// something rusted), as its steps are paced by it. `false` if the client
    /// has gone.
    pub(super) fn practise_moving(&mut self, elevations: u32, now: u64) -> bool {
        let before = Gait::of(&self.sheet.skills);
        let played = self.activity.played;
        let skills = &*MOVEMENT_SKILLS;
        let mut rusted = false;
        for (skill, amount) in [
            (skills.running, RUN_PRACTICE),
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
