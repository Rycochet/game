use shared::creature::CreatureKind;
use shared::door::Side;
use shared::dungeon::Cell;
use shared::protocol::{ClientMessage, DoorAsk};

use super::tests::{run, the_mob};
use super::*;
use crate::connections::tests::TestClient;

/// A castle level with every door closed, its first mob made `kind` and put on
/// one side of a door, going home to the other side, with the player away on
/// the world map. The door, and the mob's home.
fn going_home_through_a_door(kind: CreatureKind) -> (TestClient, (i64, i64), (i64, i64)) {
    let mut test = TestClient::new();
    test.enter_a_castle();
    test.send(&[ClientMessage::Door(DoorAsk::DevAll { open: false })]);
    let visit = test.client.visit.as_ref().unwrap();
    let level = visit.cells().clone();
    let floor = |(x, y): (i64, i64)| level.cell(x, y) == Cell::Floor;
    let (door, (a, b)) = (level.swings().map(|(at, _)| at))
        .filter(|&(x, y)| level.cell(x, y) == Cell::Door)
        .map(|at| {
            let sides = if level.opening_runs_east_west(at.0, at.1) {
                (Side::North, Side::South)
            } else {
                (Side::West, Side::East)
            };
            (at, sides)
        })
        .find(|&(at, (a, b))| floor(a.beside(at)) && floor(b.beside(at)))
        .expect("a castle has a door between floors");
    crate::sites::share_doors(std::slice::from_mut(&mut test.client), &mut test.world);
    test.send(&[ClientMessage::DevBackToStart]);
    let tick = test.world.action_clock.ticks();
    let population = test.world.populations.0.values_mut().next().unwrap();
    assert!(population.level.is_shut(door.0, door.1));
    let home = b.beside(door);
    let mob = &mut population.mobs[0];
    mob.kind = kind;
    mob.position = a.beside(door);
    mob.home = home;
    mob.doing = Doing::GoingHome { since: tick };
    let id = mob.id;
    population.due.insert((tick, id));
    (test, door, home)
}

#[test]
fn a_mob_with_hands_opens_a_closed_door_on_its_way() {
    let (mut test, door, home) = going_home_through_a_door(CreatureKind::Human);
    run(&mut test, 400);
    assert_eq!(the_mob(&test).position, home);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(!population.level.is_shut(door.0, door.1));
    // Kept, for whoever comes next.
    let saved = test
        .world
        .sites
        .values()
        .flat_map(|site| site.levels.values());
    assert!(
        saved
            .flat_map(|level| level.doors.get(&door))
            .any(|d| !d.swing.is_shut())
    );
}

#[test]
fn a_mob_without_hands_is_stopped_by_a_closed_door() {
    // (Unseen, it may still be put back home at last: the door stays shut.)
    let (mut test, door, _) = going_home_through_a_door(CreatureKind::Wolf);
    run(&mut test, 400);
    let population = test.world.populations.0.values().next().unwrap();
    assert!(population.level.is_shut(door.0, door.1));
}
