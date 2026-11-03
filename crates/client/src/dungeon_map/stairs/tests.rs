use shared::protocol::SeenStairs;
use shared::site::{StairsDirection, StairsStyle};

use super::super::CELL_SIZE;
use super::super::mesh::{Builder, Surface, View};
use super::*;

fn build(direction: StairsDirection, style: StairsStyle) -> Vec<Builder> {
    let mut meshes: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
    let stairs = SeenStairs {
        open_to_sky: false,
        x: 0,
        y: 0,
        direction,
        descends: (0, 1),
        style,
    };
    build_stairs(
        &mut meshes,
        None,
        &stairs,
        Surface::Floor,
        Vec3::ONE,
        &|_| Vec3::ONE,
    );
    meshes
}

fn all_points(meshes: &[Builder]) -> impl Iterator<Item = Vec3> + '_ {
    meshes
        .iter()
        .flat_map(|mesh| mesh.positions.iter().map(|&p| Vec3::from_array(p)))
}

/// Ladders going up have a brighter circle of floor as wide as the hole
/// above, and the floor still covers the whole cell; stairs have no circle.
#[test]
fn only_ladders_going_up_have_light_under_them() {
    let half = CELL_SIZE / 2.0;
    for style in StairsStyle::ALL {
        let mut meshes: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
        let stairs = SeenStairs {
            open_to_sky: false,
            x: 0,
            y: 0,
            direction: StairsDirection::Up,
            descends: (0, 1),
            style,
        };
        // Only the floor comes straight from `floor_colour`.
        build_stairs(
            &mut meshes,
            None,
            &stairs,
            Surface::Floor,
            Vec3::ONE,
            &|_| Vec3::ONE,
        );
        let floor = &meshes[Surface::Floor as usize];
        let flat = || {
            floor
                .positions
                .iter()
                .zip(&floor.colours)
                .filter(|(p, _)| p[1] == 0.0)
        };
        let lit: Vec<_> = flat().filter(|(_, c)| c[0] == LADDER_LIGHT_TINT).collect();
        if matches!(style, StairsStyle::Spiral | StairsStyle::Turning) {
            assert!(lit.is_empty(), "{style:?}");
            continue;
        }
        let reach = lit
            .iter()
            .map(|(p, _)| Vec3::from_array(**p).length())
            .fold(0.0, f32::max);
        assert!((reach - ROUND_RADIUS).abs() < 0.001, "{style:?}: {reach}");
        let furthest = flat()
            .map(|(p, _)| p[0].abs().max(p[2].abs()))
            .fold(0.0, f32::max);
        assert!((furthest - half).abs() < 0.001, "{style:?}: {furthest}");
    }
}

/// Every model leaves a border of floor round it inside its cell, apart
/// from the floor itself; a rope's peg may stand on that border.
#[test]
fn models_stay_inside_their_cell() {
    for style in StairsStyle::ALL {
        for direction in [StairsDirection::Up, StairsDirection::Down] {
            let meshes = build(direction, style);
            for p in all_points(&meshes[Surface::Door as usize..=Surface::Door as usize]) {
                assert!(p.x.abs() < CELL_SIZE / 2.0, "{style:?}: {p}");
                assert!(p.z.abs() < CELL_SIZE / 2.0, "{style:?}: {p}");
            }
            for p in all_points(&meshes[Surface::Wall as usize..=Surface::Wall as usize]) {
                assert!(p.x.abs() < CELL_SIZE * 0.45, "{style:?}: {p}");
                assert!(p.z.abs() < CELL_SIZE * 0.45, "{style:?}: {p}");
            }
        }
    }
}

