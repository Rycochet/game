use super::kept_on_hard;
use crate::creature::CreatureKind;
use crate::difficulty::Difficulty;
use crate::id::Id;
use crate::item::{Inventory, Item, ItemKind, Made, StackLimits};

/// A human carrying these kinds, ids from 1, wearing those it can.
fn dressed(kinds: &[ItemKind]) -> Inventory {
    let def = CreatureKind::Human.def();
    let mut inventory = Inventory {
        creature: CreatureKind::Human,
        size: (40, 40),
        ..Inventory::default()
    };
    for (index, &kind) in kinds.iter().enumerate() {
        let item = Item {
            made: Made::for_body(kind, def.size, def.fingers(), false),
            ..Item::new(Id::for_test(index as u128 + 1), kind)
        };
        inventory
            .give(item, StackLimits::NONE, &mut crate::id::stand_ins())
            .expect("room");
    }
    for index in 0..kinds.len() {
        let _ = inventory.wear(Id::for_test(index as u128 + 1));
    }
    inventory
}

fn kinds(items: &[Item]) -> Vec<ItemKind> {
    let mut kinds: Vec<_> = items.iter().map(|item| item.kind).collect();
    kinds.sort_by_key(|kind| format!("{kind:?}"));
    kinds
}

const OUTFIT: [ItemKind; 7] = [
    ItemKind::TShirt,
    ItemKind::Socks,
    ItemKind::Trousers,
    ItemKind::Ring,
    ItemKind::Necklace,
    ItemKind::Sword,
    ItemKind::Gambeson,
];

#[test]
fn only_skin_clothes_that_arent_jewellery_are_kept_on_hard() {
    use ItemKind::*;
    let inventory = dressed(&OUTFIT);
    let kept: Vec<_> = inventory
        .worn
        .iter()
        .filter(|item| kept_on_hard(item))
        .collect();
    assert_eq!(
        kinds(&kept.into_iter().cloned().collect::<Vec<_>>()),
        vec![Socks, TShirt]
    );
}

#[test]
fn hard_drops_all_but_skin_clothes() {
    use ItemKind::*;
    let mut inventory = dressed(&[OUTFIT.as_slice(), &[Stone]].concat());
    let dropped = inventory.drop_on_death(Difficulty::Hard);
    assert_eq!(kinds(&inventory.worn), vec![Socks, TShirt]);
    assert!(inventory.items.is_empty());
    assert!(dropped.iter().any(|item| item.kind == Stone));
    assert!(dropped.iter().any(|item| item.kind == Ring));
    assert!(dropped.iter().any(|item| item.kind == Sword));
    assert_eq!(dropped.len() + 2, OUTFIT.len() + 1);
}

#[test]
fn ironman_drops_everything_and_easier_ones_nothing() {
    let mut inventory = dressed(&OUTFIT);
    let before = inventory.clone();
    assert!(inventory.drop_on_death(Difficulty::Easy).is_empty());
    assert!(inventory.drop_on_death(Difficulty::Normal).is_empty());
    assert_eq!(inventory, before);
    let dropped = inventory.drop_on_death(Difficulty::Ironman);
    assert_eq!(dropped.len(), OUTFIT.len());
    assert!(inventory.worn.is_empty() && inventory.items.is_empty());
}
