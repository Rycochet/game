//! Who has harmed a mob, and when last, so a kill goes to everyone who helped
//! (owner, 2026-11-07). Kept in memory only, and forgotten once the mob gives up
//! fighting (`Doing::gave_up`).

use shared::alignment::{Alignment, Deed};
use shared::creature::CreatureKind;
use shared::id::Id;

use crate::ServerWorld;
use crate::connections::Client;
use crate::storage::SiteId;

/// How long after their last blow someone still shares in a kill: 2 minutes, in
/// action ticks (20 a second). Long enough to have been part of it, short enough
/// that leaving and letting another finish it off doesn't count.
pub(crate) const KILL_CREDIT_TICKS: u64 = 2 * 60 * 20;

/// Each who harmed a mob, and the action tick they last did.
#[derive(Clone, Debug, Default)]
pub(crate) struct HarmedBy(Vec<(Id, u64)>);

impl HarmedBy {
    /// They harmed it at `now`, and anyone whose blows are too old is forgotten.
    pub fn harmed(&mut self, who: Id, now: u64) {
        self.0
            .retain(|&(id, at)| id != who && at + KILL_CREDIT_TICKS > now);
        self.0.push((who, now));
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Who shares in its death at `now`: everyone who harmed it recently enough.
    pub fn credited(&self, now: u64) -> impl Iterator<Item = Id> + '_ {
        self.0
            .iter()
            .filter(move |&&(_, at)| at + KILL_CREDIT_TICKS > now)
            .map(|&(id, _)| id)
    }
}

/// A mob killed, for the players who helped to be told (`credit_kills`).
pub(crate) struct Kill {
    /// The level it died on.
    pub place: (SiteId, u32),
    pub kind: CreatureKind,
    pub by: HarmedBy,
    /// Its alignment, if it had one (animals don't).
    pub alignment: Option<Alignment>,
    /// Those of `by` for whom it was self-defence (`Memory::provoked`).
    pub provoked: Vec<Id>,
}

impl Kill {
    /// What it was for this one of its killers, if it moves their alignment.
    fn deed_for(&self, who: Id) -> Option<Deed> {
        Some(Deed::Killed {
            victim: self.alignment?,
            provoked: self.provoked.contains(&who),
        })
    }
}

/// Every tick, after telling who died: each kill counts, in `Sheet::foes`, for
/// everyone who harmed the mob in the last `KILL_CREDIT_TICKS` and is still on
/// its level (so not if they were killed, or left), and moves their alignment
/// (`Deed::Killed`): down for a Neutral or Good victim, unless in self-defence.
pub(super) fn credit_kills(clients: &mut [Client], world: &mut ServerWorld) {
    let now = world.action_clock.ticks();
    for kill in std::mem::take(&mut world.kills) {
        for who in kill.by.credited(now) {
            let Some(client) = clients.iter_mut().find(|c| c.entity_id() == who) else {
                continue;
            };
            let here = client.visit.as_ref().map(|visit| visit.population());
            if here == Some(kill.place) {
                client.sheet.foes.defeated(kill.kind);
                crate::quests::killed(client, world, kill.kind, kill.place.0);
                if let (Some(deed), Some(alignment)) =
                    (kill.deed_for(who), client.sheet.alignment.as_mut())
                {
                    *alignment = alignment.after(deed);
                }
                client.unsaved = true;
            }
        }
    }
}

#[cfg(test)]
mod tests;
