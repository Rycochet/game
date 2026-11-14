//! Someone a quester escorts (owner): a mob everyone on its level sees, that
//! keeps near its quester while it follows them and waits where it is
//! otherwise. Never kept with a site: `quests::escort` makes it, moves it to
//! whatever level its quester is on and takes it away, by its id.

use shared::id::Id;
use shared::mob::{Memory, Spawn};
use shared::quest::QuestId;
use shared::spot::Spot;

use super::made::ESCORT_NUMBER;
use super::notice::Spot as Someone;
use super::{Doing, FRIENDLY_GAP, Mob, dressed, nearest_free};
use crate::ServerWorld;
use crate::storage::SiteId;

/// Who a mob is escorted by (`Mob::escort`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Escortee {
    /// The quester's entity id.
    pub(crate) quester: Id,
    pub(crate) quest: QuestId,
    /// Its name, as everyone is told it.
    pub(crate) name: String,
    /// It keeps near them; otherwise it waits where it is.
    pub(crate) following: bool,
}

/// An escortee as rolled (`spawn`), dressed in what it was rolled with.
pub(crate) fn escortee_mob(world: &ServerWorld, id: Id, spawn: &Spawn, escort: Escortee) -> Mob {
    let spawn = Spawn {
        number: ESCORT_NUMBER,
        ..spawn.clone()
    };
    let carrying = dressed(spawn.kind, &spawn.gear, 0, world.made_now());
    Mob {
        memory: Memory::default(),
        escort: Some(escort),
        ..Mob::new(id, &spawn, &carrying, 0)
    }
}

/// What can be seen of what an escortee rolled as `spawn` wears.
pub(crate) fn escortee_worn(world: &ServerWorld, spawn: &Spawn) -> Vec<shared::wear::WornLook> {
    dressed(spawn.kind, &spawn.gear, 0, world.made_now()).shown()
}

/// What an escortee does now: going after its quester while it follows them
/// and they are on its level, else nothing. Whether it was at rest before,
/// so needs a turn.
pub(super) fn mind_escort(mob: &mut Mob, spots: &[Someone]) -> bool {
    let Some(escort) = &mob.escort else {
        return false;
    };
    let quester = spots.iter().find(|spot| spot.id == escort.quester);
    let was_idle = mob.doing == Doing::Idle;
    mob.doing = match quester {
        Some(quester) if escort.following => Doing::Going {
            after: quester.id,
            last_seen: quester.position,
            gap: FRIENDLY_GAP,
        },
        _ => Doing::Idle,
    };
    mob.target = None;
    was_idle && mob.doing != Doing::Idle
}

impl ServerWorld {
    /// Where an escortee is: its level and spot, if it is on one in use.
    pub(crate) fn escortee_at(&self, id: Id) -> Option<((SiteId, u32), Spot)> {
        self.populations.0.iter().find_map(|(&place, population)| {
            let mob = population.mobs.iter().find(|mob| mob.id == id)?;
            Some((place, mob.spot))
        })
    }

    /// Takes an escortee off whatever level it is on (it leaves with its
    /// quester, or goes home). Nothing is left of it there.
    pub(crate) fn take_escortee(&mut self, id: Id) -> Option<Mob> {
        for population in self.populations.0.values_mut() {
            if let Some(index) = population.mobs.iter().position(|mob| mob.id == id) {
                population.due.retain(|&(_, due)| due != id);
                return Some(population.mobs.remove(index));
            }
        }
        None
    }

    /// Puts an escortee on level `place`, at `spot` if it can stand there,
    /// else on the nearest free cell. Whether there was room.
    pub(crate) fn put_escortee(&mut self, place: (SiteId, u32), mut mob: Mob, spot: Spot) -> bool {
        let tick = self.action_clock.ticks();
        let Some(population) = self.populations.0.get_mut(&place) else {
            return false;
        };
        let taken = |cell: (i64, i64)| population.mobs.iter().any(|mob| mob.position == cell);
        let cell = spot.cell();
        let free =
            shared::movement::mob_can_stand(&population.level, cell.0, cell.1) && !taken(cell);
        if free {
            mob.place(spot);
        } else if let Some(cell) = nearest_free(&population.level, cell, taken) {
            mob.put(cell);
        } else {
            return false;
        }
        mob.home = mob.position;
        population.due.insert((tick, mob.id));
        population.mobs.push(mob);
        true
    }

    /// Whether an escortee keeps near its quester from now on.
    pub(crate) fn set_following(&mut self, id: Id, following: bool) {
        let mobs = self.populations.0.values_mut().flat_map(|p| &mut p.mobs);
        for mob in mobs.filter(|mob| mob.id == id) {
            if let Some(escort) = &mut mob.escort {
                escort.following = following;
            }
        }
    }
}
