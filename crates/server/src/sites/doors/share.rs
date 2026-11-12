//! Telling everyone on a level when a door changes, and who stands in doorways.

use shared::door::Door;
use shared::id::Id;
use std::collections::BTreeSet;

use super::DoorKey;
use super::act::changed_at;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::update_sight;
use crate::storage::SiteId;

/// A door that has just changed: where it is kept, and how it is now.
#[derive(Clone, Copy, Debug)]
pub(crate) struct DoorChange {
    pub(crate) key: DoorKey,
    pub(crate) door: Door,
}

/// Where each player stood when doors were last shared: the site, level and
/// cell (each map a cell is kept on: a poi's cutout is its site's too), and who.
pub(crate) type Standing = BTreeSet<((SiteId, u32, (i64, i64)), Id)>;

/// Every tick, after players and mobs have acted: tells everyone whose level
/// has a door that changed, by setting it on their level (sent again if they
/// can see it), and the mobs there. Then notes where everyone stands, for
/// closing doors (`someone_in_doorway`).
pub(crate) fn share_doors(clients: &mut [Client], world: &mut ServerWorld) {
    let changes = std::mem::take(&mut world.door_changes);
    for change in &changes {
        if change.key.site != SiteId::Wilderness {
            let place = (change.key.site, change.key.index);
            if let Some(population) = world.populations.0.get_mut(&place) {
                population.set_door(change.key.at, change.door);
            }
        }
    }
    for client in clients.iter_mut() {
        let Some(visit) = &mut client.visit else {
            continue;
        };
        let mut any = false;
        for change in &changes {
            let here: Vec<(i64, i64)> = (visit.doors.iter())
                .filter(|(_, door)| door.key == change.key && door.door != change.door)
                .map(|(&at, _)| at)
                .collect();
            for at in here {
                let door = visit.doors.get_mut(&at).expect("found above");
                door.door = change.door;
                visit.level.level.set_swing(at.0, at.1, change.door.swing);
                visit
                    .level
                    .level
                    .set_locked(at.0, at.1, change.door.is_locked());
                changed_at(visit, at.0, at.1);
                any = true;
            }
        }
        if any && !update_sight(client) {
            // Gone: noticed when its messages are next read.
            continue;
        }
    }
    world.standing = standing(clients);
}

/// Where each player stands, on every map their cell is kept on.
fn standing(clients: &[Client]) -> Standing {
    let mut found = Standing::new();
    for client in clients {
        let Some(visit) = &client.visit else {
            continue;
        };
        let id = client.entity_id();
        let (x, y) = visit.position();
        found.insert(((visit.id, visit.index, (visit.kept_at(x), y)), id));
        for mirror in &visit.mirrors {
            if let Some(at) = mirror.of(x, y) {
                found.insert(((mirror.id, mirror.index, at), id));
            }
        }
    }
    found
}

impl ServerWorld {
    /// Whether anyone or anything is in the doorway of the door at `at` on
    /// `client`'s level (kept as `key`): the player, another player, a mob or a
    /// bag on the ground. Nothing is ever pushed out of the way.
    pub(crate) fn someone_in_doorway(
        &mut self,
        client: &Client,
        at: (i64, i64),
        key: DoorKey,
    ) -> bool {
        let Some(visit) = &client.visit else {
            return true;
        };
        let me = client.entity_id();
        let kept = (visit.id, visit.index, (visit.kept_at(at.0), at.1));
        let places = [(key.site, key.index, key.at), kept];
        let player = (self.standing.iter()).any(|&(place, id)| id != me && places.contains(&place));
        let mob = (self.populations.0.get(&visit.population()))
            .is_some_and(|population| population.mobs.iter().any(|mob| mob.position == at));
        let bag = places.iter().any(|&(site, index, cell)| {
            let level = self.site(site).levels.get(&index);
            // A bag anywhere in the doorway's cell.
            level.is_some_and(|level| {
                (level.ground.iter()).any(|(at, items)| at.cell() == cell && !items.is_empty())
            })
        });
        visit.position() == at || player || mob || bag
    }
}
