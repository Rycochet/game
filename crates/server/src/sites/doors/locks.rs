//! Locks put on a site level's doors the first time it is used, each key given
//! to one of the level's mobs that can be reached without going through any
//! locked door (`shared::door::place_locks`). Kept in its save, so it happens
//! once, whoever comes first and from wherever.

use shared::door::{Lock, Swing, place_locks};
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::ItemForm;
use shared::mob::spawns;
use shared::poi::Poi;
use shared::site::{Exit, SiteLevel};

use super::{DoorSite, set_doors};
use crate::ServerWorld;
use crate::ids::new_id;
use crate::storage::SiteId;

/// Locks some of level `index`'s doors, if that hasn't been done for it yet,
/// and gives their keys to its mobs.
pub(in crate::sites) fn ensure_locks(
    world: &mut ServerWorld,
    (id, poi): (SiteId, &Poi),
    index: u32,
    site_level: &SiteLevel,
) {
    let saved = world.site(id).levels.get(&index);
    if saved.is_some_and(|level| level.locks_placed) {
        return;
    }
    world
        .site_mut(id)
        .levels
        .entry(index)
        .or_default()
        .locks_placed = true;
    let mut level = site_level.level.clone();
    let site = DoorSite { id, index, poi };
    let bounds = level.bounds();
    let doors = set_doors(world, &site, &mut level, bounds, (0, 0));
    let stairs: Vec<(i64, i64)> = site_level.stairs.iter().map(|s| (s.x, s.y)).collect();
    let mut starts = stairs.clone();
    if index == 0 {
        starts.push(site_level.arrive(Exit::WorldMap));
    }
    let plan = match id {
        SiteId::DevArea(area) => area.spawns(site_level),
        _ => spawns(poi.seed, index, poi.kind, &level, &stairs),
    };
    let saved = world.site_mut(id).levels.entry(index).or_default();
    let holders: Vec<(u32, (i64, i64))> = (plan.iter())
        .filter(|spawn| !saved.dead.contains(&spawn.number))
        .map(|spawn| (spawn.number, spawn.position))
        .collect();
    for (at, holder) in place_locks(poi.seed, &level, &starts, &holders) {
        let Some(mut door) = doors.get(&at).map(|door| door.door) else {
            continue;
        };
        let lock = Lock {
            id: new_id(),
            quality: Quality::Common,
            locked: true,
        };
        door.lock = Some(lock);
        door.swing = Swing::Closed;
        saved.doors.insert(at, door);
        let mut key = Item::new(new_id(), ItemKind::Key);
        key.form = Some(ItemForm::Key(lock.id));
        let mob = *saved.mobs.entry(holder).or_insert_with(new_id);
        saved.records.entry(mob).or_default().carried.push(key);
    }
}
