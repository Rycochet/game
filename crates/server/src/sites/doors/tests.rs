use shared::door::{Side, Swing};
use shared::dungeon::Cell;
use shared::protocol::{ClientMessage, DoorAsk, ServerMessage};

use crate::connections::tests::TestClient;

/// Into a castle, standing beside one of its doors (north or west of it, with
/// floor across it): the door, and its wall's way (east-west if `true`).
fn beside_a_door(test: &mut TestClient) -> ((i64, i64), bool) {
    test.enter_a_castle();
    let visit = test.client.visit.as_ref().unwrap();
    let level = &visit.level.level;
    let floor = |(x, y): (i64, i64)| level.cell(x, y) == Cell::Floor;
    let (door, runs) = (visit.doors.keys().copied())
        .filter(|&(x, y)| level.cell(x, y) == Cell::Door)
        .map(|(x, y)| ((x, y), level.opening_runs_east_west(x, y)))
        .find(|&(at, runs)| {
            let (a, b) = if runs {
                (Side::North, Side::South)
            } else {
                (Side::West, Side::East)
            };
            floor(a.beside(at)) && floor(b.beside(at))
        })
        .expect("a castle has a door between floors");
    let from = if runs { Side::North } else { Side::West };
    let (x, y) = from.beside(door);
    test.send(&[ClientMessage::DevTeleport { x, y }]);
    (door, runs)
}

fn swing_of(test: &TestClient, at: (i64, i64)) -> Swing {
    test.client.visit.as_ref().unwrap().doors[&at].door.swing
}

fn ask(test: &mut TestClient, ask: DoorAsk) -> Vec<ServerMessage> {
    test.send(&[ClientMessage::Door(ask)])
}

#[test]
fn a_closed_door_beside_the_player_opens_away_from_them_and_is_kept() {
    let mut test = TestClient::new();
    let (door, runs) = beside_a_door(&mut test);
    ask(&mut test, DoorAsk::DevAll { open: false });
    assert_eq!(swing_of(&test, door), Swing::Closed);
    let answers = ask(
        &mut test,
        DoorAsk::Open {
            x: door.0,
            y: door.1,
        },
    );
    let away = if runs { Side::South } else { Side::East };
    assert_eq!(swing_of(&test, door), Swing::Open(away));
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.level.level.is_walkable(door.0, door.1));
    // Kept for everyone, and told to the client with its cells.
    let saved = &test.world.site(visit.id).levels[&visit.index].doors;
    assert_eq!(saved[&door].swing, Swing::Open(away));
    let told = answers.iter().any(|answer| match answer {
        ServerMessage::LevelCells(chunk) => (chunk.doors.iter())
            .any(|look| (look.x, look.y) == door && look.swing == Swing::Open(away)),
        _ => false,
    });
    assert!(told, "{answers:?}");
    // And remembered as seen.
    let memory = &test.client.character.doors[&visit.id][&visit.index];
    assert_eq!(memory[&door], Swing::Open(away));
}

#[test]
fn a_door_not_beside_the_player_is_refused() {
    let mut test = TestClient::new();
    let (door, _) = beside_a_door(&mut test);
    ask(&mut test, DoorAsk::DevAll { open: false });
    let (x, y) = (door.0 + 5, door.1 + 5);
    let answers = ask(&mut test, DoorAsk::Open { x, y });
    assert!(answers.contains(&ServerMessage::DoorRefused {
        x,
        y,
        locked: false
    }));
    assert_eq!(swing_of(&test, door), Swing::Closed);
}

#[test]
fn closing_leaves_a_door_with_someone_in_its_doorway() {
    let mut test = TestClient::new();
    let (door, runs) = beside_a_door(&mut test);
    ask(&mut test, DoorAsk::DevAll { open: true });
    let back = if runs { (0, -1) } else { (-1, 0) };
    let into = (-back.0, -back.1);
    test.steps(&[into]);
    ask(&mut test, DoorAsk::CloseNear);
    assert!(matches!(swing_of(&test, door), Swing::Open(_)));
    test.steps(&[back]);
    ask(&mut test, DoorAsk::CloseNear);
    assert_eq!(swing_of(&test, door), Swing::Closed);
    let visit = test.client.visit.as_ref().unwrap();
    assert!(!visit.level.level.is_walkable(door.0, door.1));
}

#[test]
fn a_secret_door_must_be_found_and_opens_towards_the_player() {
    let mut test = TestClient::new();
    let (door, runs) = beside_a_door(&mut test);
    let (x, y) = door;
    test.send(&[ClientMessage::DevSetCell {
        x,
        y,
        cell: Cell::SecretDoor,
    }]);
    // It was seen as an ordinary door, which would count as found.
    test.client.character.doors.clear();
    let answers = ask(&mut test, DoorAsk::Open { x, y });
    assert!(answers.contains(&ServerMessage::DoorRefused {
        x,
        y,
        locked: false
    }));
    test.send(&[ClientMessage::DevTruesight(true)]);
    ask(&mut test, DoorAsk::Open { x, y });
    let towards = if runs { Side::North } else { Side::West };
    assert_eq!(swing_of(&test, door), Swing::Open(towards));
}

