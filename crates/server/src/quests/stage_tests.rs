use shared::item::ItemKind;
use shared::item_form::ItemForm;
use shared::quest::Quest;
use shared::site::{CAVE_VERSION, Site};

use super::accept::accept;
use super::dungeon::{RESET_AFTER, enter, site_of};
use super::dungeon_tests::a_dungeon_where;
use super::tests::NOW;
use crate::connections::tests::TestClient;

/// Whether a quest's dungeon has a stage door on its first level.
fn staged(quest: &Quest) -> bool {
    let Some(dungeon) = quest.dungeon else {
        return false;
    };
    let site = Site::with_generator(
        dungeon.poi.kind,
        dungeon.poi.seed,
        CAVE_VERSION,
        dungeon.generator(),
    );
    site.is_some_and(|site| dungeon.stage_door(0, &site.level(0)).is_some())
}

#[test]
fn a_stage_door_is_locked_with_a_quest_key_on_the_near_side() {
    let mut test = TestClient::new();
    let (_, quest) = a_dungeon_where(&mut test, staged);
    assert!(accept(
        &mut test.client,
        &mut test.world,
        quest.id,
        NOW,
        false
    ));
    assert!(enter(&mut test.client, &mut test.world, quest.id, false));
    let site = site_of(&test.client, quest.id).unwrap();
    let visit = test.client.visit.as_ref().unwrap();
    assert_eq!(visit.id(), site);
    let at = quest
        .dungeon
        .unwrap()
        .stage_door(0, visit.site_level())
        .unwrap();
    assert!(visit.is_locked(at));
    // Only that door is locked, and one of the level's creatures has its key,
    // a quest item of the quest's.
    let saved = test.world.site(site).levels[&0].clone();
    let locked: Vec<_> = (saved.doors.iter())
        .filter(|(_, door)| door.is_locked())
        .map(|(at, _)| *at)
        .collect();
    assert_eq!(locked, vec![at]);
    let lock = saved.doors[&at].lock.unwrap();
    let keys: Vec<_> = (saved.records.values())
        .flat_map(|record| record.carried.iter().cloned())
        .filter(|item| item.kind == ItemKind::Key)
        .collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].form, Some(ItemForm::Key(lock.id)));
    assert_eq!(keys[0].quest, Some(quest.id));
    // It has the quest's dot while locked.
    let marked = super::stage_door::locked_stage(&test.client, &mut test.world);
    assert_eq!(marked.map(|(door, _)| door), Some(at));
    // A key carried out vanishes when the copy resets.
    let mut key = keys[0].clone();
    key.id = crate::ids::new_id();
    test.client
        .inventory
        .give(key, test.world.stacks, &mut crate::ids::new_id)
        .unwrap();
    assert!(crate::sites::leave_dungeon(
        &mut test.client,
        &mut test.world
    ));
    let left = test.client.character.quests.dungeons[&quest.id]
        .left
        .unwrap();
    let due = left + RESET_AFTER;
    assert!(super::dungeon_reset::reset_due(
        &mut test.client,
        &mut test.world,
        due
    ));
    assert_eq!(test.client.inventory.remove_quest_items(quest.id), 0);
}
