use crate::ServerWorld;
use crate::storage::{CellMasks, SiteId};
use shared::dungeon::{Cell, Level, Sight};
use shared::id::Id;
use shared::item::{Item, Quality};
use shared::movement::{Step, step_on_level};
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::rng::Seed;
use shared::shop::sells;
use shared::site::{Exit, Site, leads_out};
use std::collections::VecDeque;

use shared::world::WorldSeed;

use super::*;
use crate::connections::tests::TestClient;
use crate::storage::Storage;
use crate::storage::tests::TestFolder;

/// A village, and a shop in it with what is beside its door.
fn in_a_village() -> (TestClient, Id, (i64, i64)) {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Village,
        seed: 4,
        level: 0,
        ruin: 0,
    }]);
    let visit = test.client.visit.as_ref().unwrap();
    let shop = visit
        .level
        .buildings
        .iter()
        .find(|b| !sells(b.look.kind).is_empty());
    let look = &shop.expect("a village with a shop").look;
    let front = (look.door.0 + look.faces.0, look.door.1 + look.faces.1);
    let saved = test.world.site(visit.id).levels.get(&visit.index);
    let keeper = saved.unwrap().buildings[&look.id].keeper.expect("a keeper");
    (test, keeper, front)
}

fn stock_in(answers: &[ServerMessage]) -> Option<Vec<shared::item::Item>> {
    answers.iter().find_map(|answer| match answer {
        ServerMessage::TradeStock { items, .. } => Some(items.clone()),
        _ => None,
    })
}

#[test]
fn a_shop_only_opens_from_beside_its_door_and_keeps_its_stock() {
    let (mut test, with, front) = in_a_village();
    let open = ClientMessage::OpenTrade { with };
    let far = ClientMessage::DevTeleport {
        x: front.0 + 40,
        y: front.1,
    };
    test.send(&[far]);
    assert_eq!(stock_in(&test.send(std::slice::from_ref(&open))), None);
    let near = ClientMessage::DevTeleport {
        x: front.0,
        y: front.1,
    };
    test.send(&[near]);
    let first = stock_in(&test.send(std::slice::from_ref(&open))).expect("it opens");
    assert!(!first.is_empty());
    assert!(first.iter().all(|item| item.made_at.is_some()));
    let again = stock_in(&test.send(std::slice::from_ref(&open))).expect("it opens");
    assert_eq!(again, first, "no time has passed");
}

#[test]
fn a_keeper_stays_and_takes_over_what_a_shop_had() {
    let (mut test, keeper, _) = in_a_village();
    let visit = test.client.visit.as_ref().unwrap();
    let (id, index) = (visit.id, visit.index);
    let building = {
        let level = &test.world.site(id).levels[&index];
        let mut buildings = level.buildings.iter();
        *buildings.find(|(_, b)| b.keeper == Some(keeper)).unwrap().0
    };
    let old = Item::new(Id::for_test(9), shared::item::ItemKind::Apple);
    let level = test.world.site_mut(id).levels.entry(index).or_default();
    level
        .buildings
        .get_mut(&building)
        .unwrap()
        .stock
        .push(old.clone());
    let shops = vec![building];
    let new = NewKeepers { id, index, shops };
    assert_eq!(keepers_of(&mut test.world, &new), vec![(building, keeper)]);
    let level = &test.world.site(id).levels[&index];
    assert!(level.buildings[&building].stock.is_empty());
    assert!(level.records[&keeper].carried.contains(&old));
}

