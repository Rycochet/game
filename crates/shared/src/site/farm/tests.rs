use super::super::grid::Reach;
use super::super::town::Rect;
use super::super::{Generator, Site, StairsDirection, leads_out};
use super::*;
use crate::dungeon::{Cell, Level};
use crate::poi::PoiKind;
use crate::rng::Seed;

fn farms(ruin: u8) -> impl Iterator<Item = Site> {
    (0..150).map(move |n| {
        let kind = PoiKind::Farm;
        let generator = Generator::for_poi(kind).with_ruin(ruin);
        Site::with_generator(kind, Seed::new(n * 37 + 5), generator.version(), generator)
            .expect("the current version runs")
    })
}

fn cells_of(level: &Level) -> impl Iterator<Item = (i64, i64)> + use<> {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
}

/// One to eight fields, or a pond too; all rectangles of at least 8 by 8
/// fenced all round, with one gate, on a side and not a corner.
#[test]
fn fields_are_fenced_rectangles_with_one_gate() {
    let (mut most, mut ponds) = (0, 0);
    for site in farms(0) {
        let ground = site.level(0);
        let level = &ground.level;
        let water = cells_of(level).any(|(x, y)| level.cell(x, y) == Cell::Water);
        let count = ground.fields.len() + usize::from(water);
        assert!((1..=8).contains(&count), "{count}");
        assert!(!ground.fields.is_empty());
        most = most.max(count);
        ponds += usize::from(water);
        for field in &ground.fields {
            let Rect {
                x,
                y,
                width,
                height,
            } = field.rect;
            assert!(width >= 8 && height >= 8, "{field:?}");
            let plot = Plot {
                x0: x - 1,
                y0: y - 1,
                x1: x + width + 1,
                y1: y + height + 1,
            };
            assert!(cells_of(level).all(|(cx, cy)| {
                field.contains(cx, cy) == (level.cell(cx, cy) == Cell::Field)
                    || !field.contains(cx, cy)
            }));
            let line = fence_line(plot);
            let gates: Vec<_> = line
                .iter()
                .filter(|&&(cx, cy)| level.cell(cx, cy) == Cell::FieldGate)
                .collect();
            assert_eq!(gates.len(), 1, "{field:?}");
            let &&(gx, gy) = &gates[0];
            let x_side = gx == plot.x0 || gx == plot.x1 - 1;
            let y_side = gy == plot.y0 || gy == plot.y1 - 1;
            assert!(x_side != y_side, "a gate on a corner");
            assert!(
                line.iter().all(|&(cx, cy)| {
                    matches!(level.cell(cx, cy), Cell::Fence | Cell::FieldGate)
                })
            );
        }
    }
    assert_eq!(most, 8);
    assert!((20..80).contains(&ponds), "{ponds}");
}

/// The gate is as near the door as any cell of the fence's sides.
#[test]
fn gates_are_nearest_the_front_door() {
    for site in farms(0) {
        let ground = site.level(0);
        let level = &ground.level;
        let door = ground.entrance.expect("a way in");
        let distance = |x: i64, y: i64| (x - door.0).pow(2) + (y - door.1).pow(2);
        for field in &ground.fields {
            let Rect {
                x,
                y,
                width,
                height,
            } = field.rect;
            let sides = (x..x + width)
                .flat_map(|cx| [(cx, y - 1), (cx, y + height)])
                .chain((y..y + height).flat_map(|cy| [(x - 1, cy), (x + width, cy)]));
            let (gate, best) = sides.fold(((0, 0), i64::MAX), |(gate, best), (cx, cy)| {
                let gate = if level.cell(cx, cy) == Cell::FieldGate {
                    (cx, cy)
                } else {
                    gate
                };
                (gate, best.min(distance(cx, cy)))
            });
            assert_eq!(distance(gate.0, gate.1), best);
        }
    }
}

