use super::*;
use crate::creature::{CreatureKind, Size};
use crate::id::Id;
use crate::item::{Favour, ItemKind, Quality};
use crate::rng::Seed;

fn made(order: &ItemOrder, id: u128, seed: u64) -> crate::item::Item {
    let mut next = 1000;
    let mut new_id = || {
        next += 1;
        Id::for_test(next)
    };
    make_item(
        order,
        CreatureKind::Human,
        Id::for_test(id),
        Seed::new(seed),
        None,
        &mut new_id,
    )
}

#[test]
fn the_same_seed_makes_the_same_item_whatever_its_id() {
    for kind in [
        ItemKind::Sword,
        ItemKind::Ring,
        ItemKind::FullArmour,
        ItemKind::Gem,
    ] {
        let order = ItemOrder::plain(kind, Quality::Rare);
        let (one, two) = (made(&order, 1, 42), made(&order, 2, 42));
        assert_eq!(one.form, two.form, "{kind:?}");
        assert!(one.looks_like(&two) || kind == ItemKind::Ring, "{kind:?}");
    }
    let order = ItemOrder::plain(ItemKind::Sword, Quality::Rare);
    assert_ne!(made(&order, 1, 1).form, made(&order, 1, 2).form);
}

#[test]
fn what_is_asked_for_is_kept_where_the_kind_allows_it() {
    let order = ItemOrder {
        size: Some(Size::Large),
        magical: true,
        quality: Quality::Common,
        set: true,
        favour: Some(Favour::Blessed),
        ..ItemOrder::plain(ItemKind::Hat, Quality::Common)
    };
    let hat = made(&order, 1, 1);
    assert_eq!(hat.made.size, Size::Large);
    assert!(hat.is_magical(), "magic raises the quality enough for it");
    assert!(hat.set);
    assert_eq!(hat.favour, Some(Favour::Blessed));
    let apple = made(
        &ItemOrder {
            kind: ItemKind::Apple,
            ..order
        },
        2,
        1,
    );
    assert_eq!(apple.made.size, Size::Medium);
    assert!(!apple.set && apple.favour.is_none() && !apple.is_magical());
}

#[test]
fn a_batch_starts_with_its_own_seed() {
    assert_eq!(batch_seed(9, 0), Seed::new(9));
    assert_ne!(batch_seed(9, 1), batch_seed(9, 0));
}
