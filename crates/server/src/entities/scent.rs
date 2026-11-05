//! Scent marks on a level's cells: who left each cell, which way, when, and how
//! strongly. Everything that walks leaves them, players and mobs alike (not
//! anyone in god mode); noses follow them (`tracking.rs`). Kept only on the
//! server, and saved with the level.

use serde::{Deserialize, Serialize};
use shared::creature::Acuity;
use shared::id::Id;
use shared::mob::{SCENT_KEPT_FOR, can_smell};
use std::collections::BTreeMap;

/// Which way someone went on leaving a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Went {
    /// To the cell this far away: a step, or a jump of two.
    Towards(i8, i8),
    /// Up or down these stairs, to that level and cell.
    Stairs { to_level: u32, arrives: (i64, i64) },
}

/// One creature's latest mark on a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// The action tick it was left at.
    pub laid: u64,
    /// How strong, as a percentage (`shared::mob::scent_strength`).
    pub strength: u32,
    pub went: Went,
}

/// The marks on a level, by cell, then by who left them. Each creature has only
/// its latest mark on a cell, so a level holds at most a mark per creature per
/// cell, however long they walk about.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scents(BTreeMap<(i64, i64), BTreeMap<Id, Mark>>);

impl Scents {
    /// Someone left this cell, going this way.
    pub(super) fn lay(&mut self, who: Id, cell: (i64, i64), went: Went, strength: u32, now: u64) {
        let mark = Mark {
            laid: now,
            strength,
            went,
        };
        self.0.entry(cell).or_default().insert(who, mark);
    }

    /// Someone left `from` for `to` by walking: marks it if it was a step or jump.
    pub(super) fn walked(
        &mut self,
        who: Id,
        from: (i64, i64),
        to: (i64, i64),
        strength: u32,
        now: u64,
    ) {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        if from != to && dx.abs() <= 2 && dy.abs() <= 2 {
            self.lay(who, from, Went::Towards(dx as i8, dy as i8), strength, now);
        }
    }

    /// Throws away marks no nose could smell any more.
    pub(crate) fn sweep(&mut self, now: u64) {
        self.0.retain(|_, marks| {
            marks.retain(|_, mark| now.saturating_sub(mark.laid) < SCENT_KEPT_FOR);
            !marks.is_empty()
        });
    }

    /// Someone's mark on this cell, if a nose this good can still smell it.
    pub(super) fn smelt(&self, cell: (i64, i64), who: Id, smell: Acuity, now: u64) -> Option<Mark> {
        let mark = self.0.get(&cell)?.get(&who)?;
        can_smell(smell, mark.strength, mark.laid, now).then_some(*mark)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
