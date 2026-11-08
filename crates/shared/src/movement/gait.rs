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

/// How long wading or swimming takes against walking, per mille, at 0%: an
/// average swimmer's 50 m a minute against walking's 85 (owner, 2026-11-08).
pub const SWIM_SLOW_PER_MILLE: u32 = 1700;

/// The same at 100%: a record's 100 m in 50 seconds, 120 m a minute, quicker
/// than walking.
pub const SWIM_FAST_PER_MILLE: u32 = 708;

/// Swimming from which deep water can be swum: water on a level, and the sea
/// near the coast on the world map (50%).
pub const DEEP_WATER_FROM: u32 = 5_000;

/// How deep a sea 100% Swimming can cross on the world map, in the surface's
/// height units (100 m): the continental shelf, never the deep ocean beyond.
pub const DEEPEST_SEA: f64 = 2.0;

/// How long a jump takes at 100% Jumping, per mille of its distance (all of it
/// at 0%): a practised leap carries further for the effort. A setting to tune.
pub const JUMP_BEST_PER_MILLE: u32 = 700;

/// The movement skills' nodes, found once (`MOVEMENT_SKILLS`).
pub struct MovementSkills {
    pub climbing: SkillId,
    pub running: SkillId,
    pub weight_training: SkillId,
    pub swimming: SkillId,
    pub jumping: SkillId,
}

/// Where Climbing and Running are in the tree.
pub static MOVEMENT_SKILLS: LazyLock<MovementSkills> = LazyLock::new(|| MovementSkills {
    climbing: SkillId::find("athletics/climbing").expect("Climbing is in the tree"),
    running: SkillId::find("athletics/running").expect("Running is in the tree"),
    weight_training: SkillId::find("athletics/weighttraining")
        .expect("Weight Training is in the tree"),
    swimming: SkillId::find("athletics/swimming").expect("Swimming is in the tree"),
    jumping: SkillId::find("athletics/jumping").expect("Jumping is in the tree"),
});

/// How well a creature climbs, runs, swims and jumps: each skill's value up to
/// `FULL`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Gait {
    climbing: u32,
    running: u32,
    swimming: u32,
    jumping: u32,
}

impl Gait {
    /// No skill at all: what roads are laid by, and anyone untrained.
    pub const UNSKILLED: Self = Self {
        climbing: 0,
        running: 0,
        swimming: 0,
        jumping: 0,
    };

    /// From a creature's skills.
    pub fn of(skills: &Skills) -> Self {
        Self {
            climbing: skills.value(MOVEMENT_SKILLS.climbing).min(FULL),
            running: skills.value(MOVEMENT_SKILLS.running).min(FULL),
            swimming: skills.value(MOVEMENT_SKILLS.swimming).min(FULL),
            jumping: skills.value(MOVEMENT_SKILLS.jumping).min(FULL),
        }
    }

    /// How long wading takes against walking, per mille: from
    /// `SWIM_SLOW_PER_MILLE` at 0% to `SWIM_FAST_PER_MILLE` at 100%.
    pub fn swim_per_mille(self) -> u32 {
        let gained = (SWIM_SLOW_PER_MILLE - SWIM_FAST_PER_MILLE) * self.swimming / FULL;
        SWIM_SLOW_PER_MILLE - gained
    }

    /// Whether deep water can be swum (`DEEP_WATER_FROM`).
    pub fn swims_deep(self) -> bool {
        self.swimming >= DEEP_WATER_FROM
    }

    /// How deep a sea can be crossed on the world map, in height units: none
    /// below `DEEP_WATER_FROM`, then up to `DEEPEST_SEA` at 100%.
    pub fn deepest_sea(self) -> f64 {
        if !self.swims_deep() {
            return -1.0;
        }
        let past = f64::from(self.swimming - DEEP_WATER_FROM);
        DEEPEST_SEA * past / f64::from(FULL - DEEP_WATER_FROM)
    }

    /// How long a jump takes, per mille of its distance: from all of it at 0%
    /// to `JUMP_BEST_PER_MILLE` at 100%.
    pub fn jump_per_mille(self) -> u32 {
        1000 - (1000 - JUMP_BEST_PER_MILLE) * self.jumping / FULL
    }

    /// What a jump costing `cost` comes to with this Jumping. At least 1.
    pub fn jump(self, cost: u32) -> u32 {
        ((u64::from(cost) * u64::from(self.jump_per_mille()) / 1000) as u32).max(1)
    }

    /// The least any level step can cost against `step_cost`, per mille: what
    /// routes' estimates are scaled by, as a good swimmer or jumper can go
    /// quicker than walking. Never more than 1000.
    pub fn cheapest_per_mille(self) -> u32 {
        self.swim_per_mille().min(self.jump_per_mille()).min(1000)
    }

    /// What a step costing `cost` on land comes to, waded. At least 1.
    pub fn swim(self, cost: u32) -> u32 {
        ((u64::from(cost) * u64::from(self.swim_per_mille()) / 1000) as u32).max(1)
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
