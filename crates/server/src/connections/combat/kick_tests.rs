use shared::ability::Ability;
use shared::creature::CreatureKind;
use shared::door::Swing;
use shared::dungeon::Cell;
use shared::id::Id;
use shared::protocol::{ClientMessage, DoorAsk};
use shared::spot::Spot;

use super::tests::{beside, mob_health, run};
use crate::connections::tests::TestClient;

fn kick(target: Option<Id>) -> ClientMessage {
    ClientMessage::UseAbility {
        ability: Ability::Kick,
        target,
    }
}

fn mob_at(test: &TestClient, id: Id) -> Spot {
    let population = test.world.populations.0.values().next().unwrap();
    population
        .mobs
        .iter()
        .find(|mob| mob.id == id)
        .unwrap()
        .spot
}

#[test]
fn a_kick_shoves_a_creature_half_a_cell_away_and_harms_it() {
    let (mut test, id) = beside(CreatureKind::Kobold);
    let before = mob_at(&test, id);
    let health = mob_health(&test, id).unwrap();
    test.send(&[kick(Some(id))]);
    run(&mut test, 60, false);
    assert_eq!(mob_at(&test, id), before.offset(120, 0));
    assert!(mob_health(&test, id).unwrap() < health);
}

#[test]
fn kicks_break_a_closed_door_open_for_good() {
    let mut test = TestClient::new();
    test.enter_a_castle();
    let visit = test.client.visit.as_ref().unwrap();
    let level = visit.cells();
    let door = (visit.doors_for_test().keys().copied())
        .find(|&(x, y)| {
            level.cell(x, y) == Cell::Door
                && level.opening_runs_east_west(x, y)
                && level.cell(x, y + 1) == Cell::Floor
        })
        .expect("a castle has a door with floor south of it");
    test.send(&[ClientMessage::DevTeleport {
        x: door.0,
        y: door.1 + 1,
    }]);
    test.send(&[ClientMessage::Door(DoorAsk::DevAll { open: false })]);
    let mut kicks = 0;
    let swing = |test: &TestClient| {
        let visit = test.client.visit.as_ref().unwrap();
        visit.doors_for_test()[&door].door.swing
    };
    while swing(&test) == Swing::Closed && kicks < 100 {
        test.send(&[kick(None)]);
        run(&mut test, 120, false);
        kicks += 1;
    }
    assert!(
        matches!(swing(&test), Swing::Broken(shared::door::Side::North)),
        "after {kicks} kicks"
    );
    assert!((3..40).contains(&kicks), "{kicks} kicks");
    // It can't be closed again.
    test.send(&[ClientMessage::Door(DoorAsk::CloseNear)]);
    assert!(matches!(swing(&test), Swing::Broken(_)));
}