#[test]
fn what_is_given_to_a_keeper_joins_the_stock_from_beside_the_door_only() {
    let (mut test, with, front) = in_a_village();
    let coin = ClientMessage::DevGiveItem {
        kind: shared::item::ItemKind::Coin,
        size: None,
        magical: false,
        quality: Quality::Common,
        set: false,
        favour: None,
        seed: Some(1),
        count: 1,
        identical: true,
        request: shared::item_form::RollRequest::NONE,
        to: None,
    };
    test.send(&[coin]);
    let item = test.client.inventory.items.last().unwrap().item.id;
    let give = ClientMessage::GiveItem { item, to: with };
    test.send(&[ClientMessage::DevTeleport {
        x: front.0 + 40,
        y: front.1,
    }]);
    assert_eq!(stock_in(&test.send(std::slice::from_ref(&give))), None);
    assert!(test.client.inventory.find(item).is_some(), "still theirs");
    test.send(&[ClientMessage::DevTeleport {
        x: front.0,
        y: front.1,
    }]);
    let stock = stock_in(&test.send(&[give])).expect("the stock comes back");
    assert!(stock.iter().any(|one| one.id == item));
    assert!(
        test.client.inventory.find(item).is_none(),
        "no longer theirs"
    );
}

#[test]
fn taking_from_a_keeper_needs_reach_room_and_it_still_being_there() {
    use shared::item::{Holder, Rotation};
    let (mut test, with, front) = in_a_village();
    let near = ClientMessage::DevTeleport {
        x: front.0,
        y: front.1,
    };
    test.send(std::slice::from_ref(&near));
    let open = ClientMessage::OpenTrade { with };
    let stock = stock_in(&test.send(&[open])).unwrap();
    let item = stock[0].id;
    let take = |x: u8| ClientMessage::TakeItem {
        with,
        item,
        to: Holder::Inventory,
        x,
        y: 0,
        rotation: Rotation::None,
    };
    // Clear the inventory, for room.
    test.client.inventory.items.clear();
    // Set directly, as a teleport only goes somewhere walkable.
    test.client.visit.as_mut().unwrap().position =
        shared::spot::Spot::middle_of((front.0 + 40, front.1));
    test.send(&[take(0)]);
    assert!(test.client.inventory.find(item).is_none(), "out of reach");
    test.client.visit.as_mut().unwrap().position = shared::spot::Spot::middle_of(front);
    // Off the end of the inventory.
    test.send(&[take(200)]);
    assert!(test.client.inventory.find(item).is_none(), "no room there");
    let answers = test.send(&[take(0)]);
    assert!(test.client.inventory.find(item).is_some());
    let left = stock_in(&answers).expect("the stock comes back");
    assert!(left.iter().all(|one| one.id != item));
    assert_eq!(left.len(), stock.len() - 1);
    // Taken twice, it isn't copied.
    test.client.inventory.items.clear();
    test.send(&[take(0)]);
    assert!(test.client.inventory.find(item).is_none(), "already gone");
}

#[test]
fn something_that_is_not_a_shop_stays_shut() {
    let (mut test, keeper, front) = in_a_village();
    let visit = test.client.visit.as_ref().unwrap();
    let house = visit
        .level
        .buildings
        .iter()
        .find(|b| sells(b.look.kind).is_empty());
    let look = house.expect("a village with a house").look.clone();
    test.send(&[ClientMessage::DevTeleport {
        x: look.door.0 + look.faces.0,
        y: look.door.1 + look.faces.1,
    }]);
    // A keeper of somewhere else, too far from here.
    let open = ClientMessage::OpenTrade { with: keeper };
    assert_eq!(stock_in(&test.send(&[open])), None);
    // Someone who keeps nothing.
    let open = ClientMessage::OpenTrade {
        with: Id::for_test(1),
    };
    assert_eq!(stock_in(&test.send(&[open])), None);
    assert_ne!(front, look.door);
}

#[test]
fn a_shops_stock_changes_as_time_passes() {
    let (mut test, with, front) = in_a_village();
    test.send(&[ClientMessage::DevTeleport {
        x: front.0,
        y: front.1,
    }]);
    let open = ClientMessage::OpenTrade { with };
    let first = stock_in(&test.send(std::slice::from_ref(&open))).unwrap();
    let later = test.world.minutes() + 200_000.0;
    test.send(&[ClientMessage::DevSetWorldClock {
        minutes: Some(later),
        rate: None,
    }]);
    let second = stock_in(&test.send(std::slice::from_ref(&open))).unwrap();
    let ids = |items: &[shared::item::Item]| -> Vec<shared::id::Id> {
        items.iter().map(|item| item.id).collect()
    };
    assert!(
        ids(&second).iter().all(|id| !ids(&first).contains(id)),
        "past the limit everything has been replaced"
    );
}

