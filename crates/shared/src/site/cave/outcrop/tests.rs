use super::*;
use crate::dungeon::Level;
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::site::Site;

#[test]
fn hull_of_a_square_is_its_corners() {
    let square = (0..5).flat_map(|y| (0..5).map(move |x| (x, y))).collect();
    let hull = convex_hull(square);
    assert_eq!(hull.len(), 4);
    for corner in [(0, 0), (4, 0), (4, 4), (0, 4)] {
        assert!(hull.contains(&corner));
    }
}

#[test]
fn rock_reaches_its_depth_round_a_dot() {
    let rock = Rock {
        hull: convex_hull(vec![(0, 0)]),
        depth: 3,
    };
    assert!(rock.covers(3, 0) && rock.covers(2, 2) && rock.covers(0, -3));
    assert!(!rock.covers(4, 0) && !rock.covers(3, 3));
}

/// Cells of outside ground joined to the level's edge: what is round the hill.
fn outside(level: &Level) -> Vec<bool> {
    let (_, _, width, height) = level.bounds();
    let index = |x: i64, y: i64| (y * width + x) as usize;
    let mut seen = vec![false; (width * height) as usize];
    let mut queue: Vec<(i64, i64)> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .filter(|&(x, y)| x == 0 || y == 0 || x == width - 1 || y == height - 1)
        .collect();
    while let Some((x, y)) = queue.pop() {
        if !level.contains(x, y) || seen[index(x, y)] || level.cell(x, y) != Cell::Ground {
            continue;
        }
        seen[index(x, y)] = true;
        queue.extend([(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]);
    }
    seen
}

/// Whether walking from `(x, y)` one way along a row or column reaches
/// outside ground over passage floor only: a mouth's way out.
fn tunnel_out(level: &Level, out: &[bool], (x, y): (i64, i64)) -> bool {
    let width = level.bounds().2;
    [(0, 1), (0, -1), (1, 0), (-1, 0)].iter().any(|&(dx, dy)| {
        let (mut cx, mut cy) = (x, y);
        loop {
            if !level.contains(cx, cy) {
                return false;
            }
            if out[(cy * width + cx) as usize] {
                return true;
            }
            if level.cell(cx, cy) != Cell::Floor || level.room(cx, cy).is_some() {
                return false;
            }
            (cx, cy) = (cx + dx, cy + dy);
        }
    })
}

/// A cave mouth's top level: the hill (everything but the ground round it)
/// is one run along every row and column, nothing open lies within the least
/// depth of the ground round it but the mouths' ways out, and you arrive on
/// that ground.
#[test]
fn cave_mouths_stand_in_a_convex_hill() {
    for n in 0..60 {
        for kind in [
            PoiKind::GrassyCave,
            PoiKind::EarthCave,
            PoiKind::RockCave,
            PoiKind::IceCave,
        ] {
            let first = Site::new(kind, Seed::new(n)).level(0);
            let level = &first.level;
            let (_, _, width, height) = level.bounds();
            let out = outside(level);
            let hill = |x: i64, y: i64| !out[(y * width + x) as usize];
            let runs = |cells: Vec<bool>| cells.windows(2).filter(|w| w[0] != w[1]).count() <= 2;
            for y in 0..height {
                assert!(
                    runs((0..width).map(|x| hill(x, y)).collect()),
                    "{kind:?} {n} row {y}"
                );
            }
            for x in 0..width {
                assert!(
                    runs((0..height).map(|y| hill(x, y)).collect()),
                    "{kind:?} {n} column {x}"
                );
            }
            let least = LEAST_ROCK.0;
            for y in 0..height {
                for x in 0..width {
                    if !hill(x, y) || level.cell(x, y) == Cell::Wall {
                        continue;
                    }
                    let close = (-least + 1..least).any(|dy: i64| {
                        (-least + 1..least).any(|dx: i64| {
                            let (nx, ny) = (x + dx, y + dy);
                            dx * dx + dy * dy < least * least
                                && level.contains(nx, ny)
                                && !hill(nx, ny)
                        })
                    });
                    assert!(
                        !close || tunnel_out(level, &out, (x, y)),
                        "{kind:?} {n}: ({x}, {y}) too near the outside"
                    );
                }
            }
            let (ex, ey) = first.entrance.expect("an entrance");
            assert!(!hill(ex, ey), "{kind:?} {n}: entrance in the hill");
        }
    }
}
