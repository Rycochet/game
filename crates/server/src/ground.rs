//! What lies on the ground of a site's levels: dropped by players, to be picked up
//! by anyone standing on the same cell. Never on the world map. Kept in
//! `LevelSave::ground`, so everyone on a level shares it.

use bevy::prelude::*;
use shared::energy::{dressing_cost, handling_cost};
use shared::ground::Load;
use shared::id::Id;
use shared::item::{Holder, Item, Rotation};
use shared::poi::PoiKind;
use shared::protocol::ServerMessage;

use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::storage::{LevelSave, SiteId};

/// Something a client asked to do with the ground, in turn with its steps: where the
/// player is decides what it means.
#[derive(Clone, Copy, Debug)]
pub(crate) enum GroundAction {
    Drop {
        item: Id,
    },
    Take {
        item: Id,
        to: Holder,
        x: u8,
        y: u8,
        rotation: Rotation,
    },
    Wear {
        item: Id,
    },
    Bin {
        item: Id,
    },
}

/// What a client was last told about the ground under it.
#[derive(Default)]
pub(crate) struct GroundTold {
    /// The cell, and what lay there. `None` if nothing did.
    pile: Option<((i64, i64), Vec<Id>)>,
    /// It guessed at something the server has now answered: it needs telling again,
    /// even if nothing changed.
    again: bool,
    /// It has been told the bags its character remembers on this map.
    bags_told: bool,
}

/// The cell the player stands on, and which level it is on.
fn standing_on(client: &Client) -> Option<(SiteId, PoiKind, u32, (i64, i64))> {
    let visit = client.visit.as_ref()?;
    Some((visit.id(), visit.kind(), visit.index(), visit.position()))
}

/// The level the player is on, to change.
fn level_mut<'a>(world: &'a mut ServerWorld, client: &Client) -> Option<&'a mut LevelSave> {
    let (id, kind, index, _) = standing_on(client)?;
    Some(world.site_mut(id, kind).levels.entry(index).or_default())
}

/// Takes this off the ground under the player. What is left empty goes.
fn take_up(client: &Client, world: &mut ServerWorld, item: Id) -> Option<Item> {
    let (.., at) = standing_on(client)?;
    let level = level_mut(world, client)?;
    let pile = level.ground.get_mut(&at)?;
    let place = pile.iter().position(|one| one.id == item)?;
    let taken = pile.remove(place);
    if pile.is_empty() {
        level.ground.remove(&at);
    }
    Some(taken)
}

/// Puts this on the ground under the player, as the newest there.
fn put_down(client: &Client, world: &mut ServerWorld, item: Item) {
    if let Some((.., at)) = standing_on(client)
        && let Some(level) = level_mut(world, client)
    {
        level.ground.entry(at).or_default().push(item);
    }
}

/// Moves something carried to the ground under the player: what it cost.
fn dropped(client: &mut Client, world: &mut ServerWorld, item: Id) -> Result<u32, &'static str> {
    standing_on(client).ok_or("not in a site")?;
    let cost = client.inventory.put_down_cost(item);
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't put that down")?;
    put_down(client, world, item);
    Ok(cost)
}

/// Something on the ground under the player, as it is: it stays there until `take_up`.
fn lying(client: &Client, world: &mut ServerWorld, item: Id) -> Result<Item, &'static str> {
    let (id, kind, index, at) = standing_on(client).ok_or("not in a site")?;
    let level = world.site(id, kind).levels.get(&index);
    let pile = level.and_then(|level| level.ground.get(&at));
    let found = pile.and_then(|pile| pile.iter().find(|one| one.id == item));
    found.cloned().ok_or("that isn't on the ground here")
}

