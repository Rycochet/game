//! A quest dungeon level's stage door (`shared::quest::QuestDungeon::stage_door`,
//! owner): locked the first time the level is used, in place of the usual
//! locks, its key a quest item of the quest's carried by one of the level's
//! creatures on the near side, so killing it drops it. Without such a creature
//! the door is left unlocked.

use shared::door::{Lock, Swing};
use shared::item::{Item, ItemKind, Quality};
use shared::item_form::ItemForm;
use shared::quest::QuestId;
use shared::site::SiteLevel;

use super::VisitDoors;
use crate::ServerWorld;
use crate::ids::new_id;
use crate::storage::SiteId;

pub(super) fn lock_stage(
    world: &mut ServerWorld,
    (id, quest): (SiteId, QuestId),
    index: u32,
    site_level: &SiteLevel,
    doors: &VisitDoors,
) {
    let Some(dungeon) = crate::quests::dungeon_of(world, quest) else {
        return;
    };
    let Some(at) = dungeon.stage_door(index, site_level) else {
        return;
    };
    let stairs: Vec<(i64, i64)> = site_level.stairs.iter().map(|s| (s.x, s.y)).collect();
    let holders: Vec<(u32, (i64, i64))> = (dungeon.spawns(index, &site_level.level, &stairs))
        .iter()
        .map(|spawn| (spawn.number, spawn.position))
        .collect();
    let Some(holder) = dungeon.key_holder(index, site_level, at, &holders) else {
        return;
    };
    let Some(mut door) = doors.get(&at).map(|door| door.door) else {
        return;
    };
    let lock = Lock {
        id: new_id(),
        quality: Quality::Common,
        locked: true,
    };
    door.lock = Some(lock);
    door.swing = Swing::Closed;
    let saved = world.site_mut(id).levels.entry(index).or_default();
    saved.doors.insert(at, door);
    let mut key = Item::new(new_id(), ItemKind::Key);
    key.form = Some(ItemForm::Key(lock.id));
    key.quest = Some(quest);
    let mob = *saved.mobs.entry(holder).or_insert_with(new_id);
    saved.records.entry(mob).or_default().carried.push(key);
}
