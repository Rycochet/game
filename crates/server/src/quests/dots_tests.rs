//! The maps' quest dots: a giver's door in their town, stairs towards the
//! level a step is done on (only those remembered), and none on the world map.

use shared::dungeon::Cell;
use shared::poi::PoiKind;
use shared::protocol::ClientMessage;
use shared::site::Exit;

use super::dots::dots_here;
use super::tests::{NOW, a_town};
use crate::connections::tests::TestClient;
use crate::sites::dev_enter;
use crate::storage::Progress;

#[test]
fn a_quest_to_hand_in_has_a_dot_on_its_givers_door_in_their_town() {
    let mut test = TestClient::new();
    let (town, quest) = a_town(&mut test);
    let done = (quest.steps.iter()).map(|step| step.count.unwrap_or(1));
    let progress = Progress {
        at: NOW,
        counts: done.collect(),
        ready: true,
        escort: None,
    };
    test.client
        .character
        .quests
        .taken
        .insert(quest.id, progress);
    // None on the world map: the client has the town.
    assert!(dots_here(&test.client, &mut test.world).is_empty());
    let poi = test.world.pois.in_chunk(town).expect("a town");
    test.client.character.world_position = (poi.x, poi.y);
    test.send(&[ClientMessage::EnterSite { chunk: town }]);
    let dots = dots_here(&test.client, &mut test.world);
    assert_eq!(dots.len(), 1);
    assert_eq!(dots[0].id, quest.id);
    let &[(x, y)] = dots[0].cells.as_slice() else {
        panic!("one door: {:?}", dots[0].cells);
    };
    let visit = test.client.visit.as_ref().unwrap();
    assert_eq!(visit.cells().cell(x, y), Cell::BuildingDoor);
}

#[test]
fn stairs_towards_a_level_are_those_remembered_that_lead_nearer() {
    let mut test = TestClient::new();
    // A cave of more than one level.
    let deep = (0..50).find(|&seed| {
        dev_enter(
            &mut test.client,
            &mut test.world,
            PoiKind::RockCave,
            seed,
            0,
            0,
        );
        let visit = test.client.visit.as_ref().expect("in the cave");
        visit.site().unwrap().levels() > 2
    });
    assert!(deep.is_some(), "a cave of three levels or more");
    let visit = test.client.visit.as_ref().unwrap();
    let last = visit.site().unwrap().levels() - 1;
    let down = visit.stairs_towards(last, |_| true);
    assert!(!down.is_empty());
    for cell in &down {
        let stairs = visit.stairs_at(*cell).unwrap();
        assert_eq!(stairs.to, Exit::Level(1));
    }
    assert!(visit.stairs_towards(last, |_| false).is_empty());
    // Already there: nowhere nearer.
    assert!(visit.stairs_towards(0, |_| true).is_empty());
}
