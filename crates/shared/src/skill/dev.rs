//! Dev tools' changes to a creature's skills (the debug menu's Learn Skills):
//! setting a node to a percentage, learning or forgetting one, clearing them all.

use super::{SkillId, Skills};

/// What learning a node by dev tools gives it, as a percentage.
pub const DEV_LEARN_PERCENT: u32 = 10;

/// The practice that gives a node this percentage: the cube (100% is 1,000,000).
/// Kept to what a `u32` holds, 1,625% at most.
pub fn practice_for(percent: u32) -> u32 {
    let percent = u64::from(percent.min(1625));
    (percent * percent * percent) as u32
}

impl Skills {
    /// This node's value as a whole percentage, rounded down.
    pub fn percent(&self, id: SkillId) -> u32 {
        self.value(id) / super::VALUE_SCALE
    }

    /// Sets this node alone to `percent` (0 forgets it, as `forget` does): its
    /// best is at least that, and the nodes above it are left as they are.
    pub fn set_percent(&mut self, id: SkillId, percent: u32) {
        let practice = practice_for(percent);
        if practice == 0 {
            self.forget(id);
            return;
        }
        let learned = self.0.entry(id).or_default();
        learned.practice = practice;
        learned.best = learned.best.max(practice);
    }

    /// Learns this node at `DEV_LEARN_PERCENT`, as if practised: the nodes above
    /// it learn their share too.
    pub fn learn(&mut self, id: SkillId) {
        self.practise(id, practice_for(DEV_LEARN_PERCENT));
    }

    /// Forgets this node and everything under it.
    pub fn forget(&mut self, id: SkillId) {
        let mut gone = vec![id];
        while let Some(node) = gone.pop() {
            self.0.remove(&node);
            gone.extend_from_slice(node.children());
        }
    }

    /// Forgets everything.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Sets every node of the tree to `percent` (0 forgets them all).
    pub fn set_all(&mut self, percent: u32) {
        for id in SkillId::all() {
            self.set_percent(id, percent);
        }
    }
}
