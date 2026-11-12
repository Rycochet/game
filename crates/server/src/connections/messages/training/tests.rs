//! Practice from moving, through a connected client.

use shared::movement::{MOVEMENT_SKILLS, RUN_PRACTICE, climb_practice};
use shared::protocol::ServerMessage;
use shared::skill::WINDOW_TICKS;

use shared::dungeon::{Cell, Level};
use shared::energy::site_step_cost;
use shared::movement::{Gait, LevelWay, Terrain};
use shared::relief::Relief;
use shared::spot::Spot;

use super::{Moved, Walked};
use crate::connections::tests::TestClient;

fn moved(on_world_map: bool, elevations: u32) -> Moved {
    Moved {
        on_world_map,
        elevations,
        swam: 0,
        jumped: false,
        sneaked: false,
        terrain: None,
    }
}

#[test]
fn world_map_steps_practise_running() {
    let mut test = TestClient::new();
    let running = MOVEMENT_SKILLS.running;
    let before = test.client.sheet.skills.practice(running);
    let from = test.client.world_position();
    test.steps(&[(1, 0), (1, 0), (1, 0)]);
    assert_ne!(test.client.world_position(), from, "it walked");
    // Once in the window, however many steps.
    assert_eq!(
        test.client.sheet.skills.practice(running),
        before + RUN_PRACTICE
    );
}

#[test]
fn only_steep_steps_practise_climbing() {
    let mut test = TestClient::new();
    let climbing = MOVEMENT_SKILLS.climbing;
    let before = test.client.sheet.skills.practice(climbing);
    assert!(test.client.practise_moving(moved(true, 1), 0));
    assert_eq!(test.client.sheet.skills.practice(climbing), before);
    assert!(test.client.practise_moving(moved(true, 5), WINDOW_TICKS));
    assert_eq!(
        test.client.sheet.skills.practice(climbing),
        before + climb_practice(5)
    );
}

#[test]
fn the_sheet_is_sent_only_when_moving_changes() {
    let mut test = TestClient::new();
    let sheets = |test: &mut TestClient| {
        let mut count = 0;
        while let Some(message) = test.end.receive().unwrap() {
            count += usize::from(matches!(message, ServerMessage::Sheet(_)));
        }
        count
    };
    sheets(&mut test);
    // Running's value grows from nothing, so the client is told.
    assert!(test.client.practise_moving(moved(true, 0), 0));
    assert_eq!(sheets(&mut test), 1);
    // Nothing more inside the window: nothing to tell.
    assert!(test.client.practise_moving(moved(true, 0), 1));
    assert_eq!(sheets(&mut test), 0);
}

#[test]
fn heavy_loads_practise_weight_training_on_any_map() {
    use shared::sheet::Load;
    let mut test = TestClient::new();
    let node = MOVEMENT_SKILLS.weight_training;
    test.client.sheet.attributes.strength = u8::MAX.into();
    assert_eq!(test.client.sheet.load(&test.client.inventory), Load::Light);
    assert!(test.client.practise_moving(moved(false, 0), 0));
    assert_eq!(test.client.sheet.skills.practice(node), 0, "light: nothing");
    // Too weak for even their clothes.
    test.client.sheet.attributes.strength = 1;
    assert_ne!(test.client.sheet.load(&test.client.inventory), Load::Light);
    assert!(test.client.practise_moving(moved(false, 0), WINDOW_TICKS));
    assert!(test.client.sheet.skills.practice(node) > 0);
}

#[test]
fn world_map_rivers_are_waded() {
    let mut test = TestClient::new();
    let world = &mut test.world;
    // A river point some way from any road.
    let river = (0..world.rivers.columns())
        .find_map(|x| world.rivers.in_chunk(shared::poi::ChunkPos { x, y: 2 }))
        .expect("a river");
    let to = river.points[river.points.len() / 2];
    let from = (to.0 - 1, to.1);
    let ends = (Some(8), Some(8));
    let swimming = MOVEMENT_SKILLS.swimming;
    let before = test.client.sheet.skills.practice(swimming);
    let gait = shared::movement::Gait::of(&test.client.sheet.skills);
    let (stayed, cost) = test.client.world_step(&mut test.world, (from, to), ends);
    assert!(stayed);
    if test.world.road_steps.kind_of(from, to).is_none() {
        // Waded, and not run.
        assert_eq!(cost, gait.swim(70));
        assert!(test.client.sheet.skills.practice(swimming) > before);
    }
}

