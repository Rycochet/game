use shared::dungeon::Cell;
use shared::movement::{Step, step_on_level};
use shared::poi::ChunkPos;
use shared::protocol::{ClientMessage, LevelKind, ServerMessage};
use shared::wilderness::{ChunkPlan, Cutout, SCALE};

use crate::connections::tests::TestClient;
use crate::storage::SiteId;

/// A client who has just explored the wilderness from where they started.
fn explored() -> (TestClient, Vec<ServerMessage>) {
    let mut test = TestClient::new();
    let answers = test.send(&[ClientMessage::Explore]);
    (test, answers)
}

fn position(test: &TestClient) -> (i64, i64) {
    test.client.visit.as_ref().expect("in a visit").position()
}

/// The first chunk near the start whose cutout `wanted` picks, and its plan.
fn chunk_near_start(test: &TestClient, wanted: impl Fn(&Cutout) -> bool) -> ChunkPlan {
    let world = &test.world;
    let (x, y) = test.client.world_position();
    let start = world.pois.chunk_at(x, y);
    (1..12)
        .flat_map(|ring| (-ring..=ring).flat_map(move |dy| (-ring..=ring).map(move |dx| (dx, dy))))
        .map(|(dx, dy)| ChunkPos {
            x: start.x + dx,
            y: start.y + dy,
        })
        .map(|chunk| ChunkPlan::with_new_site(&world.wilderness.wild, &world.pois, chunk))
        .find(|plan| plan.cutout().is_some_and(&wanted))
        .expect("such a poi near the start")
}

/// Explores from a cutout's poi, and stands on its stairs.
fn on_cutout_stairs(wanted: impl Fn(&Cutout) -> bool) -> (TestClient, ChunkPlan) {
    let mut test = TestClient::new();
    let plan = chunk_near_start(&test, wanted);
    let cutout = plan.cutout().unwrap();
    test.client.character.world_position = (cutout.poi().x, cutout.poi().y);
    test.send(&[ClientMessage::Explore]);
    let stairs = cutout.stairs()[0];
    test.send(&[ClientMessage::DevTeleport {
        x: stairs.x,
        y: stairs.y,
    }]);
    assert_eq!(position(&test), (stairs.x, stairs.y));
    (test, plan)
}

#[test]
fn exploring_starts_where_the_player_stands_and_remembers_what_is_seen() {
    let mut test = TestClient::new();
    let start = test.client.world_position();
    let answers = test.send(&[ClientMessage::Explore]);
    let info = answers.iter().find_map(|answer| match answer {
        ServerMessage::EnterLevel(info) => Some(info),
        _ => None,
    });
    assert_eq!(info.expect("a level").kind, LevelKind::Wilderness);
    assert!(
        answers
            .iter()
            .any(|a| matches!(a, ServerMessage::LevelCells(_)))
    );
    let (x, y) = position(&test);
    let (px, py) = test.world.wilderness.wild.point_of(x, y);
    assert!(
        (px - start.0).abs() <= 3 && (py - start.1).abs() <= 3,
        "{start:?}"
    );
    assert_eq!(test.client.world_position(), (px, py));
    let memory = &test.client.character.memory[&SiteId::Wilderness][&0];
    assert!(!memory.is_empty());
}

#[test]
fn walking_moves_the_window_and_leaving_comes_out_where_the_player_is() {
    let (mut test, _) = explored();
    let start = position(&test);
    let mut windows = 0;
    // East, or the way round anything in the way.
    for _ in 0..150 {
        let visit = test.client.visit.as_ref().unwrap();
        let level = &visit.level.level;
        let step = [(1, 0), (1, -1), (1, 1), (0, -1), (0, 1)]
            .into_iter()
            .map(|(dx, dy)| Step::new(dx, dy).unwrap())
            .find(|&step| step_on_level(level, visit.position(), step).is_some())
            .expect("somewhere to go");
        let answers = test.steps(&[(step.dx(), step.dy())]);
        windows += answers
            .iter()
            .filter(|a| matches!(a, ServerMessage::WildernessWindow { .. }))
            .count();
    }
    let (x, y) = position(&test);
    assert!(x - start.0 >= 64, "only got from {start:?} to ({x}, {y})");
    assert!(windows >= 1);
    let at = test.world.wilderness.wild.point_of(x, y);
    assert_eq!(test.client.world_position(), at);
    let answers = test.send(&[ClientMessage::Leave]);
    assert!(test.client.visit.is_none(), "{answers:?}");
    assert!(answers.contains(&ServerMessage::OnWorldMap { x: at.0, y: at.1 }));
}

#[test]
fn changes_are_kept_by_place_and_leaving_is_only_from_outside() {
    let (mut test, _) = explored();
    let (x, y) = position(&test);
    let beside = [(1, 0), (0, 1), (-1, 0), (0, -1)]
        .into_iter()
        .map(|(dx, dy)| (x + dx, y + dy))
        .find(|&(bx, by)| {
            let visit = test.client.visit.as_ref().unwrap();
            visit.level.level.is_walkable(bx, by)
        })
        .expect("room beside");
    test.send(&[
        ClientMessage::DevSetCell {
            x: beside.0,
            y: beside.1,
            cell: Cell::Floor,
        },
        ClientMessage::DevTeleport {
            x: beside.0,
            y: beside.1,
        },
        ClientMessage::Leave,
    ]);
    assert!(
        test.client
            .visit
            .as_ref()
            .is_some_and(|v| v.is_wilderness())
    );
    let width = test.world.wilderness.wild.width();
    let changes = &test.world.site(SiteId::Wilderness).levels[&0].changes;
    assert_eq!(
        changes.get(&(beside.0.rem_euclid(width), beside.1)),
        Some(&Cell::Floor)
    );
}