#[test]
fn a_change_reaches_the_mobs_on_the_level() {
    let mut test = TestClient::new();
    let (door, _) = beside_a_door(&mut test);
    ask(&mut test, DoorAsk::DevAll { open: false });
    let clients = std::slice::from_mut(&mut test.client);
    crate::sites::share_doors(clients, &mut test.world);
    let place = test.client.visit.as_ref().unwrap().population();
    let mobs_level = |test: &TestClient| test.world.populations.0[&place].level().clone();
    assert!(mobs_level(&test).is_shut(door.0, door.1));
    ask(
        &mut test,
        DoorAsk::Open {
            x: door.0,
            y: door.1,
        },
    );
    let clients = std::slice::from_mut(&mut test.client);
    crate::sites::share_doors(clients, &mut test.world);
    assert!(!mobs_level(&test).is_shut(door.0, door.1));
}

/// Puts a locked lock on the door at `at` of the player's level.
fn lock_it(test: &mut TestClient, at: (i64, i64)) {
    let visit = test.client.visit.as_mut().unwrap();
    let door = visit.doors.get_mut(&at).unwrap();
    door.door.swing = Swing::Closed;
    door.door.lock = Some(shared::door::Lock {
        id: shared::id::Id::for_test(77),
        quality: shared::item::Quality::Common,
        locked: true,
    });
    visit.level.level.set_locked(at.0, at.1, true);
}

fn locked(test: &TestClient, at: (i64, i64)) -> bool {
    test.client.visit.as_ref().unwrap().doors[&at]
        .door
        .is_locked()
}

#[test]
fn a_locked_door_opens_only_with_its_key_and_locks_again() {
    let mut test = TestClient::new();
    let (door, _) = beside_a_door(&mut test);
    lock_it(&mut test, door);
    let answers = ask(
        &mut test,
        DoorAsk::Open {
            x: door.0,
            y: door.1,
        },
    );
    let refused = ServerMessage::DoorRefused {
        x: door.0,
        y: door.1,
        locked: true,
    };
    assert!(answers.contains(&refused), "{answers:?}");
    assert_eq!(swing_of(&test, door), Swing::Closed);

    // With its key: unlocked, then opened.
    ask(&mut test, DoorAsk::DevKey);
    let key = Some(shared::item_form::ItemForm::Key(shared::id::Id::for_test(
        77,
    )));
    let carried = test.client.inventory.items.iter();
    assert!(carried.clone().any(|placed| placed.item.form == key));
    ask(
        &mut test,
        DoorAsk::Open {
            x: door.0,
            y: door.1,
        },
    );
    assert!(matches!(swing_of(&test, door), Swing::Open(_)));
    assert!(!locked(&test, door));

    // L closes it, then locks it.
    ask(&mut test, DoorAsk::CloseNear);
    assert_eq!(swing_of(&test, door), Swing::Closed);
    assert!(!locked(&test, door));
    ask(&mut test, DoorAsk::CloseNear);
    assert!(locked(&test, door));
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.level.level.is_locked(door.0, door.1));
}

#[test]
fn locks_are_placed_once_with_their_keys_on_the_levels_mobs() {
    let mut found = 0;
    for seed in 1..12 {
        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevEnterSite {
            kind: shared::poi::PoiKind::Castle,
            seed,
            level: 0,
            ruin: 0,
        }]);
        let visit = test.client.visit.as_ref().unwrap();
        let saved = &test.world.site(visit.id).levels[&visit.index];
        assert!(saved.locks_placed);
        for door in saved.doors.values().filter(|door| door.lock.is_some()) {
            let lock = door.lock.unwrap();
            let key = Some(shared::item_form::ItemForm::Key(lock.id));
            let carried = saved.records.values().flat_map(|record| &record.carried);
            assert!(carried.into_iter().any(|item| item.form == key));
            found += 1;
        }
    }
    assert!(found > 0, "no castle had a lock");
}

#[test]
fn a_step_into_a_closed_door_is_refused_and_the_door_told_again() {
    let mut test = TestClient::new();
    let (door, runs) = beside_a_door(&mut test);
    ask(&mut test, DoorAsk::DevAll { open: false });
    let start = test.client.visit.as_ref().unwrap().position;
    // A client that took it for open steps straight in.
    let step = if runs { (0, 1) } else { (1, 0) };
    let answers = test.steps(&[step]);
    assert_eq!(test.client.visit.as_ref().unwrap().position, start);
    let told = answers.iter().any(|answer| match answer {
        ServerMessage::LevelCells(chunk) => {
            (chunk.doors.iter()).any(|look| (look.x, look.y) == door && look.swing == Swing::Closed)
        }
        _ => false,
    });
    assert!(told, "{answers:?}");
}
