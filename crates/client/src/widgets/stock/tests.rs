use super::*;
use shared::item::{ItemKind, MadeAt};
use shared::world::WorldId;

const COLUMNS: u8 = 12;

fn arranged(items: &[Item]) -> Vec<Placed> {
    restocked(COLUMNS, &[], items, &[])
}

fn made(n: u128, kind: ItemKind, minute: u64) -> Item {
    Item {
        made_at: Some(MadeAt {
            world: WorldId(Id::for_test(0).0),
            minute,
        }),
        ..Item::new(Id::for_test(n), kind)
    }
}

fn place_of(stock: &[Placed], n: u128) -> (u8, u8) {
    let placed = stock.iter().find(|p| p.item.id == Id::for_test(n));
    placed.map(|placed| (placed.x, placed.y)).unwrap()
}

#[test]
fn the_oldest_goes_first_along_the_row() {
    let items = [
        made(1, ItemKind::Apple, 30),
        made(2, ItemKind::Apple, 10),
        made(3, ItemKind::Apple, 20),
    ];
    let arranged = arranged(&items);
    assert_eq!(place_of(&arranged, 2), (0, 0));
    assert_eq!(place_of(&arranged, 3), (1, 0));
    assert_eq!(place_of(&arranged, 1), (2, 0));
}

#[test]
fn small_things_fill_the_gaps_left_by_large_ones() {
    // Swords are 1 × 5 and boots 2 × 2: boots that don't fit at the end of the
    // row go under it, leaving a gap at the end for the apple.
    let mut items: Vec<Item> = (0..11)
        .map(|n| made(n, ItemKind::Sword, n as u64))
        .collect();
    items.push(made(20, ItemKind::Boots, 20));
    items.push(made(21, ItemKind::Apple, 21));
    let arranged = arranged(&items);
    assert_eq!(place_of(&arranged, 20), (0, 5));
    assert_eq!(place_of(&arranged, 21), (11, 0));
}

#[test]
fn nothing_overlaps_and_there_is_always_room_below() {
    // The ground's width as well as a shop's: the longest thing still fits.
    for columns in [8, COLUMNS] {
        let items: Vec<Item> = ItemKind::ALL
            .iter()
            .cycle()
            .take(60)
            .enumerate()
            .map(|(n, &kind)| made(n as u128, kind, n as u64))
            .collect();
        let stock = restocked(columns, &[], &items, &[]);
        let mut seen = std::collections::BTreeSet::new();
        for placed in &stock {
            for cell in placed.cells() {
                assert!(seen.insert(cell), "overlap");
                assert!(cell.0 < columns);
            }
        }
        assert_eq!(stock.len(), items.len());
        let lowest = seen.iter().map(|cell| cell.1 + 1).max().unwrap();
        assert_eq!(rows_for(&stock), lowest + longest_side());
    }
}

#[test]
fn an_empty_grid_has_room_for_the_longest_thing() {
    assert_eq!(rows_for(&[]), longest_side());
    assert!(longest_side() >= 5, "a sword");
}

#[test]
fn a_gift_goes_where_it_was_let_go_and_the_grid_grows_to_keep_its_empty_rows() {
    let old = arranged(&[made(1, ItemKind::Apple, 1)]);
    let rows = rows_for(&old);
    let items = [made(1, ItemKind::Apple, 1), made(2, ItemKind::Apple, 2)];
    let gift = Gift {
        item: Id::for_test(2),
        rotation: Rotation::None,
        wanted: (4, 5),
    };
    let stock = restocked(COLUMNS, &old, &items, &[gift]);
    assert_eq!(place_of(&stock, 1), (0, 0));
    assert_eq!(place_of(&stock, 2), (4, 5));
    assert!(rows_for(&stock) > rows);
    assert_eq!(rows_for(&stock), 6 + longest_side());
}

#[test]
fn a_gift_let_go_on_something_goes_in_the_first_place_instead() {
    let old = arranged(&[made(1, ItemKind::Apple, 1)]);
    let items = [made(1, ItemKind::Apple, 1), made(2, ItemKind::Apple, 2)];
    let gift = Gift {
        item: Id::for_test(2),
        rotation: Rotation::None,
        wanted: (0, 0),
    };
    let stock = restocked(COLUMNS, &old, &items, &[gift]);
    assert_eq!(place_of(&stock, 2), (1, 0));
}

#[test]
fn opening_again_sorts_gifts_in_with_the_rest() {
    let items = [made(2, ItemKind::Apple, 2), made(1, ItemKind::Apple, 1)];
    assert_eq!(place_of(&arranged(&items), 1), (0, 0));
}
