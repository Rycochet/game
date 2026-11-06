//! `Skills`: how much one creature has practised each node, and the values that
//! gives. Practice flows up the tree, halving at each level; a node's value is the
//! square root of its practice, and a check adds the values along the path, so
//! every level counts (`docs/context/abilities-skills.md`).

use std::collections::BTreeMap;

use super::SkillId;
use crate::creature::CreatureKind;

/// Values are in hundredths: 1,000 practice is a value of 31.62, kept as 3162.
pub const VALUE_SCALE: u32 = 100;

/// The practice an old skill level stood for (saves from before the tree, and the
/// kinds' starting skills): its square times 100, so level 3 is a value of 30.
pub const fn level_practice(level: u32) -> u32 {
    level * level * 100
}

/// Practice per node: only nodes with some are kept, so a node is known once it
/// has any. Integers only, as client and server must agree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Skills(pub(super) BTreeMap<SkillId, u32>);

impl Skills {
    /// What a new creature of this kind starts knowing.
    pub fn starting(kind: CreatureKind) -> Self {
        let mut skills = Self::default();
        for &(path, practice) in kind.def().skills {
            if let Some(id) = SkillId::find(path) {
                skills.practise(id, practice);
            }
        }
        skills
    }

    /// How much this node has been practised.
    pub fn practice(&self, id: SkillId) -> u32 {
        self.0.get(&id).copied().unwrap_or(0)
    }

    /// Whether it has been practised at all.
    pub fn knows(&self, id: SkillId) -> bool {
        self.practice(id) > 0
    }

    /// Every known node and its practice, parents before their children.
    pub fn known(&self) -> impl Iterator<Item = (SkillId, u32)> + '_ {
        self.0.iter().map(|(&id, &practice)| (id, practice))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Gives this node `amount`, its parent half that, the next a quarter, and so
    /// on until it rounds down to nothing or reaches the top.
    pub fn practise(&mut self, id: SkillId, amount: u32) {
        let mut amount = amount;
        for node in id.upwards() {
            if amount == 0 {
                break;
            }
            let practice = self.0.entry(node).or_insert(0);
            *practice = practice.saturating_add(amount);
            amount /= 2;
        }
    }

    /// This node's own value, in hundredths (`VALUE_SCALE`): the square root of its
    /// practice.
    pub fn value(&self, id: SkillId) -> u32 {
        let scaled = self.practice(id) as u64 * (VALUE_SCALE * VALUE_SCALE) as u64;
        scaled.isqrt() as u32
    }

    /// What a check with this node counts, in hundredths: its value and every value
    /// above it, up to its top level.
    pub fn total(&self, id: SkillId) -> u32 {
        id.upwards().map(|node| self.value(node)).sum()
    }
}
