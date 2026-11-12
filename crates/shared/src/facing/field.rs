//! How wide a creature sees, from its kind and its Observation.

use crate::creature::CreatureKind;
use crate::skill::{SkillId, VALUE_SCALE};
use crate::spot::CELL_SPAN;

use super::heading::TURN;

/// The circle all round that anything is seen in, whichever way it faces: a
/// cell, in units.
pub const NEAR: u64 = CELL_SPAN as u64;

/// Observation's value at 100% (`Skills::value`).
const FULL_SKILL: u64 = 100 * VALUE_SCALE as u64;

/// Degrees Observation adds to the cone's width at 100%.
const WIDER_AT_FULL: u64 = 40;

/// Cells Observation adds to how far one sees, where that has a limit, at 100%.
const FURTHER_AT_FULL: u64 = 3;

/// The Observation skill (`survival/observation`): widens the cone and
/// lengthens sight a little, and seeing with it practises it.
pub fn observation_skill() -> SkillId {
    SkillId::find("survival/observation").expect("survival/observation is in the tree")
}

/// What a creature sees round it: the cone's width and the near circle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Field {
    /// Half the cone's width, in heading steps (at most half a turn: all round).
    pub half: u16,
    /// The near circle's radius, in units.
    pub near: u64,
    /// Cells added to sight's reach, where it has one.
    pub further: u32,
}

impl Field {
    /// Seeing all round, as on the world map.
    pub const ALL_ROUND: Self = Self {
        half: (TURN / 2) as u16,
        near: NEAR,
        further: 0,
    };

    /// A creature of `kind` with Observation's value `observation`
    /// (`Skills::value`; 0 for mobs, which have no skills yet).
    pub fn of(kind: CreatureKind, observation: u32) -> Self {
        let skill = u64::from(observation).min(FULL_SKILL);
        let degrees = u64::from(kind.field_of_view()) + WIDER_AT_FULL * skill / FULL_SKILL;
        let half = (degrees.min(360) * u64::from(TURN) / 720).min(u64::from(TURN / 2));
        Self {
            half: half as u16,
            near: NEAR,
            further: (FURTHER_AT_FULL * skill / FULL_SKILL) as u32,
        }
    }

    /// Whether it sees all round anyway.
    pub fn is_all_round(self) -> bool {
        u32::from(self.half) >= TURN / 2
    }
}