/// A town arrives whole and remembered, with its buildings, and nothing
/// of it is kept in the character's memory.
#[test]
fn towns_are_known_at_once() {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Village,
        seed: 4,
        level: 0,
        ruin: 0,
    }]);
    let site = Site::new(PoiKind::Village, Seed::new(4)).level(0);
    let looks = answers.iter().find_map(|message| match message {
        ServerMessage::Buildings(looks) => Some(looks),
        _ => None,
    });
    assert_eq!(looks.unwrap().len(), site.buildings.len());
    let mut cells = 0;
    for answer in &answers {
        if let ServerMessage::LevelCells(chunk) = answer {
            assert!(chunk.remembered);
            cells += chunk.cells.len() as i64;
        }
    }
    let (_, _, width, height) = site.level.bounds();
    assert_eq!(cells, width * height);
    let memory = test.client.character.memory.values();
    assert!(
        memory
            .flat_map(|levels| levels.values())
            .all(|m| m.is_empty())
    );
}

/// A castle with more than one floor.
fn castle_seed() -> u64 {
    (1..)
        .find(|&seed| Site::new(PoiKind::Castle, Seed::new(seed)).levels() > 1)
        .unwrap()
}

/// Enters a test site (a castle with stairs) and returns the answers.
fn enter(test: &mut TestClient, level: u32) -> Vec<ServerMessage> {
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        level,
        ruin: 0,
    }])
}

/// The steps of a shortest walk from `from` to `to` on `level`.
pub(super) fn path(level: &Level, from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
    let mut came_from = std::collections::BTreeMap::from([(from, from)]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        if at == to {
            break;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(step) = Step::new(dx, dy) else {
                    continue;
                };
                if let Some(next) = step_on_level(level, at, step)
                    && !came_from.contains_key(&next)
                {
                    came_from.insert(next, at);
                    queue.push_back(next);
                }
            }
        }
    }
    let mut steps = Vec::new();
    let mut at = to;
    while at != from {
        let back = came_from[&at];
        steps.push((at.0 - back.0, at.1 - back.1));
        at = back;
    }
    steps.reverse();
    steps
}

/// Walks the player to `to` on their level.
/// Doors on the way are opened first (Open All Doors), as walking into one
/// opens it rather than stepping.
pub(super) fn walk(test: &mut TestClient, to: (i64, i64)) -> Vec<ServerMessage> {
    let open = shared::protocol::DoorAsk::DevAll { open: true };
    crate::sites::door_act(&mut test.client, &mut test.world, open);
    let visit = test.client.visit.as_ref().unwrap();
    let steps = path(&visit.level.level, visit.position(), to);
    test.steps(&steps)
}

/// A walkable cell next to the player, and the step there.
fn walkable_neighbour(test: &TestClient) -> (i64, i64) {
    let visit = test.client.visit.as_ref().unwrap();
    [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ]
    .into_iter()
    .find(|&(dx, dy)| {
        let step = Step::new(dx, dy).unwrap();
        step_on_level(&visit.level.level, visit.position(), step).is_some()
    })
    .expect("room to move")
}

fn cells(answers: &[ServerMessage]) -> Vec<(i64, i64, Cell, bool)> {
    answers
        .iter()
        .filter_map(|answer| match answer {
            ServerMessage::LevelCells(chunk) => Some(chunk),
            _ => None,
        })
        .flat_map(|chunk| {
            chunk
                .cells()
                .map(|(x, y, cell)| (x, y, cell, chunk.remembered))
        })
        .collect()
}

