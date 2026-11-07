//! Keeping mobs from pacing back and forth: one making for the same place may
//! turn back on itself once, and then stands still until something changes.

use shared::dungeon::Level;

use super::{Maps, Mob, THINKING_COST, Taken, take_step};

/// What a mob remembers of its last steps, so it doesn't pace. Not saved.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Pacing {
    /// The cell it last stepped off.
    left: Option<(i64, i64)>,
    /// Where it is making for, if it is making for anywhere.
    aim: Option<(i64, i64)>,
    /// It has already turned back on itself on the way there.
    turned_back: bool,
    /// It made for `aim` this turn.
    aimed: bool,
}

impl Pacing {
    /// It is making for `goal` this turn. A new goal is something changing, so it
    /// may turn back again.
    pub(super) fn aim_at(&mut self, goal: (i64, i64)) {
        if self.aim != Some(goal) {
            self.aim = Some(goal);
            self.turned_back = false;
        }
        self.aimed = true;
    }

    /// Whether it may step onto this cell: not back where it has just come from,
    /// if it has turned back once already on the way to the same place.
    pub(super) fn allows(&self, to: (i64, i64)) -> bool {
        !(self.turned_back && self.left == Some(to))
    }

    /// After every turn: where it was and is now. A turn spent on anything but
    /// making for somewhere (wandering, fleeing, fighting) forgets the goal.
    pub(super) fn turn_ended(&mut self, was_at: (i64, i64), now: (i64, i64)) {
        if !self.aimed {
            self.aim = None;
            self.turned_back = false;
        }
        self.aimed = false;
        if was_at != now {
            if self.aim.is_some() && self.left == Some(now) {
                self.turned_back = true;
            }
            self.left = Some(was_at);
        }
    }
}

/// A step towards `goal`, or none if there is no way there (`None`). Rather than
/// turn back on itself a second time, it stands where it is.
pub(super) fn walk_towards(
    mob: &mut Mob,
    level: &Level,
    maps: &mut Maps,
    goal: (i64, i64),
    taken: &Taken,
) -> Option<u32> {
    mob.pacing.aim_at(goal);
    let pacing = mob.pacing;
    let free = |cell: (i64, i64)| !taken.contains(&cell);
    let map = maps.to(level, goal);
    let step = map.towards(level, mob.position, |cell| {
        free(cell) && pacing.allows(cell)
    });
    if step.is_some() {
        return take_step(mob, level, step, taken);
    }
    // Only back the way it came: it has paced enough, and waits.
    map.towards(level, mob.position, free)
        .map(|_| THINKING_COST)
}

#[cfg(test)]
mod tests;
