use super::*;
use shared::id::Id;
use shared::item::ItemKind;

#[test]
fn the_list_is_all_then_each_category() {
    let mut dropdown = Dropdown {
        options: vec![String::new(); 1 + ItemCategory::ALL.len()],
        ..default()
    };
    assert_eq!(chosen(&dropdown), None);
    for (at, category) in ItemCategory::ALL.into_iter().enumerate() {
        dropdown.selected = at + 1;
        assert_eq!(chosen(&dropdown), Some(category));
    }
}

#[test]
fn only_what_is_not_chosen_fades() {
    let apple = Item::new(Id::for_test(1), ItemKind::Apple);
    assert!(!dims(None, &apple));
    assert!(!dims(Some(ItemCategory::Food), &apple));
    assert!(dims(Some(ItemCategory::Weapons), &apple));
}

#[test]
fn categories_with_nothing_in_are_greyed_but_never_all() {
    let greyed = greyed(&[ItemCategory::Food, ItemCategory::Food]);
    assert_eq!(greyed.len(), 1 + ItemCategory::ALL.len());
    assert!(!greyed[0]);
    for (at, category) in ItemCategory::ALL.into_iter().enumerate() {
        assert_eq!(
            greyed[at + 1],
            category != ItemCategory::Food,
            "{category:?}"
        );
    }
}