#[test]
fn entering_sends_the_level_then_what_is_in_sight() {
    let mut test = TestClient::new();
    let answers = enter(&mut test, 0);
    let ServerMessage::EnterLevel(info) = &answers[0] else {
        panic!("{:?}", answers[0]);
    };
    let sent = cells(&answers);
    assert!(!sent.is_empty());
    // The cell the player arrives on, and nothing remembered yet.
    let (ax, ay) = info.arrival.cell();
    assert!(sent.iter().any(|&(x, y, _, _)| (x, y) == (ax, ay)));
    assert!(sent.iter().all(|&(_, _, _, remembered)| !remembered));
    // Never the whole level, and never a secret door.
    let (_, _, width, height) = info.bounds;
    assert!(sent.len() < (width * height) as usize);
    assert!(sent.iter().all(|&(_, _, cell, _)| cell != Cell::SecretDoor));
}

#[test]
fn truesight_shows_secret_doors_and_only_while_it_is_on() {
    let has_door = |answers: &[ServerMessage]| {
        cells(answers)
            .iter()
            .any(|&(_, _, cell, _)| cell == Cell::SecretDoor)
    };
    let found = (0..100).any(|seed| {
        let mut test = TestClient::new();
        test.send(&[
            ClientMessage::DevEnterSite {
                kind: PoiKind::Castle,
                seed,
                level: 0,
                ruin: 0,
            },
            ClientMessage::DevRevealMap,
        ]);
        let answers = test.send(&[ClientMessage::DevTruesight(true)]);
        if !has_door(&answers) {
            return false;
        }
        let answers = test.send(&[ClientMessage::DevTruesight(false)]);
        assert!(!cells(&answers).is_empty(), "sent again");
        assert!(!has_door(&answers));
        true
    });
    assert!(found, "no level with a secret door to try this on");
}

#[test]
fn cells_are_sent_once_and_remembered_next_time() {
    let mut test = TestClient::new();
    let first = cells(&enter(&mut test, 0));
    // A step there and back: back where everything was sent already.
    let (dx, dy) = walkable_neighbour(&test);
    test.steps(&[(dx, dy)]);
    assert!(cells(&test.steps(&[(-dx, -dy)])).is_empty());
    // Coming back: what was seen arrives remembered, before anything new.
    let again = cells(&enter(&mut test, 0));
    assert!(again.iter().any(|&(_, _, _, remembered)| remembered));
    let remembered: Vec<_> = again.iter().filter(|c| c.3).collect();
    assert!(remembered.len() <= first.len());
}

#[test]
fn revealing_sends_the_whole_level_remembered_and_keeps_the_place() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let start = test.client.visit.as_ref().unwrap().position();
    let answers = test.send(&[ClientMessage::DevRevealMap]);
    let ServerMessage::EnterLevel(info) = &answers[0] else {
        panic!("{:?}", answers[0]);
    };
    assert_eq!(info.arrival.cell(), start);
    assert!(!info.entering_site);
    let (_, _, width, height) = info.bounds;
    let sent = cells(&answers);
    let remembered = sent.iter().filter(|cell| cell.3).count();
    assert_eq!(remembered, (width * height) as usize);
    assert!(sent.iter().all(|&(_, _, cell, _)| cell != Cell::SecretDoor));
}

#[test]
fn forgetting_a_level_leaves_only_what_is_in_sight() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    test.send(&[ClientMessage::DevRevealMap]);
    let answers = test.send(&[ClientMessage::DevForgetMap]);
    assert!(matches!(answers[0], ServerMessage::EnterLevel(_)));
    let sent = cells(&answers);
    assert!(sent.iter().all(|cell| !cell.3), "nothing remembered");
    let ServerMessage::EnterLevel(info) = &answers[0] else {
        unreachable!();
    };
    let (_, _, width, height) = info.bounds;
    assert!(sent.len() < (width * height) as usize);
}

