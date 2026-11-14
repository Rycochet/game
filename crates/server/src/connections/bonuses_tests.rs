//! Bonuses through a connected client: wearing armour, eating apples, and
//! what ends an apple's effect.

use shared::bonus::Source;
use shared::id::Id;
use shared::item::{Item, ItemKind, Quality};
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, DevVitals, ServerMessage};
use shared::vitals::Vital;

use super::tests::TestClient;

fn give(kind: ItemKind, count: u32) -> ClientMessage {
    ClientMessage::DevGiveItem(shared::protocol::DevGive {
        kind,
        size: None,
        magical: false,
        quality: Quality::Common,
        set: false,
        favour: None,
        seed: Some(1),
        count,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    })
}

/// In a castle, with room to walk east, carrying three apples in a stack.
fn with_apples() -> (TestClient, Id) {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(2);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    test.send(&[give(ItemKind::Apple, 3)]);
    let stack = carried(&test, ItemKind::Apple).expect("apples");
    (test, stack)
}

fn carried(test: &TestClient, kind: ItemKind) -> Option<Id> {
    let items = test.client.inventory.items.iter();
    items
        .map(|placed| &placed.item)
        .find(|item: &&Item| item.kind == kind)
        .map(|item| item.id)
}

fn apples(test: &TestClient, stack: Id) -> u32 {
    test.client.inventory.find(stack).map_or(0, Item::count)
}

fn eating(test: &TestClient) -> bool {
    let effects = &test.client.sheet.effects.0;
    effects
        .iter()
        .any(|effect| effect.source == Source::Eaten(ItemKind::Apple))
}

#[test]
fn eating_takes_one_apple_and_lays_on_its_effect() {
    let (mut test, stack) = with_apples();
    let answers = test.send(&[ClientMessage::Consume { item: stack }]);
    assert_eq!(apples(&test, stack), 2, "only the top one");
    assert!(eating(&test));
    let told = answers.iter().any(|answer| match answer {
        ServerMessage::Sheet(sheet) => !sheet.effects.0.is_empty(),
        _ => false,
    });
    assert!(told, "the client is told of the effect");
}

#[test]
fn only_food_is_eaten() {
    let (mut test, _) = with_apples();
    test.send(&[give(ItemKind::Hat, 1)]);
    let hat = carried(&test, ItemKind::Hat).unwrap();
    test.send(&[ClientMessage::Consume { item: hat }]);
    assert!(test.client.inventory.find(hat).is_some());
    assert!(!eating(&test));
}

#[test]
fn moving_or_being_hurt_ends_an_apple() {
    let (mut test, stack) = with_apples();
    test.send(&[ClientMessage::Consume { item: stack }]);
    test.steps(&[(1, 0)]);
    assert!(!eating(&test), "moving ends it");
    test.send(&[ClientMessage::Consume { item: stack }]);
    assert!(eating(&test));
    test.send(&[ClientMessage::DevVitals(DevVitals::Hurt(1))]);
    assert!(!eating(&test), "being hurt ends it");
    assert_eq!(apples(&test, stack), 1);
}

#[test]
fn an_apple_on_the_ground_is_eaten_where_it_lies() {
    let (mut test, stack) = with_apples();
    test.send(&[ClientMessage::DropItem { item: stack }]);
    let answers = test.send(&[ClientMessage::Consume { item: stack }]);
    assert!(eating(&test));
    let left = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::GroundItems { items, .. } => Some(items.clone()),
        _ => None,
    });
    let left = left.expect("told what is on the ground");
    assert_eq!(left.iter().map(Item::count).sum::<u32>(), 2);
}

#[test]
fn armour_changes_attributes_and_keeps_healths_share() {
    let mut test = TestClient::new();
    let base = test.client.sheet.attributes;
    test.send(&[ClientMessage::DevVitals(DevVitals::Hurt(4))]);
    let now = test.world.action_clock.ticks();
    let limits = test.client.limits(now);
    let before = test.client.sheet.vitals.at(Vital::Health, limits, now);
    test.send(&[give(ItemKind::Gambeson, 1)]);
    let gambeson = carried(&test, ItemKind::Gambeson).unwrap();
    test.send(&[ClientMessage::WearItem { item: gambeson }]);
    assert!(test.client.inventory.is_worn(gambeson));
    let with = test.client.bonused.attributes;
    assert!(with.toughness > base.toughness, "{with:?}");
    assert!(with.agility < base.agility, "{with:?}");
    let now = test.world.action_clock.ticks();
    let after = test.client.limits(now);
    assert!(after.health > limits.health);
    let health = test.client.sheet.vitals.at(Vital::Health, after, now);
    // The same share of a larger most, rounding down (`Vitals::rebase`).
    let share = |value: shared::vitals::Points, most: u32| value.milli() / most;
    assert!(share(health, after.health).abs_diff(share(before, limits.health)) <= 1);
    // The client is told what they come to.
    assert!(test.client.keep_vitals(now));
    let told: Vec<_> = std::iter::from_fn(|| test.end.receive().unwrap()).collect();
    assert!(told.contains(&ServerMessage::Attributes(with)));
}
