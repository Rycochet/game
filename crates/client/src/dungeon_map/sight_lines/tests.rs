use shared::dungeon::Cell;
use shared::dungeon::{Level, Sight};

use super::*;

fn point(x: i64, y: i64) -> Vec2d {
    Vec2d {
        x: x as f64,
        y: y as f64,
    }
}

/// Whether a point is inside an outline (or within `margin` cells of it):
/// some ray from the viewer towards it gets that far.
fn reaches(map: &WallMap, from: Vec2d, to: Vec2d, margin: f64) -> bool {
    let angle = (to.y - from.y).atan2(to.x - from.x);
    let distance = (to.x - from.x).hypot(to.y - from.y);
    [-1.0, 0.0, 1.0].iter().any(|side| {
        let beside = side * margin / distance.max(1.0);
        let end = map.ray_end(from, angle + beside);
        (end.x - from.x).hypot(end.y - from.y) >= distance - margin
    })
}

/// A room of floor with walls scattered by a hash, so every arrangement of
/// walls and gaps turns up.
fn scattered() -> Level {
    let mut level = Level::new(-20, -20, 41, 41);
    for y in -19..=19 {
        for x in -19..=19 {
            let hash: i64 = (x * 73_856_093) ^ (y * 19_349_663);
            let wall = hash.rem_euclid(7) == 0 && (x, y) != (0, 0);
            level.set(x, y, if wall { Cell::Wall } else { Cell::Floor });
        }
    }
    level
}

/// Whether rays to a point and a little to each side of it all get there:
/// it is well inside the outline, not only on a line grazing a wall's corner
/// (which the game's sight counts as blocked).
fn well_inside(map: &WallMap, from: Vec2d, to: Vec2d) -> bool {
    let angle = (to.y - from.y).atan2(to.x - from.x);
    let distance = (to.x - from.x).hypot(to.y - from.y);
    [-0.001, 0.0, 0.001].iter().all(|side| {
        let end = map.ray_end(from, angle + side);
        (end.x - from.x).hypot(end.y - from.y) >= distance
    })
}

/// The outline agrees with the game's sight: what the game says is seen is
/// inside it, and what it says isn't is outside, whichever cell looks.
#[test]
fn outline_agrees_with_the_games_sight() {
    for level in [Level::test_room(), scattered()] {
        let map = WallMap::of(&level);
        let (min_x, min_y, width, height) = level.bounds();
        for (vx, vy) in [(0, 0), (-5, 2), (3, -3), (6, 1)] {
            if !level.is_open(vx, vy) {
                continue;
            }
            let mut sight = Sight::new(&level);
            sight.add_view(&level, vx, vy, None);
            for y in min_y..min_y + height {
                for x in min_x..min_x + width {
                    if !level.is_open(x, y) || (x, y) == (vx, vy) {
                        continue;
                    }
                    let (from, to) = (point(vx, vy), point(x, y));
                    if sight.is_seen(x, y) {
                        assert!(
                            reaches(&map, from, to, 0.001),
                            "{x},{y} is seen from {vx},{vy}"
                        );
                    } else {
                        assert!(
                            !well_inside(&map, from, to),
                            "{x},{y} isn't seen from {vx},{vy}"
                        );
                    }
                }
            }
        }
    }
}

/// The outline turns at a wall's end: rays either side of it end very
/// differently, one on the wall and one far beyond.
#[test]
fn outline_turns_at_the_end_of_a_wall() {
    let level = Level::test_room();
    let map = WallMap::of(&level);
    let outline = map.outline(point(0, 0));
    // The 1 × 3 wall at x = -2 hides what is behind it from the centre.
    assert!(!outline.iter().any(|p| p.x < -3.0 && p.y.abs() < 0.2));
    assert!(outline.iter().any(|p| p.x < -6.0 && p.y.abs() > 1.0));
}

/// Every cell the game counts as glimpsed or seen that the outline also
/// reaches is touched, and nothing is touched behind a closed wall.
#[test]
fn touched_cells_stop_at_walls() {
    let mut level = Level::new(-8, -3, 17, 7);
    for y in -2..=2 {
        for x in -7..=7 {
            level.set(x, y, Cell::Floor);
        }
    }
    for y in -3..=3 {
        level.set(2, y, Cell::Wall);
    }
    let map = WallMap::of(&level);
    let from = point(0, 0);
    let mut touched = Sight::new(&level);
    touch_cells(&level, from, &map.outline(from), &mut touched);
    assert!(touched.is_seen(-7, 2) && touched.is_seen(1, -2));
    assert!(touched.is_seen(2, 0), "the wall itself");
    assert!(!touched.is_seen(3, 0) && !touched.is_seen(7, 2));
}

/// What the lines reach beyond what the server sends is rare: it sends what
/// glimpses see and the cells beside them, and more besides, from nearby
/// cells. Anything it hasn't sent is wall here, so this is at worst a small
/// piece missing from the edge of sight, never something revealed.
#[test]
fn lines_rarely_reach_past_what_is_sent() {
    let level = scattered();
    let map = WallMap::of(&level);
    let (min_x, min_y, width, height) = level.bounds();
    let (mut touched_cells, mut unsent) = (0, 0);
    for (vx, vy) in [(0, 0), (-5, 2), (3, -3), (6, 1), (-12, -12), (10, 14)] {
        if !level.is_open(vx, vy) {
            continue;
        }
        let from = point(vx, vy);
        let mut touched = Sight::new(&level);
        touch_cells(&level, from, &map.outline(from), &mut touched);
        let mut sent = Sight::new(&level);
        sent.add_glimpses(&level, vx, vy, None);
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                if !level.is_open(x, y) || !touched.is_seen(x, y) {
                    continue;
                }
                touched_cells += 1;
                let beside = [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|(dx, dy)| sent.is_seen(x + dx, y + dy));
                unsent += i32::from(!beside);
            }
        }
    }
    assert!(touched_cells > 100);
    assert!(unsent * 20 <= touched_cells, "{unsent} of {touched_cells}");
}

/// A viewer moving between cells still gets a whole outline round it.
#[test]
fn outline_from_between_cells_stays_in_the_room() {
    let level = Level::test_room();
    let map = WallMap::of(&level);
    let outline = map.outline(Vec2d { x: 0.5, y: 0.5 });
    assert!(outline.len() >= EVEN_RAYS);
    for end in outline {
        assert!(end.x.abs() <= 8.0 && end.y.abs() <= 4.0, "{end:?}");
    }
}

#[test]
fn facing_a_way_the_outline_reaches_ahead_and_only_the_near_circle_behind() {
    use shared::creature::CreatureKind;
    use shared::facing::{Cone, Field, Heading};
    let level = Level::test_room();
    let map = WallMap::of(&level);
    let from = point(0, 2);
    let cone = Cone::new(
        shared::spot::Spot::middle_of((0, 2)),
        Heading::EAST,
        Field::of(CreatureKind::Human, 0),
    );
    let outline = map.outline_in(from, Some(&cone));
    let furthest = |east: bool| {
        outline
            .iter()
            .filter(|end| (end.x > from.x) == east)
            .map(|end| (end.x - from.x).hypot(end.y - from.y))
            .fold(0.0, f64::max)
    };
    assert!(furthest(true) > 5.0, "ahead reaches the far wall");
    assert!(furthest(false) <= 1.0 + 1e-6, "behind only the near circle");
    // All round, it is the plain outline.
    assert_eq!(map.outline_in(from, None).len(), map.outline(from).len());
}
