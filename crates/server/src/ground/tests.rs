use shared::energy::site_step_cost;
use shared::item::{Favour, Inventory, ItemKind, Quality};
use shared::protocol::{ClientMessage, ServerMessage};

use super::*;
use crate::connections::tests::TestClient;

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

fn give(kind: ItemKind) -> ClientMessage {
    ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
    }
}

fn give_cursed(kind: ItemKind) -> ClientMessage {
    ClientMessage::DevGiveItem {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: Some(Favour::Cursed),
    }
}

fn carrying(answers: &[ServerMessage]) -> Inventory {
    let found = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::Inventory(inventory) => Some(inventory.clone()),
        _ => None,
    });
    found.expect("an inventory among the answers")
}

fn ground(answers: &[ServerMessage]) -> Vec<Item> {
    let found = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::GroundItems { items, .. } => Some(items.clone()),
        _ => None,
    });
    found.expect("what is on the ground among the answers")
}

/// A character holding one of these in the pockets, and its id.
fn holding(kind: ItemKind) -> (TestClient, Id) {
    let mut test = in_a_site();
    let id = carrying(&test.send(&[give(kind)])).pockets[0].item.id;
    (test, id)
}

#[test]
fn dropped_things_lie_under_the_player_and_come_back() {
    let (mut test, coin) = holding(ItemKind::Coin);
    let answers = test.send(&[ClientMessage::DropItem { item: coin }]);
    assert!(carrying(&answers).find(coin).is_none());
    assert_eq!(ground(&answers)[0].id, coin);
    assert!(test.client.unsaved);

    let answers = test.send(&[ClientMessage::TakeFromGround {
        item: coin,
        to: Holder::Pockets,
        x: 2,
        y: 1,
        rotation: Rotation::None,
    }]);
    assert!(carrying(&answers).find(coin).is_some());
    assert!(ground(&answers).is_empty());
    let visit = test.client.visit.as_ref().unwrap();
    let (id, kind, index) = (visit.id(), visit.kind(), visit.index());
    let level = &test.world.site(id, kind).levels[&index];
    assert!(level.ground.is_empty(), "nothing is left to save");
}

#[test]
fn stepping_off_it_stops_it_being_reachable() {
    let (mut test, coin) = holding(ItemKind::Coin);
    let start = test.client.visit.as_ref().unwrap().room_to_walk_east(2);
    test.send(&[ClientMessage::DevTeleport {
        x: start.0,
        y: start.1,
    }]);
    test.send(&[ClientMessage::DropItem { item: coin }]);
    let answers = test.steps(&[(1, 0)]);
    assert!(ground(&answers).is_empty(), "nothing on this cell");

    let answers = test.send(&[ClientMessage::BinFromGround { item: coin }]);
    assert!(carrying(&answers).bin.is_none());

    let answers = test.steps(&[(-1, 0)]);
    assert_eq!(ground(&answers)[0].id, coin, "told on coming back");
}

#[test]
fn the_world_map_has_no_ground() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::Coin)]);
    let coin = carrying(&answers).pockets[0].item.id;
    let answers = test.send(&[ClientMessage::DropItem { item: coin }]);
    assert!(carrying(&answers).find(coin).is_some());
}

#[test]
fn what_cant_go_where_it_was_asked_stays_on_the_ground() {
    let (mut test, stone) = holding(ItemKind::Stone);
    test.send(&[ClientMessage::DropItem { item: stone }]);
    // Off the edge of the pockets.
    let answers = test.send(&[ClientMessage::TakeFromGround {
        item: stone,
        to: Holder::Pockets,
        x: 200,
        y: 0,
        rotation: Rotation::None,
    }]);
    assert!(carrying(&answers).find(stone).is_none());
    assert_eq!(ground(&answers)[0].id, stone);
}

#[test]
fn clothes_go_straight_on_and_off() {
    let (mut test, boots) = holding(ItemKind::Boots);
    test.send(&[ClientMessage::DropItem { item: boots }]);
    let answers = test.send(&[ClientMessage::WearFromGround { item: boots }]);
    assert!(carrying(&answers).is_worn(boots));
    assert!(ground(&answers).is_empty());

    let answers = test.send(&[ClientMessage::DropItem { item: boots }]);
    assert!(!carrying(&answers).is_worn(boots));
    assert_eq!(ground(&answers)[0].id, boots);
}

#[test]
fn what_is_under_something_stays_on() {
    let (mut test, boots) = holding(ItemKind::Boots);
    let worn = carrying(&test.send(&[ClientMessage::WearItem { item: boots }]));
    let shoes = worn.worn.iter().find(|item| item.kind == ItemKind::Shoes);
    let shoes = shoes.unwrap().id;
    let answers = test.send(&[ClientMessage::DropItem { item: shoes }]);
    assert!(carrying(&answers).is_worn(shoes));
}