/// Going down, the model is below the floor, with only rails and the tops
/// of ladders above it, and the hole has a black base.
#[test]
fn stairs_down_go_below_the_floor_with_a_black_base() {
    for style in StairsStyle::ALL {
        let meshes = build(StairsDirection::Down, style);
        let lowest = all_points(&meshes).map(|p| p.y).fold(f32::MAX, f32::min);
        // The corner of a leaning pole's foot dips a little under the base.
        assert!((lowest + STAIRS_DOWN_DEPTH).abs() < 0.6, "{style:?}");
        let highest = all_points(&meshes).map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(highest > 0.5, "{style:?} doesn't come above the floor");

        let floor = &meshes[Surface::Floor as usize];
        let base = floor
            .positions
            .iter()
            .zip(&floor.colours)
            // The middle of the base, which step fronts don't reach.
            .filter(|(p, _)| p[1] < 0.01 - STAIRS_DOWN_DEPTH && p[0] == 0.0 && p[2] == 0.0)
            .map(|(_, colour)| colour);
        let mut found = false;
        for colour in base {
            found = true;
            assert_eq!(colour[..3], [0.0, 0.0, 0.0], "{style:?}");
        }
        assert!(found, "{style:?} has no base");
    }
}

/// Stairs are see-through for a viewer on them or behind them from the
/// camera, not for one in front of them or off to the side.
#[test]
fn stairs_hide_viewers_on_them_or_behind_them() {
    use shared::dungeon::{Level, Sight};
    let level = Level::test_room();
    let stairs = SeenStairs {
        open_to_sky: false,
        x: 0,
        y: 0,
        direction: StairsDirection::Up,
        descends: (0, 1),
        style: StairsStyle::Spiral,
    };
    // South of the stairs, looking north.
    let camera = Vec3::new(0.0, 200.0, 200.0);
    let at = |x, y| {
        hides_viewer(
            &stairs,
            &[View {
                cell: (x, y),
                sight: Sight::new(&level),
            }],
            camera,
        )
    };
    assert!(at(0, 0));
    assert!(at(0, -1));
    assert!(!at(0, 1));
    assert!(!at(3, -1));
}

/// See-through stairs keep their floor solid.
#[test]
fn only_what_stands_above_the_floor_goes_see_through() {
    let mut meshes: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
    let mut through: Vec<Builder> = Surface::ALL.iter().map(|_| Builder::default()).collect();
    let stairs = SeenStairs {
        open_to_sky: false,
        x: 0,
        y: 0,
        direction: StairsDirection::Down,
        descends: (0, 1),
        style: StairsStyle::Spiral,
    };
    let colour = |_| Vec3::ONE;
    build_stairs(
        &mut meshes,
        Some(&mut through),
        &stairs,
        Surface::Floor,
        Vec3::ONE,
        &colour,
    );
    assert!(all_points(&meshes).all(|p| p.y <= 0.01));
    assert!(all_points(&through).any(|p| p.y > 0.01));
}

/// Going up, models are cut off at a tile and a half, and reach it, and
/// stairs have no handrails.
#[test]
fn stairs_up_stop_at_their_top_without_rails() {
    for style in StairsStyle::ALL {
        let meshes = build(StairsDirection::Up, style);
        let highest = all_points(&meshes).map(|p| p.y).fold(f32::MIN, f32::max);
        assert!(
            (highest - STAIRS_UP_TOP).abs() < 0.5,
            "{style:?}: {highest}"
        );
        if matches!(style, StairsStyle::Spiral | StairsStyle::Turning) {
            assert!(
                meshes[Surface::Door as usize].indices.is_empty(),
                "{style:?}"
            );
        }
    }
}

/// Every step of a spiral has a back and a bottom.
#[test]
fn spiral_steps_are_closed() {
    let meshes = build(StairsDirection::Up, StairsStyle::Spiral);
    let wall = &meshes[Surface::Wall as usize];
    // Each step's edge, bottom and back; the column adds more.
    let quads = wall.indices.len() / 6;
    assert!(quads >= 6 * (2 * SPIRAL_STEP_PIECES + 1), "{quads}");
}

/// Going up, nothing is below the floor.
#[test]
fn stairs_up_stand_on_the_floor() {
    for style in StairsStyle::ALL {
        let meshes = build(StairsDirection::Up, style);
        let lowest = all_points(&meshes).map(|p| p.y).fold(f32::MAX, f32::min);
        assert!(lowest >= -0.5, "{style:?}: {lowest}");
    }
}
