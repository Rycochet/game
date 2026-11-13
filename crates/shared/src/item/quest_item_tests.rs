use crate::id::Id;
use crate::item::{Holder, Inventory, Item, ItemKind, Rotation};
use crate::poi::ChunkPos;
use crate::quest::{QuestId, Slot};

fn quest(n: i64) -> QuestId {
    QuestId {
        town: ChunkPos { x: n, y: 0 },
        slot: Slot::Feeder,
    }
}

fn parcel(n: u128, of: QuestId) -> Item {
    Item {
        quest: Some(of),
        ..Item::new(Id::for_test(n), ItemKind::Parcel)
    }
}

#[test]
fn a_quest_item_is_found_inside_what_holds_it() {
    let mut pouch = Item::new(Id::for_test(1), ItemKind::Pouch);
    assert!(!pouch.holds_quest_item());
    pouch.contents.push(crate::item::Placed {
        item: Item::new(Id::for_test(2), ItemKind::Key),
        x: 0,
        y: 0,
        rotation: Rotation::None,
    });
    pouch.contents.push(crate::item::Placed {
        item: Item {
            quest: Some(quest(1)),
            ..Item::new(Id::for_test(3), ItemKind::Key)
        },
        x: 1,
        y: 0,
        rotation: Rotation::None,
    });
    assert!(pouch.holds_quest_item() && !pouch.is_quest_item());
    assert_eq!(pouch.strip_quest_items(), 1);
    assert!(!pouch.holds_quest_item());
    assert_eq!(pouch.contents.len(), 1, "the rest stays");
}

#[test]
fn a_quests_items_go_from_wherever_they_are_carried() {
    let mut inventory = Inventory::default();
    for (n, of) in [(10, quest(1)), (11, quest(2))] {
        let (x, y, rotation) = inventory
            .first_place_for_new(&parcel(n, of), Holder::Inventory, Rotation::None)
            .expect("room");
        inventory
            .add(parcel(n, of), Holder::Inventory, x, y, rotation)
            .unwrap();
    }
    assert_eq!(inventory.quest_items(quest(1)), 1);
    assert_eq!(inventory.remove_quest_items(quest(1)), 1);
    assert_eq!(inventory.quest_items(quest(1)), 0);
    assert_eq!(inventory.quest_items(quest(2)), 1, "another quest's stay");
}