#[test]
fn a_curse_doesnt_let_go_for_the_ground_either() {
    let mut test = in_a_site();
    let answers = test.send(&[give_cursed(ItemKind::Hat)]);
    let hat = carrying(&answers).pockets.last().unwrap().item.id;
    test.send(&[ClientMessage::WearItem { item: hat }]);
    let answers = test.send(&[ClientMessage::DropItem { item: hat }]);
    let seen = carrying(&answers);
    assert!(seen.is_worn(hat));
    assert_eq!(seen.find(hat).unwrap().favour, Some(Favour::Cursed));
}

#[test]
fn what_is_on_the_ground_keeps_its_secrets() {
    let mut test = in_a_site();
    let answers = test.send(&[give_cursed(ItemKind::Hat)]);
    let hat = carrying(&answers).pockets.last().unwrap().item.id;
    let answers = test.send(&[ClientMessage::DropItem { item: hat }]);
    assert_eq!(ground(&answers)[0].favour, None);
    let answers = test.send(&[ClientMessage::DevTruesight(true)]);
    assert_eq!(ground(&answers)[0].favour, Some(Favour::Cursed));
}

#[test]
fn throwing_away_from_the_ground_goes_in_the_bin() {
    let (mut test, coin) = holding(ItemKind::Coin);
    test.send(&[ClientMessage::DropItem { item: coin }]);
    let answers = test.send(&[ClientMessage::BinFromGround { item: coin }]);
    assert!(carrying(&answers).in_bin(coin));
    assert!(ground(&answers).is_empty());
}

/// What is said of bags among these answers.
fn bags_told(answers: &[ServerMessage]) -> Vec<&ServerMessage> {
    let bags = answers.iter().filter(|answer| {
        matches!(
            answer,
            ServerMessage::GroundSeen { .. } | ServerMessage::GroundGone { .. }
        )
    });
    bags.collect()
}

/// Puts something on the ground here, as someone else would.
fn lay_down(test: &mut TestClient, at: (i64, i64), grams: ItemKind) {
    let visit = test.client.visit.as_ref().unwrap();
    let (id, kind, index) = (visit.id(), visit.kind(), visit.index());
    let level = test.world.site_mut(id, kind).levels.entry(index);
    let pile = level.or_default().ground.entry(at).or_default();
    pile.push(Item::new(Id::for_test(pile.len() as u128 + 100), grams));
}

#[test]
fn bags_are_only_told_of_when_seen_and_remembered_until_seen_again() {
    let mut test = in_a_site();
    let far = test.client.visit.as_mut().unwrap().walkable_out_of_sight();
    let here = test.client.visit.as_ref().unwrap().position();
    lay_down(&mut test, far, ItemKind::Coin);
    assert!(bags_told(&test.send(&[])).is_empty(), "out of sight");

    let go_there = ClientMessage::DevTeleport { x: far.0, y: far.1 };
    let answers = test.send(std::slice::from_ref(&go_there));
    let seen = ServerMessage::GroundSeen {
        x: far.0,
        y: far.1,
        load: Load::Flat,
    };
    assert_eq!(bags_told(&answers), [&seen]);

    // Someone takes it while the player is away: they still remember it.
    let answers = test.send(&[ClientMessage::DevTeleport {
        x: here.0,
        y: here.1,
    }]);
    assert_eq!(bags_told(&answers), [&seen], "remembered on arriving");
    let visit = test.client.visit.as_ref().unwrap();
    let (id, kind, index) = (visit.id(), visit.kind(), visit.index());
    let level = test.world.site_mut(id, kind).levels.entry(index);
    level.or_default().ground.remove(&far);
    assert!(bags_told(&test.send(&[])).is_empty());

    // And find out on seeing the place again.
    let answers = test.send(&[go_there]);
    let gone = ServerMessage::GroundGone { x: far.0, y: far.1 };
    assert_eq!(bags_told(&answers), [&seen, &gone]);
}

#[test]
fn a_bag_seen_to_fill_up_looks_it() {
    let mut test = in_a_site();
    let here = test.client.visit.as_ref().unwrap().position();
    lay_down(&mut test, here, ItemKind::Coin);
    test.send(&[]);
    lay_down(&mut test, here, ItemKind::Sword);
    let answers = test.send(&[]);
    let [ServerMessage::GroundSeen { load, .. }] = bags_told(&answers)[..] else {
        panic!("told how it looks now");
    };
    assert!(*load > Load::Flat);
}

#[test]
fn ten_kilos_takes_about_a_step_to_pick_up() {
    assert_eq!(handling_cost(10_000), site_step_cost((0, 0), (1, 0)));
    assert_eq!(handling_cost(1), 1, "nothing is free");
    assert_eq!(handling_cost(0), 0);
}