#[test]
fn a_cutouts_stairs_lead_into_its_site_and_back_out_to_the_wilderness() {
    let (mut test, plan) = on_cutout_stairs(|c| c.offset().is_some() && !c.stairs().is_empty());
    let cutout = plan.cutout().unwrap();
    let id = SiteId::Poi(cutout.poi().chunk);
    // What was seen of it is remembered of its site too.
    assert!(test.client.character.memory.contains_key(&id));
    let stairs = cutout.stairs()[0];
    test.send(&[ClientMessage::UseExit]);
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.came_from_wilderness() && visit.id() == id);
    assert_eq!(
        Some(visit.index()),
        match stairs.to {
            shared::site::Exit::Level(index) => Some(index),
            shared::site::Exit::WorldMap => None,
        }
    );
    test.send(&[ClientMessage::UseExit]);
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.is_wilderness(), "back out");
    let width = test.world.wilderness.wild.width();
    let (x, y) = visit.position();
    // Back where the stairs are, or for a cave at the stairs it came up.
    assert!(cutout.contains(x.rem_euclid(width), y));
}

#[test]
fn a_site_entered_from_above_has_only_its_way_down_set_in() {
    let (mut test, plan) = on_cutout_stairs(|c| c.offset().is_none());
    let cutout = plan.cutout().unwrap();
    let way_down = (cutout.stairs()[0].x, cutout.stairs()[0].y);
    test.send(&[ClientMessage::UseExit]);
    let visit = test.client.visit.as_ref().unwrap();
    assert!(visit.came_from_wilderness() && visit.index() == 0);
    test.send(&[ClientMessage::UseExit]);
    assert!(test.client.visit.as_ref().unwrap().is_wilderness());
    let width = test.world.wilderness.wild.width();
    let (x, y) = position(&test);
    assert_eq!((x.rem_euclid(width), y), way_down);
}

#[test]
fn exploring_from_water_or_a_site_is_refused() {
    let mut test = TestClient::new();
    test.send(&[ClientMessage::DevEnterSite {
        kind: shared::poi::PoiKind::Castle,
        seed: 1,
        level: 0,
        ruin: 0,
    }]);
    test.send(&[ClientMessage::Explore]);
    assert!(!test.client.visit.as_ref().unwrap().is_wilderness());
    let mut test = TestClient::new();
    let (x, y) = test.client.world_position();
    let water = (0..2000)
        .map(|d| (x + d, y))
        .find(|&(wx, wy)| {
            test.world
                .ground
                .elevation(wx, wy, test.world.conditions())
                .is_none()
        })
        .expect("sea somewhere east");
    test.client.character.world_position = water;
    test.send(&[ClientMessage::Explore]);
    assert!(test.client.visit.is_none());
    assert_eq!(SCALE, 16);
}

#[test]
fn a_town_set_into_the_wilderness_has_its_buildings_sent() {
    let mut test = TestClient::new();
    let plan = chunk_near_start(&test, |c| c.poi().kind.is_town());
    let cutout = plan.cutout().unwrap();
    test.client.character.world_position = (cutout.poi().x, cutout.poi().y);
    let answers = test.send(&[ClientMessage::Explore]);
    let buildings = answers.iter().find_map(|answer| match answer {
        ServerMessage::Buildings(buildings) => Some(buildings),
        _ => None,
    });
    let buildings = buildings.expect("the town's buildings");
    assert!(!buildings.is_empty());
    // Where the town is in the wilderness.
    let width = test.world.wilderness.wild.width();
    for building in buildings {
        let rect = building.rects[0];
        assert!(
            cutout.contains(rect.x.rem_euclid(width), rect.y),
            "{rect:?}"
        );
    }
}

#[test]
fn a_road_found_runs_through_the_wilderness() {
    let mut test = TestClient::new();
    let (px, py) = test.client.world_position();
    let link = shared::road::RoadLink::new(ChunkPos { x: 0, y: 0 }, ChunkPos { x: 1, y: 0 });
    let road = shared::road::Road {
        link,
        kind: shared::road::RoadKind::Track,
        points: (px - 8..=px + 8).map(|x| (x, py)).collect(),
    };
    test.world.wilderness.add_road(std::sync::Arc::new(road));
    test.send(&[ClientMessage::Explore]);
    let visit = test.client.visit.as_ref().unwrap();
    let level = &visit.level.level;
    let (min_x, min_y, width, height) = level.bounds();
    let surfaced = (min_y..min_y + height)
        .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(level.cell(x, y), Cell::GravelTrack | Cell::Bridge))
        .count();
    assert!(surfaced > 100, "{surfaced}");
}

#[test]
fn stepping_onto_a_world_gate_goes_through_it() {
    // New characters arrive at a World Gate.
    let mut test = TestClient::new();
    test.send(&[ClientMessage::Explore]);
    let wild = &test.world.wilderness.wild;
    let (x, y) = position(&test);
    let gates = shared::wilderness::gates_of(wild, &test.world.pois);
    let gate = gates
        .iter()
        .find(|gate| (gate.cell.0 - wild.wrap(x)).abs() < 64 && (gate.cell.1 - y).abs() < 64)
        .expect("the gate they arrived at");
    let visit = test.client.visit.as_ref().unwrap();
    assert_eq!(
        visit.level.level.cell(gate.cell.0, gate.cell.1),
        Cell::WorldGate
    );
    assert_ne!((wild.wrap(x), y), gate.cell, "not arrived on it");
    test.walk_to(gate.cell);
    assert!(test.client.visit.is_none());
    assert!(test.client.character.hall.is_some(), "in the Hall");
}
