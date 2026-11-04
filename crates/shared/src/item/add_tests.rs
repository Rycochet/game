use super::*;
use crate::id::Id;

#[test]
fn something_new_goes_by_the_same_rules_as_a_move() {
    let mut inventory = Inventory::default();
    let stone = Item::new(Id::for_test(1), ItemKind::Stone);
    let coin = Item::new(Id::for_test(2), ItemKind::Coin);
    assert!(
        inventory
            .add(stone.clone(), Holder::Inventory, 0, 0, Rotation::None)
            .is_ok()
    );
    assert_eq!(
        inventory.check_add(&coin, Holder::Inventory, 1, 1, Rotation::None),
        Err(MoveRefused::NoRoom)
    );
    assert_eq!(
        inventory.check_add(&coin, Holder::Inside(Id::for_test(9)), 0, 0, Rotation::None),
        inventory.check_move(
            stone.id,
            Holder::Inside(Id::for_test(9)),
            0,
            0,
            Rotation::None
        ),
    );
    assert!(
        inventory
            .add(coin, Holder::Inventory, 2, 0, Rotation::None)
            .is_ok()
    );
}