#[test]
fn dev_teleports_go_through_walls_but_not_into_them() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let level = test.client.visit.as_ref().unwrap().level.level.clone();
    let start = test.client.visit.as_ref().unwrap().position();
    // Somewhere walkable, more than a step away.
    let (min_x, min_y, width, height) = level.bounds();
    let far = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .find(|&(x, y)| level.is_walkable(x, y) && (x - start.0).abs() > 3)
        .unwrap();
    let answers = test.send(&[ClientMessage::DevTeleport { x: far.0, y: far.1 }]);
    // Told in units: the cell's middle.
    let (ux, uy) = (far.0 * 240, far.1 * 240);
    assert_eq!(answers[0], ServerMessage::Teleported { x: ux, y: uy });
    assert_eq!(test.client.visit.as_ref().unwrap().position(), far);
    // Not into a wall.
    let (wx, wy) = (min_x - 1, min_y - 1);
    assert!(
        test.send(&[ClientMessage::DevTeleport { x: wx, y: wy }])
            .is_empty()
    );
    // Back to the start leaves the site.
    let answers = test.send(&[ClientMessage::DevBackToStart]);
    assert!(matches!(answers[0], ServerMessage::OnWorldMap { .. }));
    assert!(test.client.visit.is_none());
}

#[test]
fn steps_into_walls_are_refused() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let visit = test.client.visit.as_ref().unwrap();
    let (x, y) = visit.position();
    let level = visit.level.level.clone();
    let (dx, dy) = [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (1, 1),
        (-1, -1),
        (1, -1),
        (-1, 1),
    ]
    .into_iter()
    .find(|&(dx, dy)| !level.is_walkable(x + dx, y + dy))
    .expect("a wall nearby");
    let answers = test.steps(&[(dx, dy)]);
    assert_eq!(test.client.visit.as_ref().unwrap().position(), (x, y));
    // The client is told where the player still is, in units.
    assert!(answers.contains(&ServerMessage::Position {
        map: test.client_map(),
        step: 1,
        x: x * 240,
        y: y * 240
    }));
}

#[test]
fn stairs_and_the_way_out_work() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    // A castle has floors above: take the first stairs.
    let visit = test.client.visit.as_ref().unwrap();
    let stairs = visit.level.stairs[0];
    walk(&mut test, (stairs.x, stairs.y));
    assert_eq!(
        test.client.visit.as_ref().unwrap().position(),
        (stairs.x, stairs.y)
    );
    let answers = test.send(&[ClientMessage::UseExit]);
    let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
        panic!("{answers:?}");
    };
    assert_eq!(Exit::Level(info.index), stairs.to);
    // Back down, then out through the entrance's ground.
    test.send(&[ClientMessage::UseExit]);
    assert_eq!(test.client.visit.as_ref().unwrap().index, 0);
    let entrance = test.client.visit.as_ref().unwrap().level.entrance.unwrap();
    let level = test.client.visit.as_ref().unwrap().level.level.clone();
    // Walk south from the entrance to the edge of the level.
    let mut out = entrance;
    while !leads_out(&level, out.0, out.1) && level.is_walkable(out.0, out.1 + 1) {
        out.1 += 1;
    }
    walk(&mut test, out);
    let answers = test.send(&[ClientMessage::UseExit]);
    assert!(matches!(
        answers[..],
        [ServerMessage::OnWorldMap { .. }, ..]
    ));
    assert!(test.client.visit.is_none());
    // Steps taken in the site before leaving don't count out here.
    let stale = ClientMessage::Step {
        map: test.client_map() - 1,
        number: 99,
        step: Step::new(1, 0).unwrap(),
    };
    let at = test.client.world_position();
    test.send(&[stale]);
    assert_eq!(test.client.world_position(), at);
}

#[test]
fn coming_into_a_site_tells_what_is_remembered_of_its_other_levels() {
    let mut test = TestClient::new();
    let remembered = |answers: &[ServerMessage]| -> Vec<u32> {
        let levels = answers.iter().filter_map(|message| match message {
            ServerMessage::RememberedLevel(level) => Some(level.index),
            _ => None,
        });
        levels.collect()
    };
    // Nothing yet, and nothing of the level the player is going to.
    let answers = enter(&mut test, 0);
    assert_eq!(remembered(&answers), [] as [u32; 0]);
    let answers = enter(&mut test, 0);
    assert_eq!(remembered(&answers), [] as [u32; 0]);

    enter(&mut test, 1);
    let answers = enter(&mut test, 0);
    let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
        panic!("{answers:?}");
    };
    assert!(info.entering_site);
    assert_eq!(remembered(&answers), [1]);
    let level = answers.iter().find_map(|message| match message {
        ServerMessage::RememberedLevel(level) => Some(level),
        _ => None,
    });
    let chunks = &level.expect("found above").chunks;
    assert!(chunks.iter().any(|chunk| chunk.remembered));

    // Taking stairs tells nothing new: the client keeps what it has.
    let visit = test.client.visit.as_ref().unwrap();
    let stairs = visit.level.stairs[0];
    walk(&mut test, (stairs.x, stairs.y));
    let answers = test.send(&[ClientMessage::UseExit]);
    let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
        panic!("{answers:?}");
    };
    assert!(!info.entering_site);
    assert_eq!(remembered(&answers), [] as [u32; 0]);
}

