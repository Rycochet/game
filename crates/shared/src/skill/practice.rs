//! `Skills`: how much one creature has practised each node, and the values that
//! gives. Practice flows up the tree, halving at each level; a node's value is the
//! cube root of its practice, and a check adds the values along the path, so
//! every level counts (`docs/context/abilities-skills.md`).

use std::collections::BTreeMap;

use super::SkillId;
use super::learned::Learned;
use crate::creature::CreatureKind;

/// Values are in hundredths: 1,000 practice is a value of 10.00, kept as 1000, and
/// 100% takes 1,000,000 (owner, 2026-11-07: square roots grew far too fast).
pub const VALUE_SCALE: u32 = 100;

/// The practice an old skill level stood for (saves from before the tree, and the
/// kinds' starting skills): its cube times 1,000, so level 3 is a value of 30.
pub const fn level_practice(level: u32) -> u32 {
    level * level * level * 1000
}

/// Practice per node, and what rust goes by: only nodes with some are kept, so a
/// node is known once it has any. Integers only, as client and server must agree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Skills(pub(super) BTreeMap<SkillId, Learned>);

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
        self.0.get(&id).map_or(0, |learned| learned.practice)
    }

    /// Whether it has been practised at all.
    pub fn knows(&self, id: SkillId) -> bool {
        self.practice(id) > 0
    }

    /// Every known node and its practice, parents before their children.
    pub fn known(&self) -> impl Iterator<Item = (SkillId, u32)> + '_ {
        self.0.iter().map(|(&id, learned)| (id, learned.practice))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The most practice this node has ever had: rust stops at a share of it.
    pub fn best(&self, id: SkillId) -> u32 {
        self.0.get(&id).map_or(0, |learned| learned.best)
    }

    /// Gives this node `amount`, its parent half that, the next a quarter, and so
    /// on until it rounds down to nothing or reaches the top. Practice below the
    /// best a node reached counts double. Using it, for rust, is `used`.
    pub fn practise(&mut self, id: SkillId, amount: u32) {
        let mut amount = amount;
        for node in id.upwards() {
            if amount == 0 {
                break;
            }
            self.0.entry(node).or_default().gain(amount);
            amount /= 2;
        }
    }

    /// This node's own value, in hundredths (`VALUE_SCALE`): the cube root of its
    /// practice.
    pub fn value(&self, id: SkillId) -> u32 {
        let scale = u64::from(VALUE_SCALE);
        cube_root(u64::from(self.practice(id)) * scale * scale * scale) as u32
    }

    /// What a check with this node counts, in hundredths: its value and every value
    /// above it, up to its top level.
    pub fn total(&self, id: SkillId) -> u32 {
        id.upwards().map(|node| self.value(node)).sum()
    }
}

/// The whole number cube root, rounded down: no floats, so client and server agree.
fn cube_root(n: u64) -> u64 {
    // The largest cube that fits a `u64` is 2,642,245 cubed.
    let (mut low, mut high) = (0u64, 2_642_245u64);
    while low < high {
        let middle = (low + high).div_ceil(2);
        if middle * middle * middle <= n {
            low = middle;
        } else {
            high = middle - 1;
        }
    }
    low
}
