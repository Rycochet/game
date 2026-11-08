//! How well a creature gets about, from its movement skills: what Climbing and
//! Running take off world map steps. Worked out from the skills on both sides,
//! so the client paces steps as the server charges them.

use std::sync::LazyLock;

use crate::skill::{SkillId, Skills};

/// A skill's value at 100%, in hundredths (`skill::VALUE_SCALE`): the effects
/// below are full there and go no further, specialised or not.
pub const FULL: u32 = 10_000;

/// What 100% Climbing takes off the slope's cost, per mille: routes go almost
/// straight over mountains, but a climb never comes quite free (owner).
pub const CLIMB_SAVING_PER_MILLE: u32 = 900;

/// How much faster 100% Running goes on the world map than walking, in tenths:
/// a marathon record's pace (about 350 m a minute) against walking's 85 (owner,
/// 2026-11-08).
pub const RUN_TENTHS: u32 = 41;

/// The movement skills' nodes, found once (`MOVEMENT_SKILLS`).
pub struct MovementSkills {
    pub climbing: SkillId,
    pub running: SkillId,
}

/// Where Climbing and Running are in the tree.
pub static MOVEMENT_SKILLS: LazyLock<MovementSkills> = LazyLock::new(|| MovementSkills {
    climbing: SkillId::find("athletics/climbing").expect("Climbing is in the tree"),
    running: SkillId::find("athletics/running").expect("Running is in the tree"),
});

/// How well a creature climbs and runs: each skill's value up to `FULL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gait {
    climbing: u32,
    running: u32,
}

impl Gait {
    /// No skill at all: what roads are laid by, and anyone untrained.
    pub const UNSKILLED: Self = Self {
        climbing: 0,
        running: 0,
    };

    /// From a creature's skills.
    pub fn of(skills: &Skills) -> Self {
        Self {
            climbing: skills.value(MOVEMENT_SKILLS.climbing).min(FULL),
            running: skills.value(MOVEMENT_SKILLS.running).min(FULL),
        }
    }

    /// What a slope costing `slope_cost` comes to with this Climbing: down to a
    /// tenth of it at 100%.
    pub fn climb(self, slope_cost: u32) -> u32 {
        let saved = u64::from(CLIMB_SAVING_PER_MILLE) * u64::from(self.climbing);
        let kept = u64::from(FULL) * 1000 - saved;
        (u64::from(slope_cost) * kept / (u64::from(FULL) * 1000)) as u32
    }

    /// How long a world map step costing `cost` takes with this Running: from
    /// walking at 0% to `RUN_TENTHS` times as fast at 100%. At least 1.
    pub fn run(self, cost: u32) -> u32 {
        let faster = u64::from(RUN_TENTHS - 10) * u64::from(self.running);
        let divisor = u64::from(FULL) * 10 + faster;
        ((u64::from(cost) * u64::from(FULL) * 10 / divisor) as u32).max(1)
    }
}

#[cfg(test)]
mod tests;
