//! Kick: a blow with a foot, by anyone with legs. Slower than a punch and weaker
//! against creatures, as it shoves them a step away; better against things, as
//! doors are broken by it. First guesses, to balance by playing.

use crate::rng::Rng;
use crate::skill::SkillId;
use crate::vitals::Points;

use super::{Blow, Fighter, Swing, Weapon, attack_cost, power, roll_blow, skill_percent, wind_up};

/// A kick takes this much longer than an unarmed attack, per mille: more than twice.
pub const KICK_COST_PER_MILLE: u32 = 2500;

/// What a kick does to a creature it shoves away, per mille of an unarmed blow;
/// one that can't be shoved takes twice that, still less than a punch.
pub const KICK_HARM_PER_MILLE: u64 = 400;

/// What a kick does to a thing (a door), against an unarmed blow with nothing
/// defending: it is made for it.
pub const KICK_FORCE_PER_MILLE: u64 = 2000;

/// The Kick skill.
pub fn kick_skill() -> Option<SkillId> {
    SkillId::find("combat/melee/kick")
}

impl Fighter<'_> {
    fn kick_power(&self) -> u64 {
        let skill = kick_skill();
        let total = self.skills.zip(skill).map_or(0, |(s, id)| s.total(id));
        power(self.attributes, skill.map_or(&[], SkillId::leans), total)
    }

    /// A kick: an unarmed attack's cost, by agility and the Kick skill, made
    /// `KICK_COST_PER_MILLE` as long.
    pub fn kick(&self) -> Swing {
        let value = self
            .skills
            .zip(kick_skill())
            .map_or(0, |(s, id)| s.value(id));
        let cost = attack_cost(
            &Weapon::UNARMED,
            self.attributes.agility,
            skill_percent(value),
            None,
        );
        let cost = cost * KICK_COST_PER_MILLE / 1000;
        Swing {
            weapon: Weapon::UNARMED,
            off_hand: false,
            cost,
            wind_up: wind_up(cost),
        }
    }

    /// What a kick does to `defender`: less if it shoves them away (`shoved`),
    /// twice that if it can't.
    pub fn kick_harm(&self, defender: &Fighter, shoved: bool, rng: &mut Rng) -> Points {
        let share = if shoved {
            KICK_HARM_PER_MILLE
        } else {
            2 * KICK_HARM_PER_MILLE
        };
        let harm = self.kick_blow(defender.defence_power(), rng).harm();
        Points((u64::from(harm.0) * share / 1000) as u32)
    }

    /// What a kick does to a thing, such as a door, in points of its toughness.
    pub fn kick_force(&self, rng: &mut Rng) -> Points {
        let harm = self.kick_blow(0, rng).harm();
        Points((u64::from(harm.0) * KICK_FORCE_PER_MILLE / 1000) as u32)
    }

    fn kick_blow(&self, defence: u64, rng: &mut Rng) -> Blow {
        Blow {
            weapon: Weapon::UNARMED,
            strength: self.attributes.strength,
            luck: roll_blow(rng),
            hand: 1000,
            attack: self.kick_power(),
            defence,
            armour: 0,
        }
    }
}