/// A world saved in `folder`.
fn saved_world(folder: &TestFolder) -> ServerWorld {
    let seed = WorldSeed::new(1);
    ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap()
}

#[test]
fn coming_back_resumes_the_visit_with_its_memory() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    let first = cells(&enter(&mut test, 1));
    let position = test.client.visit.as_ref().unwrap().position();
    test.leave();

    // Another connection, even another server, on the same data.
    let mut test = TestClient::of(saved_world(&folder));
    let answers = test.send(&[]);
    assert!(answers.is_empty(), "the welcome was read: {answers:?}");
    let visit = test.client.visit.as_ref().expect("back in the site");
    assert_eq!((visit.index, visit.position()), (1, position));
    // What was seen came back remembered.
    let memory = &test.client.character.memory;
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    };
    let seen: u32 = memory[&id][&1].values().map(|m| m.count_ones()).sum();
    assert!(seen > 0 && seen as usize <= first.len());
}

#[test]
fn welcome_sends_the_level_after_the_world() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    enter(&mut test, 0);
    let test = TestClient::of(test.leave());
    assert!(matches!(
        test.welcome[..],
        [
            ServerMessage::Welcome { x: 70, y: -3, .. },
            ServerMessage::Sheet(_),
            ServerMessage::Vitals { .. },
            ServerMessage::ActionBar(_),
            ServerMessage::VisitedGates(_),
            ServerMessage::Inventory(_),
            ServerMessage::Difficulty { .. },
            ServerMessage::EnterLevel(_),
            ..
        ]
    ));
}

#[test]
fn changes_are_applied_over_the_generated_level() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let (x, y) = test.client.visit.as_ref().unwrap().position();
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    };
    let level = test.world.site_mut(id).levels.entry(0);
    level.or_default().changes.insert((x + 1, y), Cell::Water);
    enter(&mut test, 0);
    let level = &test.client.visit.as_ref().unwrap().level.level;
    assert_eq!(level.cell(x + 1, y), Cell::Water);
}

#[test]
fn dev_tools_change_cells_and_keep_the_change() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let (x, y) = test.client.visit.as_ref().unwrap().position();
    let (dx, dy) = walkable_neighbour(&test);
    let (nx, ny) = (x + dx, y + dy);
    let answers = test.send(&[ClientMessage::DevSetCell {
        x: nx,
        y: ny,
        cell: Cell::Window,
    }]);
    assert!(
        cells(&answers)
            .iter()
            .any(|&sent| (sent.0, sent.1, sent.2) == (nx, ny, Cell::Window))
    );
    // Not where the player stands.
    test.send(&[ClientMessage::DevSetCell {
        x,
        y,
        cell: Cell::Wall,
    }]);
    // Windows can't be stepped on; and the change is there next time.
    enter(&mut test, 0);
    let level = &test.client.visit.as_ref().unwrap().level.level;
    assert_eq!(level.cell(nx, ny), Cell::Window);
    assert!(level.is_walkable(x, y));
    assert!(!level.is_walkable(nx, ny));
}

