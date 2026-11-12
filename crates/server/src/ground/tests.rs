use shared::energy::{ACTION_TICKS_PER_SECOND, NORMAL_SPEED};
use shared::item::{Favour, Inventory, ItemKind, Quality};
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};

use shared::ground::Load;
use shared::spot::Spot;

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
    ClientMessage::DevGiveItem(shared::protocol::DevGive {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: None,
        seed: Some(1),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    })
}

fn give_cursed(kind: ItemKind) -> ClientMessage {
    ClientMessage::DevGiveItem(shared::protocol::DevGive {
        kind,
        size: None,
        magical: false,
        quality: Quality::Rare,
        set: false,
        favour: Some(Favour::Cursed),
        seed: Some(1),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    })
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

/// A character holding one of these in the inventory, and its id.
fn holding(kind: ItemKind) -> (TestClient, Id) {
    let mut test = in_a_site();
    let id = carrying(&test.send(&[give(kind)])).items[0].item.id;
    (test, id)
}

#[test]
fn dropped_things_lie_under_the_player_and_come_back() {
    let (mut test, coin) = holding(ItemKind::CopperCoin);
    let answers = test.send(&[ClientMessage::DropItem { item: coin }]);
    assert!(carrying(&answers).find(coin).is_none());
    assert_eq!(ground(&answers)[0].id, coin);
    assert!(test.client.unsaved);

    let answers = test.send(&[ClientMessage::TakeFromGround {
        item: coin,
        to: Holder::Inventory,
        x: 2,
        y: 1,
        rotation: Rotation::None,
    }]);
    assert!(carrying(&answers).find(coin).is_some());
    assert!(ground(&answers).is_empty());
    let visit = test.client.visit.as_ref().unwrap();
    let (id, index) = (visit.id(), visit.index());
    let level = &test.world.site(id).levels[&index];
    assert!(level.ground.is_empty(), "nothing is left to save");
}

#[test]
fn stepping_off_it_stops_it_being_reachable() {
    let (mut test, coin) = holding(ItemKind::CopperCoin);
    let start = test.client.visit.as_ref().unwrap().room_to_walk_east(2);
    test.send(&[ClientMessage::DevTeleport {
        x: start.0,
        y: start.1,
    }]);
    test.send(&[ClientMessage::DropItem { item: coin }]);
    let answers = test.steps(&[(1, 0)]);
    assert!(ground(&answers).is_empty(), "out of reach a cell away");

    let answers = test.send(&[ClientMessage::BinFromGround { item: coin }]);
    assert!(carrying(&answers).bin.is_none());

    let answers = test.steps(&[(-1, 0)]);
    assert_eq!(ground(&answers)[0].id, coin, "told on coming back");
}

#[test]
fn the_world_map_has_no_ground() {
    let mut test = TestClient::new();
    let answers = test.send(&[give(ItemKind::CopperCoin)]);
    let coin = carrying(&answers).items[0].item.id;
    let answers = test.send(&[ClientMessage::DropItem { item: coin }]);
    assert!(carrying(&answers).find(coin).is_some());
}

#[test]
fn what_cant_go_where_it_was_asked_stays_on_the_ground() {
    let (mut test, stone) = holding(ItemKind::Stone);
    test.send(&[ClientMessage::DropItem { item: stone }]);
    // Off the edge of the inventory.
    let answers = test.send(&[ClientMessage::TakeFromGround {
        item: stone,
        to: Holder::Inventory,
        x: 200,
        y: 0,
        rotation: Rotation::None,
    }]);
    assert!(carrying(&answers).find(stone).is_none());
    assert_eq!(ground(&answers)[0].id, stone);
}

#[test]
fn clothes_go_straight_on_and_off() {
    let (mut test, sabatons) = holding(ItemKind::FootArmour);
    test.send(&[ClientMessage::DropItem { item: sabatons }]);
    let answers = test.send(&[ClientMessage::WearFromGround { item: sabatons }]);
    assert!(carrying(&answers).is_worn(sabatons));
    assert!(ground(&answers).is_empty());

    let answers = test.send(&[ClientMessage::DropItem { item: sabatons }]);
    assert!(!carrying(&answers).is_worn(sabatons));
    assert_eq!(ground(&answers)[0].id, sabatons);
}

#[test]
fn what_is_under_something_comes_off_too() {
    let (mut test, sabatons) = holding(ItemKind::FootArmour);
    let worn = carrying(&test.send(&[ClientMessage::WearItem { item: sabatons }]));
    let shoes = worn.worn.iter().find(|item| item.kind == ItemKind::Shoes);
    let shoes = shoes.unwrap().id;
    let answers = test.send(&[ClientMessage::DropItem { item: shoes }]);
    assert!(!carrying(&answers).is_worn(shoes));
    assert!(carrying(&answers).is_worn(sabatons));
    assert!(ground(&answers).iter().any(|item| item.id == shoes));
}

#[test]
fn a_curse_doesnt_let_go_for_the_ground_either() {
    let mut test = in_a_site();
    let answers = test.send(&[give_cursed(ItemKind::Hat)]);
    let hat = carrying(&answers).items.last().unwrap().item.id;
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
    let hat = carrying(&answers).items.last().unwrap().item.id;
    let answers = test.send(&[ClientMessage::DropItem { item: hat }]);
    assert_eq!(ground(&answers)[0].favour, None);
    let answers = test.send(&[ClientMessage::DevTruesight(true)]);
    assert_eq!(ground(&answers)[0].favour, Some(Favour::Cursed));
}

#[test]
fn throwing_away_from_the_ground_goes_in_the_bin() {
    let (mut test, coin) = holding(ItemKind::CopperCoin);
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
fn lay_down(test: &mut TestClient, at: Spot, grams: ItemKind) {
    let visit = test.client.visit.as_ref().unwrap();
    let (id, index) = (visit.id(), visit.index());
    let ground = &mut test
        .world
        .site_mut(id)
        .levels
        .entry(index)
        .or_default()
        .ground;
    let laid = ground.values().map(Vec::len).sum::<usize>() as u128;
    let pile = ground.entry(at).or_default();
    pile.push(Item::new(Id::for_test(laid + 100), grams));
}

#[test]
fn bags_are_only_told_of_when_seen_and_remembered_until_seen_again() {
    let mut test = in_a_site();
    // (The test client teleports to cells' middles.)
    let far_cell = test.client.visit.as_mut().unwrap().walkable_out_of_sight();
    let far = Spot::middle_of(far_cell);
    let here = test.client.visit.as_ref().unwrap().position();
    lay_down(&mut test, far, ItemKind::CopperCoin);
    assert!(bags_told(&test.send(&[])).is_empty(), "out of sight");

    let go_there = ClientMessage::DevTeleport {
        x: far_cell.0,
        y: far_cell.1,
    };
    let answers = test.send(std::slice::from_ref(&go_there));
    let seen = ServerMessage::GroundSeen {
        at: far,
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
    let (id, index) = (visit.id(), visit.index());
    let level = test.world.site_mut(id).levels.entry(index);
    level.or_default().ground.remove(&far);
    assert!(bags_told(&test.send(&[])).is_empty());

    // And find out on seeing the place again.
    let answers = test.send(&[go_there]);
    let gone = ServerMessage::GroundGone { at: far };
    assert_eq!(bags_told(&answers), [&seen, &gone]);
}

#[test]
fn a_bag_seen_to_fill_up_looks_it() {
    let mut test = in_a_site();
    let here = test.client.visit.as_ref().unwrap().spot();
    lay_down(&mut test, here, ItemKind::CopperCoin);
    test.send(&[]);
    lay_down(&mut test, here, ItemKind::Sword);
    let answers = test.send(&[]);
    let [ServerMessage::GroundSeen { load, .. }] = bags_told(&answers)[..] else {
        panic!("told how it looks now");
    };
    assert!(*load > Load::Flat);
}

#[test]
fn ten_kilos_takes_a_twelfth_of_a_second_to_pick_up() {
    // A twelfth of a second's energy at the usual speed.
    let per_second = NORMAL_SPEED * ACTION_TICKS_PER_SECOND as u32;
    assert_eq!(handling_cost(10_000), per_second / 12);
    assert_eq!(handling_cost(1), 1, "nothing is free");
    assert_eq!(handling_cost(0), 0);
}

#[test]
fn a_stack_on_the_ground_joins_a_carried_one_as_far_as_it_fits() {
    let mut test = in_a_site();
    let three = [
        give(ItemKind::CopperCoin),
        give(ItemKind::CopperCoin),
        give(ItemKind::CopperCoin),
    ];
    let dropped = carrying(&test.send(&three)).items[0].item.id;
    test.send(&[ClientMessage::DropItem { item: dropped }]);
    let kept = carrying(&test.send(&[give(ItemKind::CopperCoin)])).items[0]
        .item
        .id;
    test.world.stacks = shared::item::StackLimits {
        grams_per_cell: 2000,
        most: 3,
    };
    let answers = test.send(&[ClientMessage::StackFromGround {
        item: dropped,
        onto: kept,
    }]);
    assert_eq!(carrying(&answers).find(kept).map(Item::count), Some(3));
    let left = ground(&answers);
    assert_eq!((left[0].id, left[0].count()), (dropped, 1));
}

#[test]
fn wearing_a_stack_from_the_ground_leaves_the_rest_there() {
    let mut test = in_a_site();
    let hats = carrying(&test.send(&[give(ItemKind::Hat), give(ItemKind::Hat)]));
    let hats = &hats.items[0].item;
    let (bottom, top) = (hats.id, hats.top().id);
    test.send(&[ClientMessage::DropItem { item: bottom }]);
    let answers = test.send(&[ClientMessage::WearFromGround { item: bottom }]);
    assert!(carrying(&answers).is_worn(top));
    let left = ground(&answers);
    assert_eq!((left[0].id, left[0].count()), (bottom, 1));
}

/// Where the player stands, in units.
fn standing(test: &TestClient) -> Spot {
    test.client.visit.as_ref().unwrap().spot()
}

/// The bags on the player's level.
fn bags_here(test: &mut TestClient) -> Vec<(Spot, usize)> {
    let visit = test.client.visit.as_ref().unwrap();
    let (id, index) = (visit.id(), visit.index());
    let level = test.world.site(id).levels.get(&index);
    let bags = level.into_iter().flat_map(|level| level.ground.iter());
    bags.map(|(&at, items)| (at, items.len())).collect()
}

#[test]
fn a_drop_joins_a_bag_within_reach_or_starts_one_at_the_feet() {
    let (mut test, coin) = holding(ItemKind::CopperCoin);
    let here = standing(&test);
    let near = here.offset(150, 0);
    lay_down(&mut test, near, ItemKind::Sword);
    test.send(&[ClientMessage::DropItem { item: coin }]);
    assert_eq!(bags_here(&mut test), [(near, 2)], "into the bag in reach");

    // Out of reach of it, a new bag where they stand.
    let mut test = in_a_site();
    let coin = carrying(&test.send(&[give(ItemKind::CopperCoin)])).items[0]
        .item
        .id;
    let here = standing(&test);
    let far = here.offset(1000, 0);
    lay_down(&mut test, far, ItemKind::Sword);
    test.send(&[ClientMessage::DropItem { item: coin }]);
    let mut bags = bags_here(&mut test);
    bags.sort();
    let mut expected = vec![(here, 1), (far, 1)];
    expected.sort();
    assert_eq!(bags, expected);
}

#[test]
fn anything_in_reach_can_be_picked_up_and_is_told_of() {
    let mut test = in_a_site();
    let here = standing(&test);
    let (a, b) = (here.offset(-120, 0), here.offset(0, 150));
    lay_down(&mut test, a, ItemKind::CopperCoin);
    lay_down(&mut test, b, ItemKind::Sword);
    let answers = test.send(&[]);
    let told = answers.iter().rev().find_map(|answer| match answer {
        ServerMessage::GroundItems { bags, items } => Some((bags.clone(), items.len())),
        _ => None,
    });
    assert_eq!(told, Some((vec![a, b], 2)), "nearest bag first");

    let sword = ground(&answers)[1].id;
    let answers = test.send(&[ClientMessage::BinFromGround { item: sword }]);
    assert!(carrying(&answers).bin.is_some());
    assert_eq!(bags_here(&mut test), [(a, 1)], "the emptied bag went");
}
