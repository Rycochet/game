//! Dev Areas: made to order, and back from them to where the player was.

use shared::creature::CreatureKind;
use shared::poi::PoiKind;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::{DevArea, DevLayout, DevMobs};
use shared::world::WorldSeed;

use crate::ServerWorld;
use crate::connections::tests::TestClient;
use crate::storage::tests::TestFolder;
use crate::storage::{SiteId, Storage};

fn area(seed: u64, mobs: DevMobs) -> DevArea {
    DevArea {
        size: 32,
        layout: DevLayout::Moat,
        mobs,
        seed,
    }
}

fn wolves(count: u8) -> DevMobs {
    DevMobs::Kind {
        kind: CreatureKind::Wolf,
        count,
    }
}

fn generate(test: &mut TestClient, area: DevArea) -> Vec<ServerMessage> {
    test.send(&[ClientMessage::DevGenerateArea(area)])
}

/// The Dev Area the player is in, and where.
fn in_area(test: &TestClient) -> (DevArea, (i64, i64)) {
    let visit = test.client.visit.as_ref().expect("in a site");
    let SiteId::DevArea(area) = visit.id else {
        panic!("not a Dev Area: {:?}", visit.id);
    };
    (area, visit.position)
}

#[test]
fn an_area_is_entered_on_its_stairs_with_its_mobs() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    let answers = generate(&mut test, area(4, wolves(5)));
    assert!(matches!(answers[0], ServerMessage::EnterLevel(_)));
    let (made, at) = in_area(&test);
    assert_eq!(at, (1, 16), "on the stairs, west middle");
    let mobs = &test.world.populations.0[&(SiteId::DevArea(made), 0)].mobs;
    assert_eq!(mobs.len(), 5);
    assert!(mobs.iter().all(|mob| mob.kind == CreatureKind::Wolf));
}

#[test]
fn sizes_and_counts_are_not_taken_on_trust() {
    let mut test = TestClient::new();
    let asked = DevArea {
        size: 5000,
        ..area(4, wolves(250))
    };
    generate(&mut test, asked);
    let (made, _) = in_area(&test);
    assert_eq!(made, asked.checked());
    assert_eq!(made.size, 128);
}

#[test]
fn the_stairs_lead_back_to_the_world_map_where_the_player_stood() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    generate(&mut test, area(4, DevMobs::None));
    let (made, _) = in_area(&test);
    let answers = test.send(&[ClientMessage::UseExit]);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::OnWorldMap { x: 70, y: -3 }))
    );
    assert!(test.client.visit.is_none());
    assert!(test.client.character.dev_return.is_none());
    let id = SiteId::DevArea(made);
    assert!(!test.client.character.memory.contains_key(&id), "forgotten");
    assert!(!test.world.populations.0.contains_key(&(id, 0)));
}

#[test]
fn another_area_from_inside_one_still_goes_back_to_the_site_before() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevGenerateSite {
        kind: PoiKind::Castle,
        seed: 3,
        ruin: 0,
    }]);
    let castle = test.client.visit.as_ref().unwrap().save();
    generate(&mut test, area(4, DevMobs::None));
    let (first, _) = in_area(&test);
    generate(&mut test, area(5, wolves(2)));
    let (second, _) = in_area(&test);
    assert_ne!(first, second);
    assert!(
        !test
            .client
            .character
            .memory
            .contains_key(&SiteId::DevArea(first))
    );

    test.send(&[ClientMessage::UseExit]);
    let back = test
        .client
        .visit
        .as_ref()
        .expect("back in the castle")
        .save();
    assert_eq!(back, castle);
}

#[test]
fn the_hall_is_gone_back_to() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevToHall]);
    let hall = test.client.character.hall.expect("in the Hall");
    generate(&mut test, area(4, DevMobs::None));
    assert_eq!(test.client.character.hall, None);
    let answers = test.send(&[ClientMessage::UseExit]);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::EnterHall { .. }))
    );
    assert_eq!(test.client.character.hall, Some(hall));
}

#[test]
fn coming_back_to_the_game_in_an_area_goes_back_to_before_it() {
    let folder = TestFolder::new();
    let seed = WorldSeed::new(1);
    let world = ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
    let mut test = TestClient::of(world);
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    generate(&mut test, area(4, wolves(3)));
    in_area(&test);

    let test = TestClient::of(test.leave());
    assert!(test.client.visit.is_none());
    assert!(test.client.character.dev_return.is_none());
    assert_eq!(test.client.world_position(), (70, -3));
    assert!(
        test.welcome
            .iter()
            .any(|answer| matches!(answer, ServerMessage::OnWorldMap { x: 70, y: -3 }))
    );
}

#[test]
fn the_leave_key_goes_back_from_anywhere_in_an_area() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
    generate(&mut test, area(4, DevMobs::None));
    test.send(&[ClientMessage::DevTeleport { x: 20, y: 5 }]);
    assert_eq!(in_area(&test).1, (20, 5), "off the stairs");
    let answers = test.send(&[ClientMessage::LeaveWilderness]);
    assert!(
        answers
            .iter()
            .any(|answer| matches!(answer, ServerMessage::OnWorldMap { x: 70, y: -3 }))
    );
    assert!(test.client.visit.is_none());
}
