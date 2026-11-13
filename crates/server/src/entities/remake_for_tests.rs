//! Making one of a level's mobs over, for tests that need a mob of their own
//! kind in their own place.

use shared::creature::CreatureKind;
use shared::item::Inventory;
use shared::sheet::Sheet;

use super::{Doing, Mob, Population};

impl Population {
    /// Mob `index` made over as a fresh `kind` at rest at `cell` (its home
    /// too), bare handed and as quick as its kind, for tests: nothing of what
    /// it was made with or did before, which can differ from run to run (its
    /// gear is rolled anew, and how long the level ran first varies).
    pub(crate) fn remake_for_test(
        &mut self,
        index: usize,
        kind: CreatureKind,
        cell: (i64, i64),
    ) -> &mut Mob {
        let mob = &mut self.mobs[index];
        let empty = Inventory::default();
        mob.kind = kind;
        mob.put(cell);
        mob.home = cell;
        mob.doing = Doing::Idle;
        mob.target = None;
        mob.pacing = Default::default();
        mob.energy = Default::default();
        mob.wielded = shared::combat::Wielded::of(&empty);
        mob.speed = Sheet::new(kind).speed(&empty);
        let id = mob.id;
        self.due.retain(|&(_, due)| due != id);
        &mut self.mobs[index]
    }
}