/// Does what was asked. Whether the client is still there, and what it cost, which
/// is nothing if it was refused. Anything refused is put right by the answers.
pub(crate) fn act(
    client: &mut Client,
    world: &mut ServerWorld,
    action: GroundAction,
) -> (bool, u32) {
    let done = match action {
        GroundAction::Drop { item } => dropped(client, world, item),
        GroundAction::Take {
            item,
            to,
            x,
            y,
            rotation,
        } => lying(client, world, item).and_then(|found| {
            let grams = found.total_weight();
            client
                .inventory
                .add(found, to, x, y, rotation)
                .map_err(|_| "it can't go there")?;
            take_up(client, world, item);
            Ok(handling_cost(grams))
        }),
        GroundAction::Wear { item } => lying(client, world, item).and_then(|found| {
            let grams = found.total_weight();
            client
                .inventory
                .wear_new(found)
                .map_err(|_| "it can't go on")?;
            take_up(client, world, item);
            Ok(dressing_cost(grams) + handling_cost(grams))
        }),
        GroundAction::Bin { item } => lying(client, world, item).map(|found| {
            client.inventory.to_bin_new(found);
            take_up(client, world, item);
            0
        }),
    };
    let cost = match done {
        Ok(cost) => {
            client.unsaved = true;
            cost
        }
        Err(why) => {
            warn!("Refused {action:?}: {why}");
            0
        }
    };
    client.update_speed(world.action_clock.ticks());
    client.ground_told.again = true;
    let sent = client
        .link
        .send(&ServerMessage::Inventory(client.inventory.clone()));
    (sent, cost)
}

/// Tells each player what is under them, when it isn't what they were last told. A
/// client that has gone is noticed when its messages are next read.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld) {
    for client in clients {
        if client.phase != Phase::Playing {
            continue;
        }
        let Some((id, kind, index, at)) = standing_on(client) else {
            continue;
        };
        let level = world.site(id, kind).levels.get(&index);
        let items = level.and_then(|level| level.ground.get(&at));
        let pile = items.map(|items| (at, items.iter().map(|item| item.id).collect()));
        if pile != client.ground_told.pile || client.ground_told.again {
            let items = items.cloned().unwrap_or_default();
            client.link.send(&ServerMessage::GroundItems {
                x: at.0,
                y: at.1,
                items,
            });
            client.ground_told.pile = pile;
            client.ground_told.again = false;
        }
        tell_bags(client, world);
    }
}

/// Tells a client about bags: on arriving, those its character remembers; after that
/// only what has just been seen to differ, which is all its character remembers too.
fn tell_bags(client: &mut Client, world: &mut ServerWorld) {
    let Some((id, kind, index, _)) = standing_on(client) else {
        return;
    };
    let level = world.site(id, kind).levels.get(&index);
    let ground = level.map(|level| &level.ground);
    let piles: Vec<((i64, i64), Load)> = ground
        .into_iter()
        .flatten()
        .map(|(&cell, items)| (cell, Load::of(items)))
        .collect();
    let levels = client.character.bags.get(&id);
    let mut known = levels
        .and_then(|levels| levels.get(&index))
        .cloned()
        .unwrap_or_default();
    if !client.ground_told.bags_told {
        client.ground_told.bags_told = true;
        for (&(x, y), &load) in &known {
            client.link.send(&ServerMessage::GroundSeen { x, y, load });
        }
    }
    let Some(visit) = client.visit.as_mut() else {
        return;
    };
    let mut changed = false;
    for &(cell, load) in &piles {
        if known.get(&cell) != Some(&load) && visit.can_see(cell.0, cell.1) {
            known.insert(cell, load);
            changed = true;
            let (x, y) = cell;
            client.link.send(&ServerMessage::GroundSeen { x, y, load });
        }
    }
    let gone: Vec<(i64, i64)> = known
        .keys()
        .filter(|cell| !piles.iter().any(|(pile, _)| pile == *cell))
        .filter(|cell| visit.can_see(cell.0, cell.1))
        .copied()
        .collect();
    for (x, y) in gone {
        known.remove(&(x, y));
        changed = true;
        client.link.send(&ServerMessage::GroundGone { x, y });
    }
    if changed {
        let levels = client.character.bags.entry(id).or_default();
        levels.insert(index, known);
        client.unsaved = true;
    }
}

#[cfg(test)]
mod tests {
    use shared::energy::site_step_cost;
    use shared::item::{Inventory, ItemKind, Quality};
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
}
