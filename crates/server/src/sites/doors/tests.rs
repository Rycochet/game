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
    assert!(answers.contains(&ServerMessage::DoorRefused { x, y }));
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
    assert!(answers.contains(&ServerMessage::DoorRefused { x, y }));
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
