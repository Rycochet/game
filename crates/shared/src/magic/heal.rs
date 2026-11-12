//! Heal: Mana turned into Health, more for each point of Mana the better the
//! caster's Healing skill (owner, 2026-11-07). Numbers are first guesses, to
//! balance by playing.

use crate::combat::{Body, skill_percent, within};
use crate::dungeon::Level;
use crate::skill::{SkillId, Skills};
use crate::vitals::{PER_POINT, Points};

/// The skill casting it practises, and whose value sets how well it heals.
pub const HEAL_SKILL: &str = "magic/divine/healing";
/// Mana each cast spends, in whole points, however much of the healing is
/// needed: whatever is beyond the target's most is lost (owner).
pub const HEAL_MANA: u32 = 6;
/// Health for each point of Mana, per mille: 2 with no skill...
pub const RATIO_AT_FIRST: u64 = 2000;
/// ...rising in a straight line to 20 at 100%.
pub const RATIO_AT_BEST: u64 = 20_000;
/// Energy a cast takes: a second at normal speed.
pub const HEAL_COST: u32 = 4200;
/// How much of that is waited before it lands: half.
pub const HEAL_WIND_UP: u32 = HEAL_COST / 2;
/// How far away another creature can be healed, between middles in 3D, with
/// a clear line: 6 cells, in units (240 a cell).
pub const HEAL_RANGE: u64 = 6 * crate::spot::CELL_SPAN as u64;
/// An automatic Heal is cast at or below this much Health, per mille: a quarter.
pub const AUTO_HEAL_BELOW: u64 = 250;
/// Practice each cast gives Healing (halved each level up, as for blows).
pub const PRACTICE_PER_CAST: u32 = 8;

/// The Healing node.
pub fn heal_skill() -> SkillId {
    SkillId::find(HEAL_SKILL).expect("the skill tree has Healing")
}

/// Health for each point of Mana, per mille, by the caster's Healing skill
/// (`None` for a creature without skills, counting as none).
pub fn heal_ratio(skills: Option<&Skills>) -> u64 {
    let percent = skills.map_or(0, |skills| skill_percent(skills.value(heal_skill())));
    RATIO_AT_FIRST + (RATIO_AT_BEST - RATIO_AT_FIRST) * percent / 100
}

/// How much Health one cast gives.
pub fn heal_amount(skills: Option<&Skills>) -> Points {
    let mana = u64::from(HEAL_MANA) * u64::from(PER_POINT);
    Points((mana * heal_ratio(skills) / 1000) as u32)
}

/// Whether a caster `from` can heal `to`: themselves (the same place), or
/// within `HEAL_RANGE` with a clear line.
pub fn heal_reaches(level: &Level, from: &Body, to: &Body) -> bool {
    from.at == to.at || within(level, from, to, HEAL_RANGE)
}

/// Whether this much Health out of `most` (whole points) is low enough for an
/// automatic Heal.
pub fn needs_healing(health: Points, most: u32) -> bool {
    u64::from(health.milli()) * 1000 <= u64::from(most) * u64::from(PER_POINT) * AUTO_HEAL_BELOW
}
