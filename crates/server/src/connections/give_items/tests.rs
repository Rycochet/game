use shared::armour::Style;
use shared::item::{Inventory, Item, ItemKind, Quality};
use shared::item_form::{ItemForm, RollRequest, choices};
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;

use crate::connections::tests::TestClient;

fn give(kind: ItemKind, seed: u64, count: u32, identical: bool) -> ClientMessage {
    ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
        seed: Some(seed),
        count,
        identical,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    }
}

fn of_kind(inventory: &Inventory, kind: ItemKind) -> Vec<&Item> {
    let items = inventory.items.iter().map(|placed| &placed.item);
    items.filter(|item| item.kind == kind).collect()
}

fn in_a_site() -> TestClient {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    test
}

/// Everything in the bag at the player's feet.
fn on_the_ground(test: &mut TestClient) -> Vec<Item> {
    let visit = test.client.visit.as_ref().expect("in a site");
    let (id, level, at) = (visit.id(), visit.index(), visit.spot());
    let site = test.world.site(id);
    let level = site.levels.get(&level);
    level
        .and_then(|level| level.ground.get(&at))
        .cloned()
        .unwrap_or_default()
}

#[test]
fn identical_items_come_as_one_stack_no_bigger_than_allowed() {
    let mut test = TestClient::new();
    test.send(&[give(ItemKind::Coin, 5, 5, true)]);
    let coins = of_kind(&test.client.inventory, ItemKind::Coin);
    assert_eq!(coins.len(), 1);
    assert_eq!(coins[0].count(), 5);
    let mut test = TestClient::new();
    test.send(&[give(ItemKind::Coin, 5, 100_000, true)]);
    let coins = of_kind(&test.client.inventory, ItemKind::Coin);
    let most = test.world.stacks.most_of(coins[0]);
    assert_eq!(coins.iter().map(|coin| coin.count()).sum::<u32>(), most);
}

#[test]
fn the_same_seed_makes_the_same_item_and_different_ones_differ() {
    let mut test = TestClient::new();
    test.send(&[give(ItemKind::Sword, 7, 1, true)]);
    test.send(&[give(ItemKind::Sword, 7, 1, true)]);
    test.send(&[give(ItemKind::Sword, 7, 3, false)]);
    let swords = of_kind(&test.client.inventory, ItemKind::Sword);
    let swords: Vec<&Item> = swords.into_iter().flat_map(Item::members).collect();
    assert_eq!(swords.len(), 5);
    let forms: Vec<_> = swords.iter().map(|sword| sword.form.clone()).collect();
    assert_eq!(forms[0], forms[1], "alike, so stacked");
    let first_of_batch = forms.iter().filter(|&form| *form == forms[0]).count();
    assert_eq!(first_of_batch, 3, "a batch starts with the seed's own");
}

#[test]
fn on_the_world_map_nothing_is_given_unless_all_of_it_fits() {
    let mut test = TestClient::new();
    let before = test.client.inventory.items.len();
    test.send(&[give(ItemKind::FullArmour, 1, 99, false)]);
    assert_eq!(test.client.inventory.items.len(), before);
    test.send(&[give(ItemKind::Coin, 1, 1, true)]);
    assert_eq!(test.client.inventory.items.len(), before + 1);
}

#[test]
fn in_a_site_what_does_not_fit_goes_on_the_ground() {
    let mut test = in_a_site();
    test.send(&[give(ItemKind::FullArmour, 1, 99, false)]);
    let carried = of_kind(&test.client.inventory, ItemKind::FullArmour).len();
    let ground = on_the_ground(&mut test);
    assert!(carried < 99 && !ground.is_empty());
    assert_eq!(carried + ground.len(), 99);
}

fn give_asking(kind: ItemKind, request: RollRequest) -> ClientMessage {
    match give(kind, 3, 1, true) {
        ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality,
            set,
            favour,
            seed,
            count,
            identical,
            ..
        } => ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality,
            set,
            favour,
            seed,
            count,
            identical,
            request,
            to: None,
        },
        _ => unreachable!(),
    }
}

#[test]
fn what_is_asked_of_its_form_is_kept_or_nothing_is_given() {
    let mut test = TestClient::new();
    let metal = choices(ItemKind::Ring, Quality::Rare, &RollRequest::NONE).materials[0];
    let request = RollRequest {
        material: Some(metal),
        ..RollRequest::NONE
    };
    test.send(&[give_asking(ItemKind::Ring, request)]);
    let rings = of_kind(&test.client.inventory, ItemKind::Ring);
    let Some(ItemForm::Ring(ring)) = &rings[0].form else {
        panic!("a ring has a ring's form");
    };
    assert_eq!(ring.metal, metal);
    // A sword has no style, so asking for one gives nothing.
    let before = test.client.inventory.items.len();
    let request = RollRequest {
        style: Some(Style::Gothic),
        ..RollRequest::NONE
    };
    test.send(&[give_asking(ItemKind::Sword, request)]);
    assert_eq!(test.client.inventory.items.len(), before);
}

/// Stands the player in a castle room with its first mob beside them: its id.
fn beside_a_mob(test: &mut TestClient) -> shared::id::Id {
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 7,
        level: 0,
        ruin: 0,
    }]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    let population = test.world.populations.0.values_mut().next().unwrap();
    population.mobs[0].position = (x + 1, y);
    let mob = population.mobs[0].id;
    test.send(&[]);
    mob
}

fn give_to(kind: ItemKind, count: u32, to: shared::id::Id) -> ClientMessage {
    match give(kind, 5, count, true) {
        ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality,
            set,
            favour,
            seed,
            count,
            identical,
            request,
            ..
        } => ClientMessage::DevGiveItem {
            kind,
            size,
            magical,
            quality,
            set,
            favour,
            seed,
            count,
            identical,
            request,
            to: Some(to),
        },
        _ => unreachable!(),
    }
}

/// What the castle's records say a mob carries.
fn carried_by(test: &mut TestClient, mob: shared::id::Id) -> Vec<Item> {
    let site = test.world.site(crate::storage::SiteId::Dev {
        kind: PoiKind::Castle,
        seed: 7,
        ruin: 0,
    });
    let levels = site.levels.values();
    let mut records = levels.filter_map(|level| level.records.get(&mob));
    records
        .next()
        .map(|record| record.carried.clone())
        .unwrap_or_default()
}

#[test]
fn a_target_mob_in_sight_gets_the_items_and_the_player_none() {
    let mut test = TestClient::new();
    let mob = beside_a_mob(&mut test);
    let before = carried_by(&mut test, mob).len();
    test.send(&[give_to(ItemKind::Coin, 3, mob)]);
    let carried = carried_by(&mut test, mob);
    assert_eq!(carried.len(), before + 3);
    assert!(carried.iter().any(|item| item.kind == ItemKind::Coin));
    assert!(of_kind(&test.client.inventory, ItemKind::Coin).is_empty());
}

#[test]
fn a_target_that_is_no_mob_in_sight_is_left_for_other_players() {
    let mut test = TestClient::new();
    let _ = beside_a_mob(&mut test);
    test.send(&[give_to(ItemKind::Coin, 3, shared::id::Id::for_test(999))]);
    assert!(of_kind(&test.client.inventory, ItemKind::Coin).is_empty());
    // Left for the pass over other players, which finds nobody by it.
    assert_eq!(test.client.for_others.len(), 1);
}