#[test]
fn walls_arrive_with_what_shapes_them() {
    let mut test = TestClient::new();
    let sent = cells(&enter(&mut test, 0));
    let is_sent = |x, y| sent.iter().any(|&(cx, cy, _, _)| (cx, cy) == (x, y));
    let visit = test.client.visit.as_ref().unwrap();
    let level = &visit.level.level;
    let mut sight = Sight::new(level);
    sight.add_glimpses(level, visit.position().0, visit.position().1, None);
    let seen_wall =
        |&&(x, y, cell, _): &&(i64, i64, Cell, bool)| cell.is_wall() && sight.is_seen(x, y);
    let walls: Vec<_> = sent.iter().filter(seen_wall).collect();
    assert!(!walls.is_empty());
    for &&(x, y, _, _) in &walls {
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            assert!(!level.contains(nx, ny) || is_sent(nx, ny), "{nx}, {ny}");
        }
    }
}

#[test]
fn sites_keep_the_generator_version_they_were_made_with() {
    let mut test = TestClient::new();
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    };
    let current = test.world.site_mut(id).version;
    enter(&mut test, 0);
    assert_eq!(test.world.site(id).version, current);
    assert_eq!(test.client.character.site_versions[&id], current);
}

fn castle_id() -> SiteId {
    SiteId::Dev {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    }
}

#[test]
fn a_site_whose_version_cannot_run_is_made_anew() {
    let mut test = TestClient::new();
    let id = castle_id();
    let current = test.world.site(id).version;
    test.world.site_mut(id).version = 99;
    let answers = enter(&mut test, 0);
    assert!(matches!(answers[0], ServerMessage::EnterLevel(_)));
    assert_eq!(test.world.site(id).version, current);
}

#[test]
fn memory_of_another_version_of_a_site_is_dropped() {
    let mut test = TestClient::new();
    let id = castle_id();
    enter(&mut test, 0);
    assert!(!test.client.character.memory[&id].is_empty());
    test.send(&[ClientMessage::DevBackToStart]);
    let current = test.world.site(id).version;
    test.client.character.site_versions.insert(id, current + 1);
    let unseen = std::collections::BTreeMap::from([(7, CellMasks::from([((100, 100), 1)]))]);
    test.client.character.memory.insert(id, unseen);
    enter(&mut test, 0);
    assert!(!test.client.character.memory[&id].contains_key(&7));
}

#[test]
fn coming_back_to_a_changed_site_puts_the_player_out_with_the_map_sent() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    enter(&mut test, 1);
    let id = castle_id();
    let current = test.world.site(id).version;
    test.client.character.site_versions.insert(id, current + 1);
    test.client.unsaved = true;

    let test = TestClient::of(test.leave());
    assert!(test.client.visit.is_none());
    assert!(!test.client.character.memory.contains_key(&id));
    assert!(matches!(
        test.welcome[7],
        ServerMessage::OnWorldMap { x: 70, y: -3 }
    ));
    assert!(
        test.welcome
            .iter()
            .any(|answer| matches!(answer, ServerMessage::WorldChunk(_)))
    );
}

#[test]
fn coming_back_to_a_site_that_cannot_run_puts_the_player_out() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    enter(&mut test, 0);
    let id = castle_id();
    // Made, and known, at a version the server has since retired.
    test.client.character.site_versions.insert(id, 99);
    test.client.unsaved = true;
    let mut world = test.leave();
    world.site_mut(id).version = 99;
    world.save_sites();

    let test = TestClient::of(world);
    assert!(test.client.visit.is_none());
    assert!(matches!(test.welcome[7], ServerMessage::OnWorldMap { .. }));
}

#[test]
fn generating_a_site_starts_it_afresh_where_the_player_stands() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    let generate = ClientMessage::DevGenerateSite {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    };
    assert!(matches!(
        test.send(std::slice::from_ref(&generate))[0],
        ServerMessage::EnterLevel(_)
    ));
    change_castle(&mut test, 0);
    let seen: u32 = test.client.character.memory[&castle_id()][&0]
        .values()
        .map(|mask| mask.count_ones())
        .sum();
    assert!(seen > 0);
    test.send(&[ClientMessage::DevRevealMap]);

    // Again, from inside it: as new.
    test.send(&[generate]);
    let site = test.world.site(castle_id());
    assert!(site.levels[&0].changes.is_empty());
    let again: u32 = test.client.character.memory[&castle_id()][&0]
        .values()
        .map(|mask| mask.count_ones())
        .sum();
    assert_eq!(again, seen);

    // Out of it is where the player stood.
    assert_eq!(test.client.world_position(), (70, -3));
}

