//! `Fighter`: one side of a fight as the rules see it, for working out each
//! attack's cost and harm the same way for players and mobs.

use crate::creature::CreatureKind;
use crate::rng::Rng;
use crate::sheet::Attributes;
use crate::skill::{SkillId, Skills};
use crate::vitals::Points;

use super::{
    Blow, Weapon, Wielded, attack_cost, off_hand_share, power, roll_blow, skill_percent, wind_up,
};

/// How much practice each blow gives its skill: halved at each level up the
/// tree, so every level of a weapon's path (four under Combat) gains some. A
/// first guess, to balance by playing.
pub const PRACTICE_PER_BLOW: u32 = 8;

/// One side of a fight: what it is, how able, how skilled (mobs have no skills
/// yet, so count from the floor) and what it holds.
pub struct Fighter<'a> {
    pub kind: CreatureKind,
    pub attributes: &'a Attributes,
    pub skills: Option<&'a Skills>,
    pub wielded: Wielded,
}

/// One attack: what it is made with and what it costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swing {
    pub weapon: Weapon,
    /// With the off hand, when dual wielding.
    pub off_hand: bool,
    pub cost: u32,
    /// The part of `cost` waited before it lands.
    pub wind_up: u32,
}

fn dual_skill() -> Option<SkillId> {
    SkillId::find("combat/melee/dual")
}

fn leans(skill: Option<SkillId>) -> &'static [crate::sheet::Attribute] {
    skill.map_or(&[], SkillId::leans)
}

impl Fighter<'_> {
    fn value(&self, skill: Option<SkillId>) -> u32 {
        self.skills.zip(skill).map_or(0, |(s, id)| s.value(id))
    }

    fn total(&self, skill: Option<SkillId>) -> u32 {
        self.skills.zip(skill).map_or(0, |(s, id)| s.total(id))
    }

    fn dual_percent(&self) -> Option<u64> {
        let dual = skill_percent(self.value(dual_skill()));
        self.wielded.is_dual().then_some(dual)
    }

    /// Its `n`th attack in a row: hands take turns when dual wielding.
    pub fn swing(&self, n: u64) -> Swing {
        let (weapon, off_hand) = self.wielded.for_attack(n);
        let skill = skill_percent(self.value(weapon.attack_skill(self.kind)));
        let agility = self.attributes.agility;
        let cost = attack_cost(&weapon, agility, skill, self.dual_percent());
        Swing {
            weapon,
            off_hand,
            cost,
            wind_up: wind_up(cost),
        }
    }

    /// How good it is at attacking with this weapon.
    pub fn attack_power(&self, weapon: &Weapon) -> u64 {
        let skill = weapon.attack_skill(self.kind);
        power(self.attributes, leans(skill), self.total(skill))
    }

    /// How good it is at defending, with what is in its main hand.
    pub fn defence_power(&self) -> u64 {
        let skill = self.wielded.main.defence_skill();
        power(self.attributes, leans(skill), self.total(skill))
    }

    /// What its swing does to `defender`, with luck rolled from `rng`. Armour
    /// counts for nothing yet.
    pub fn harm(&self, swing: &Swing, defender: &Fighter, rng: &mut Rng) -> Points {
        let hand = match (swing.off_hand, self.dual_percent()) {
            (true, Some(dual)) => off_hand_share(dual),
            _ => 1000,
        };
        Blow {
            weapon: swing.weapon,
            strength: self.attributes.strength,
            luck: roll_blow(rng),
            hand,
            attack: self.attack_power(&swing.weapon),
            defence: defender.defence_power(),
            armour: 0,
        }
        .harm()
    }

    /// The skills a blow that lands practises: the weapon's, and dual wielding's
    /// with a weapon in each hand.
    pub fn practised_by(&self, swing: &Swing) -> Vec<SkillId> {
        let dual = dual_skill().filter(|_| self.wielded.is_dual());
        let weapon = swing.weapon.attack_skill(self.kind);
        weapon.into_iter().chain(dual).collect()
    }

    /// The skill being attacked practises: defending with what is held.
    pub fn practised_defending(&self) -> Option<SkillId> {
        self.wielded.main.defence_skill()
    }
}