#[test]
fn the_sea_is_swum() {
    let mut test = TestClient::new();
    let gait = shared::movement::Gait::of(&test.client.sheet.skills);
    // A coast: land, with the sea to the east.
    let conditions = test.world.conditions();
    let ground = &test.world.ground;
    let from = (-1500..1500)
        .step_by(4)
        .flat_map(|y| (-1500..1500).map(move |x| (x, y)))
        .find(|&(x, y)| {
            ground.elevation(x, y, conditions).is_some()
                && ground.elevation(x + 1, y, conditions).is_none()
        })
        .expect("a coast");
    let to = (from.0 + 1, from.1);
    let ends = (ground.elevation(from.0, from.1, conditions), None);
    let (stayed, cost) = test.client.world_step(&mut test.world, (from, to), ends);
    assert!(stayed);
    assert_eq!(cost, gait.swim(70), "swum, and not run");
}

/// A row of four `cell`s.
fn row(cell: Cell) -> Level {
    let mut level = Level::new(0, 0, 4, 1);
    for x in 0..4 {
        level.set(x, 0, cell);
    }
    level
}

/// The test client moving from cell `from`'s middle to `to`'s on `level`.
fn walk(
    test: &mut TestClient,
    (level, relief, terrain): (&Level, Option<&Relief>, Option<Terrain>),
    (from, to): ((i64, i64), (i64, i64)),
    jumped: bool,
) -> (bool, u32) {
    let way = LevelWay {
        level,
        relief,
        terrain,
        gait: Gait::of(&test.client.sheet.skills),
    };
    let walked = Walked::of(&way, Spot::middle_of(from), Spot::middle_of(to), jumped);
    test.client.level_move(&test.world, walked, true)
}

#[test]
fn slopes_cost_more_and_practise_climbing() {
    let mut test = TestClient::new();
    let level = row(Cell::Floor);
    // Rising four 16ths of an elevation a cell to the east.
    let hill = Relief::from_fn(level.bounds(), |x, _| x as i32 * 120);
    let climbing = MOVEMENT_SKILLS.climbing;
    let before = test.client.sheet.skills.practice(climbing);
    let running = MOVEMENT_SKILLS.running;
    let ran = test.client.sheet.skills.practice(running);
    let (from, to) = ((1, 0), (2, 0));
    let (stayed, cost) = walk(&mut test, (&level, Some(&hill), None), (from, to), false);
    assert!(stayed);
    assert!(cost > site_step_cost(from, to), "{cost} on a hillside");
    assert!(test.client.sheet.skills.practice(climbing) > before);
    // Running is the world map's alone.
    assert_eq!(test.client.sheet.skills.practice(running), ran);
}

#[test]
fn wading_is_slower_and_practises_swimming() {
    let mut test = TestClient::new();
    let level = row(Cell::Shallows);
    let swimming = MOVEMENT_SKILLS.swimming;
    let before = test.client.sheet.skills.practice(swimming);
    let gait = Gait::of(&test.client.sheet.skills);
    let (from, to) = ((0, 0), (1, 0));
    let (stayed, cost) = walk(&mut test, (&level, None, None), (from, to), false);
    assert!(stayed);
    // Between an untrained swimmer's 1.7 times as long and a record's 0.71.
    let span = shared::spot::CELL_SPAN as u64;
    let waded = (0..4).map(|_| u64::from(gait.swim(60))).sum::<u64>();
    assert_eq!(cost, shared::energy::level_move_energy(waded));
    assert_eq!(span, 240);
    assert!(
        cost > site_step_cost(from, to),
        "this one is no great swimmer"
    );
    assert!(test.client.sheet.skills.practice(swimming) > before);
}

#[test]
fn jumps_are_quicker_with_practice_and_practise_jumping() {
    let mut test = TestClient::new();
    let level = row(Cell::Floor);
    let jumping = MOVEMENT_SKILLS.jumping;
    let before = test.client.sheet.skills.practice(jumping);
    let gait = Gait::of(&test.client.sheet.skills);
    let (from, to) = ((0, 0), (2, 0));
    let (stayed, cost) = walk(&mut test, (&level, None, None), (from, to), true);
    assert!(stayed);
    let jumped = u64::from(gait.jump(480));
    assert_eq!(cost, shared::energy::level_move_energy(jumped));
    assert!(cost <= site_step_cost(from, to));
    assert!(test.client.sheet.skills.practice(jumping) > before);
}

#[test]
fn rough_country_is_slower_and_practises_pioneering() {
    let mut test = TestClient::new();
    let level = row(Cell::Ground);
    let (from, to) = ((0, 0), (1, 0));
    let rough = (&level, None, Some(Terrain::Desert));
    let (stayed, cost) = walk(&mut test, rough, (from, to), false);
    assert!(stayed);
    let flat = site_step_cost(from, to);
    assert!(cost > flat, "{cost} against {flat}");
    let practised = Terrain::ALL
        .iter()
        .any(|terrain| test.client.sheet.skills.practice(terrain.skill()) > 0);
    assert!(practised);
}