#[test]
fn generating_a_kind_that_cannot_be_ruined_ignores_the_ruin() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevGenerateSite {
        kind: PoiKind::RedTent,
        seed: 3,
        ruin: 5,
    }]);
    let id = SiteId::Dev {
        kind: PoiKind::RedTent,
        seed: 3,
        ruin: 0,
    };
    assert!(test.client.character.memory.contains_key(&id));
}

/// A player's change on a level of the castle, saved.
fn change_castle(test: &mut TestClient, level: u32) {
    let save = test.world.site_mut(castle_id());
    let level = save.levels.entry(level).or_default();
    level.changes.insert((1, 1), Cell::Floor);
    test.world.save_sites();
}

#[test]
fn forgetting_on_the_server_deletes_the_save_and_keeps_the_player_there() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    enter(&mut test, 1);
    change_castle(&mut test, 1);
    let position = test.client.visit.as_ref().unwrap().position();

    let answers = test.send(&[ClientMessage::DevForgetMapOnServer]);
    assert!(matches!(answers[0], ServerMessage::EnterLevel(_)));
    let visit = test.client.visit.as_ref().unwrap();
    assert_eq!((visit.index, visit.position()), (1, position));
    let site = test.world.site(castle_id());
    assert!(site.levels[&1].changes.is_empty());
}

#[test]
fn forgetting_the_world_map_forgets_every_site() {
    let folder = TestFolder::new();
    let mut test = TestClient::of(saved_world(&folder));
    enter(&mut test, 0);
    change_castle(&mut test, 0);
    test.send(&[ClientMessage::DevBackToStart]);
    test.world.save_sites();
    let id = castle_id();

    // Only the character's memory...
    test.send(&[ClientMessage::DevForgetMap]);
    assert!(test.client.character.memory.is_empty());
    assert!(!test.world.site(id).levels[&0].changes.is_empty());

    // ...and the server's, on disk too.
    let answers = test.send(&[ClientMessage::DevForgetMapOnServer]);
    assert_eq!(answers[0], ServerMessage::ForgetWorldMap);
    assert!(test.world.storage.load_site(id).unwrap().is_none());
    assert!(test.world.site(id).levels.is_empty());
}

#[test]
fn only_stairs_up_into_the_open_are_open_to_the_sky() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: PoiKind::Grave,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    let visit = test.client.visit.as_ref().unwrap();
    let out = visit
        .level
        .stairs
        .iter()
        .position(|stairs| stairs.to == Exit::WorldMap)
        .unwrap();
    assert!(visit.stairs[out].open_to_sky);

    // A castle's stairs lead to more castle.
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.stairs.iter().all(|stairs| !stairs.open_to_sky));
}

#[test]
fn levels_record_when_they_were_last_visited() {
    let mut test = TestClient::new();
    enter(&mut test, 0);
    let id = SiteId::Dev {
        kind: PoiKind::Castle,
        seed: castle_seed(),
        ruin: 0,
    };
    let mut world = test.leave();
    let last = world.site(id).levels[&0].last_visit;
    assert!(last > 0);
}

#[test]
fn nearby_cells_stop_at_walls() {
    let level = Level::test_room();
    let near = nearby_cells(&level, (-3, 0), 2);
    // The wall at x = -2 (y = -1..=1) is in the way east: only round
    // its ends, two steps away.
    assert!(near.iter().all(|&(x, _)| x <= -2));
    assert!(near.contains(&(-2, -2)) && !near.contains(&(-1, 0)));
    assert!(near.iter().all(|&(x, y)| level.is_walkable(x, y)));
    assert!(near.contains(&(-5, 0)) && near.contains(&(-3, 2)));
    assert!(!near.contains(&(-6, 0)));
}