/// Nothing touches the house, or another field or pond; every cell and
/// gate can be walked to without a jump; a track leads out.
#[test]
fn everything_keeps_clear_and_can_be_walked_to() {
    for ruin in [0, 3, 5] {
        for site in farms(ruin) {
            let ground = site.level(0);
            let level = &ground.level;
            let start = ground.entrance.expect("a way in");
            assert_eq!(level.cell(start.0, start.1), Cell::DirtTrack);
            let mut reach = Reach::new(level);
            reach.spread(level, &[start]);
            let farmed = |cell: Cell| {
                matches!(
                    cell,
                    Cell::Field | Cell::Fence | Cell::FieldGate | Cell::Water
                )
            };
            let house = cells_of(level)
                .filter(|&(x, y)| level.room(x, y).is_some() || level.cell(x, y).is_wall());
            for (x, y) in house {
                for (dx, dy) in (-1..=1).flat_map(|d| (-1..=1).map(move |e| (d, e))) {
                    assert!(!farmed(level.cell(x + dx, y + dy)), "at ({x}, {y})");
                }
            }
            let mut out = false;
            for (x, y) in cells_of(level) {
                assert!(!level.is_walkable(x, y) || reach.is(x, y), "({x}, {y})");
                out |= level.cell(x, y) == Cell::DirtTrack && leads_out(level, x, y);
                // Different fields' fences have ground between them.
                if level.cell(x, y) == Cell::Fence {
                    for field in &ground.fields {
                        let Rect {
                            x: fx,
                            y: fy,
                            width,
                            height,
                        } = field.rect;
                        let beside = (fx - 2..=fx + width + 1).contains(&x)
                            && (fy - 2..=fy + height + 1).contains(&y);
                        let on_it = (fx - 1..=fx + width).contains(&x)
                            && (fy - 1..=fy + height).contains(&y);
                        assert!(!beside || on_it, "fences touch at ({x}, {y})");
                    }
                }
            }
            assert!(out, "no track leads out");
        }
    }
}

/// Two floors (a ruin may lose the upper one), and sometimes a basement,
/// joined by stairs on the same cell.
#[test]
fn farmhouses_have_two_floors_and_some_a_basement() {
    let mut basements = 0;
    for site in farms(0) {
        let (storeys, basement) = if site.has_basement() {
            basements += 1;
            (site.levels() - 1, true)
        } else {
            (site.levels(), false)
        };
        assert_eq!(storeys, 2);
        if basement {
            let last = site.levels() - 1;
            let below = site.level(last);
            assert_eq!(below.stairs.len(), 1);
            let way_up = below.stairs[0];
            assert_eq!(way_up.direction, StairsDirection::Up);
            let down = site.level(0).stairs;
            let down: Vec<_> = down
                .iter()
                .filter(|s| s.direction == StairsDirection::Down)
                .collect();
            assert_eq!(down.len(), 1);
            assert_eq!((down[0].x, down[0].y), (way_up.x, way_up.y));
            assert_eq!(down[0].to, super::super::Exit::Level(last));
            let mut reach = Reach::new(&below.level);
            reach.spread(&below.level, &[(way_up.x, way_up.y)]);
            assert!(
                cells_of(&below.level)
                    .all(|(x, y)| !below.level.is_walkable(x, y) || reach.is(x, y))
            );
        }
    }
    assert!((40..110).contains(&basements), "{basements}");
}

/// However much of a ruin is gone, its basement is the last level and the
/// ground floor's stairs still go there.
#[test]
fn a_ruins_basement_stays_joined() {
    for ruin in 1..=5 {
        for site in farms(ruin).filter(Site::has_basement) {
            let last = site.levels() - 1;
            let ground = site.level(0);
            let down = ground
                .stairs
                .iter()
                .find(|s| s.direction == StairsDirection::Down)
                .expect("stairs down");
            assert_eq!(down.to, super::super::Exit::Level(last));
            let mut reach = Reach::new(&ground.level);
            reach.spread(&ground.level, &[ground.entrance.expect("a way in")]);
            assert!(reach.is(down.x, down.y));
        }
    }
}

#[test]
fn ruined_farms_lose_stretches_of_fence() {
    let fence = |site: &Site| {
        let level = site.level(0).level;
        cells_of(&level)
            .filter(|&(x, y)| level.cell(x, y) == Cell::Fence)
            .count()
    };
    let whole: usize = farms(0).map(|site| fence(&site)).sum();
    let ruined: usize = farms(5).map(|site| fence(&site)).sum();
    assert!(ruined * 10 < whole * 9, "{ruined} of {whole}");
}

#[test]
fn the_same_seed_makes_the_same_farm() {
    let (a, b) = (
        Site::new(PoiKind::Farm, Seed::new(7)),
        Site::new(PoiKind::Farm, Seed::new(7)),
    );
    assert_eq!(a.level(0), b.level(0));
}
