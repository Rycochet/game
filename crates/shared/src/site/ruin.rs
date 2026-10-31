//! Ruined buildings: damage done to a building's finished floors.
//!
//! The floorplan stays what it would be unruined; the damage has its own seed
//! and only ever takes cells away. Each floor depends on the one below (a floor
//! can't stand over a gap), so they are ruined from the ground up, together.
//!
//! Damage nibbles in from the edges: every cell next to a gap is eaten at
//! 10 × ruin %, then the neighbours of what was eaten at 10% less, and so on
//! until the chance is 0. That alone leaves the middle of a big building whole,
//! so the worst ruin also starts holes in the middle of its floors. On the
//! ground floor eaten cells become outside ground; above it they become holes,
//! and each storey counts as one level less ruined than the one below.

use std::collections::BTreeSet;

use super::building::Shape;
use super::grid::{self, Reach, STEPS, chance};
use super::{SiteLevel, StairsDirection};
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

/// The most ruined a building can be.
pub const MAX_RUIN: u8 = 5;

/// The chance of being eaten in the first pass, per level of ruin, in percent;
/// and how much less it is each pass after.
const PERCENT_PER_RUIN: u32 = 10;

/// Floor cells per hole started in the middle of a floor, by the ruin left on
/// that floor. Less ruined floors start none.
fn floor_cells_per_hole(ruin: u8) -> Option<i64> {
    match ruin {
        4 => Some(100),
        5.. => Some(40),
        _ => None,
    }
}

/// How far (counting diagonals as 1) a floor can be from a wall of the floor
/// below before it comes down.
const MAX_SPAN: i64 = 4;

/// Whether a cell has nothing there: outside ground, or a hole.
fn is_gap(cell: Cell) -> bool {
    cell.is_outside() || cell == Cell::Hole
}

/// Whether a staircase, or the cells round it, are at `(x, y)`.
fn near(stairs: (i64, i64), x: i64, y: i64) -> bool {
    (stairs.0 - x).abs() <= 1 && (stairs.1 - y).abs() <= 1
}

/// The cells of a level, row by row.
fn cells_of(level: &Level) -> impl Iterator<Item = (i64, i64)> + use<> {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
}

/// Ruins a building's floors (all of them, intact, from the ground up), given
/// each floor's outline and `stairs[n]` joining floor `n` to floor `n + 1`, and
/// any stairs from the ground floor down to a basement. Floors above one with
/// no way left from its stairs down to any stairs up are dropped.
pub(super) fn ruin_floors(
    seed: Seed,
    ruin: u8,
    floors: Vec<SiteLevel>,
    shapes: &[Shape],
    stairs: &[Vec<(i64, i64)>],
    basement: Option<(i64, i64)>,
) -> Vec<SiteLevel> {
    let seed = seed.derive("ruin");
    let mut kept: Vec<SiteLevel> = Vec::new();
    for (index, (mut floor, shape)) in floors.into_iter().zip(shapes).enumerate() {
        // A staircase keeps the cells round it on its own floors and below.
        let protected = |x: i64, y: i64| {
            stairs
                .iter()
                .enumerate()
                .any(|(n, spots)| index <= n + 1 && spots.iter().any(|&s| near(s, x, y)))
                || (index == 0 && basement.is_some_and(|s| near(s, x, y)))
        };
        let below = kept.last().map(|floor| &floor.level);
        let level = &mut floor.level;
        if let Some(below) = below {
            for (x, y) in cells_of(level) {
                if !shape.contains(x, y) || is_gap(below.cell(x, y)) {
                    make_hole(level, x, y);
                }
            }
        }
        let eaten_to = if index == 0 { Cell::Ground } else { Cell::Hole };
        let left = ruin.saturating_sub(index as u8);
        let mut rng = Seed::new(seed.at(&[index as i64])).rng();
        nibble(level, &mut rng, left, eaten_to, &protected);
        open_lone_doors(level);
        if let Some(below) = below {
            for (x, y) in cells_of(level) {
                if level.is_walkable(x, y) && !protected(x, y) && !supported(below, x, y) {
                    make_hole(level, x, y);
                }
            }
        }
        let connected = stairs_connect(&floor);
        kept.push(floor);
        if !connected {
            let top = kept.last_mut().expect("just added");
            top.stairs.retain(|s| s.direction != StairsDirection::Up);
            break;
        }
    }
    kept
}

/// Turns a cell into a gap in the floor.
fn make_hole(level: &mut Level, x: i64, y: i64) {
    level.set(x, y, Cell::Hole);
    level.set_room(x, y, None);
}

/// Whether the floor below holds up a floor at `(x, y)`: it isn't over a gap,
/// and a wall (or a door in one) is near enough to carry it.
fn supported(below: &Level, x: i64, y: i64) -> bool {
    !is_gap(below.cell(x, y))
        && (-MAX_SPAN..=MAX_SPAN).any(|dy| {
            (-MAX_SPAN..=MAX_SPAN).any(|dx| {
                let (wx, wy) = (x + dx, y + dy);
                let cell = below.cell(wx, wy);
                below.contains(wx, wy) && (cell.is_wall() || cell.is_opening())
            })
        })
}

/// Eats the level's edges, and holes started in its middle, away to `eaten_to`.
/// `ruin` is the ruin left on this floor.
fn nibble(
    level: &mut Level,
    rng: &mut Rng,
    ruin: u8,
    eaten_to: Cell,
    protected: &impl Fn(i64, i64) -> bool,
) {
    let can_eat = |level: &Level, x: i64, y: i64| {
        level.contains(x, y) && !is_gap(level.cell(x, y)) && !protected(x, y)
    };
    let mut percent = u32::from(ruin.min(MAX_RUIN)) * PERCENT_PER_RUIN;
    if percent == 0 {
        return;
    }
    let beside_gap = |x: i64, y: i64| {
        STEPS
            .iter()
            .any(|(dx, dy)| level.contains(x + dx, y + dy) && is_gap(level.cell(x + dx, y + dy)))
    };
    let mut next: BTreeSet<(i64, i64)> = cells_of(level)
        .map(|(x, y)| (y, x))
        .filter(|&(y, x)| can_eat(level, x, y) && beside_gap(x, y))
        .collect();
    let mut started = start_holes(level, rng, ruin, protected);
    while percent > 0 {
        let mut eaten = std::mem::take(&mut started);
        for (y, x) in next {
            if chance(rng, percent) && !eaten.contains(&(y, x)) {
                eaten.push((y, x));
            }
        }
        for &(y, x) in &eaten {
            level.set(x, y, eaten_to);
            level.set_room(x, y, None);
        }
        next = eaten
            .iter()
            .flat_map(|&(y, x)| STEPS.map(|(dx, dy)| (y + dy, x + dx)))
            .filter(|&(y, x)| can_eat(level, x, y))
            .collect();
        percent -= PERCENT_PER_RUIN;
    }
}

/// Picks where holes start in the middle of a floor, as `(y, x)`: on floor, so
/// that what is eaten from them on the ground floor can be walked to.
fn start_holes(
    level: &Level,
    rng: &mut Rng,
    ruin: u8,
    protected: &impl Fn(i64, i64) -> bool,
) -> Vec<(i64, i64)> {
    let Some(per_hole) = floor_cells_per_hole(ruin) else {
        return Vec::new();
    };
    let mut floor: Vec<(i64, i64)> = cells_of(level)
        .filter(|&(x, y)| level.cell(x, y) == Cell::Floor && !protected(x, y))
        .map(|(x, y)| (y, x))
        .collect();
    // Part of a hole's share of the floor makes one that often.
    let area = floor.len() as i64;
    let extra = grid::roll(rng, 0, per_hole - 1) < area % per_hole;
    let holes = area / per_hole + i64::from(extra);
    let mut started = Vec::new();
    for _ in 0..holes {
        if floor.is_empty() {
            break;
        }
        started.push(floor.swap_remove(rng.range_u64(0, floor.len() as u64) as usize));
    }
    started
}

/// A door with no wall left beside it is only floor.
fn open_lone_doors(level: &mut Level) {
    for (x, y) in cells_of(level) {
        let stands = |&(dx, dy): &(i64, i64)| {
            let cell = level.cell(x + dx, y + dy);
            level.contains(x + dx, y + dy) && (cell.is_wall() || cell.is_opening())
        };
        if level.cell(x, y) == Cell::Door && !STEPS.iter().any(stands) {
            level.set(x, y, Cell::Floor);
        }
    }
}

/// Whether a way you arrive on a floor can still be walked to some stairs up.
fn stairs_connect(floor: &SiteLevel) -> bool {
    let at = |direction| {
        floor
            .stairs
            .iter()
            .filter(move |s| s.direction == direction)
            .map(|s| (s.x, s.y))
    };
    let ups: Vec<_> = at(StairsDirection::Up).collect();
    // (A basement's stairs are no way in: come by the entrance.)
    let mut ways_in: Vec<_> = at(StairsDirection::Down)
        .filter(|_| floor.entrance.is_none())
        .collect();
    ways_in.extend(floor.entrance.filter(|_| ways_in.is_empty()));
    if ups.is_empty() || ways_in.is_empty() {
        return true;
    }
    let mut reach = Reach::new(&floor.level);
    reach.spread(&floor.level, &ways_in);
    ups.iter().any(|&(x, y)| reach.is(x, y))
}

#[cfg(test)]
mod tests {
    use super::super::building::{BuildingOptions, Outline, Plan};
    use super::super::{Exit, Generator, Stairs};
    use super::*;
    use crate::poi::PoiKind;

    const KINDS: [PoiKind; 5] = [
        PoiKind::Castle,
        PoiKind::Fort,
        PoiKind::StoneTower,
        PoiKind::RedRoofCottage,
        PoiKind::StonePyramid,
    ];

    fn options(kind: PoiKind, ruin: u8) -> BuildingOptions {
        let Generator::Building(options) = Generator::for_poi(kind).with_ruin(ruin) else {
            panic!("{kind:?} isn't a building");
        };
        options
    }

    /// Every kind of plan tested, at every level of ruin, with its intact one.
    fn ruined_plans(seeds: u64) -> Vec<(u8, Plan, Plan)> {
        let mut plans = Vec::new();
        for kind in KINDS {
            for seed in 0..seeds {
                let seed = Seed::new(seed * 6151 + 3);
                let intact = Plan::new(&options(kind, 0), seed);
                for ruin in 1..=MAX_RUIN {
                    plans.push((ruin, Plan::new(&options(kind, ruin), seed), intact.clone()));
                }
            }
        }
        plans
    }

    fn floors(plan: &Plan) -> Vec<SiteLevel> {
        (0..plan.floors()).map(|n| plan.level(n)).collect()
    }

    /// No ruin is exactly a normal building.
    #[test]
    fn no_ruin_changes_nothing() {
        let (plain, seed) = (options(PoiKind::Castle, 0), Seed::new(9));
        assert_eq!(plain.ruin, 0);
        let plan = Plan::new(&plain, seed);
        assert!(plan.floors() >= 1);
        assert_eq!(plan.level(0), Plan::new(&plain, seed).level(0));
        assert!(
            cells_of(&plan.level(0).level)
                .all(|(x, y)| plan.level(0).level.cell(x, y) != Cell::Hole)
        );
    }

    /// Damage only takes cells away: anything left is as the plan made it.
    #[test]
    fn ruin_keeps_the_floorplan() {
        for (ruin, plan, intact) in ruined_plans(2) {
            for (n, floor) in floors(&plan).iter().enumerate() {
                let whole = intact.level(n as u32);
                for (x, y) in cells_of(&floor.level) {
                    let (now, then) = (floor.level.cell(x, y), whole.level.cell(x, y));
                    let lone_door = then == Cell::Door && now == Cell::Floor;
                    // Windows go in after ruin, wherever a wall then fits one.
                    let window_moved = matches!(
                        (then, now),
                        (Cell::Window, Cell::Wall) | (Cell::Wall, Cell::Window)
                    );
                    assert!(
                        now == then || is_gap(now) || lone_door || window_moved,
                        "ruin {ruin} floor {n}: {then:?} became {now:?}"
                    );
                }
            }
        }
    }

    /// The ground floor loses cells to outside ground, in no room, with no
    /// holes, and all of it can still be walked to.
    #[test]
    fn eaten_ground_floor_is_ground_and_connected() {
        for (ruin, plan, intact) in ruined_plans(3) {
            let (floor, whole) = (plan.level(0), intact.level(0));
            let level = &floor.level;
            let mut eaten = 0;
            for (x, y) in cells_of(level) {
                assert_ne!(level.cell(x, y), Cell::Hole);
                if level.cell(x, y) == Cell::Ground && whole.level.cell(x, y) != Cell::Ground {
                    eaten += 1;
                    assert_eq!(level.room(x, y), None);
                }
            }
            assert!(eaten > 0, "ruin {ruin}: nothing eaten");
            let start = floor.arrival(Exit::WorldMap).expect("a way in");
            assert!(grid::all_reached(level, &[start]), "ruin {ruin}");
        }
    }

    /// The worst ruin reaches into the middle of big buildings too.
    #[test]
    fn worst_ruin_opens_up_the_middle_of_a_castle() {
        let seed = Seed::new(21);
        let plan = Plan::new(&options(PoiKind::Castle, 5), seed);
        let intact = Plan::new(&options(PoiKind::Castle, 0), seed);
        let (level, whole) = (plan.level(0).level, intact.level(0).level);
        let (_, _, width, height) = level.bounds();
        let middle = (width / 3..width * 2 / 3)
            .flat_map(|x| (height / 3..height * 2 / 3).map(move |y| (x, y)))
            .filter(|&(x, y)| level.cell(x, y) == Cell::Ground && whole.cell(x, y) != Cell::Ground);
        assert!(middle.count() > 0);
    }

    /// Going up: over a gap is a hole, walls included; floors have support;
    /// past the outline is open air; nothing above the ground floor is ground.
    #[test]
    fn upper_floors_stand_on_what_is_below() {
        for (ruin, plan, intact) in ruined_plans(3) {
            let all = floors(&plan);
            for n in 1..all.len() {
                let (below, level) = (&all[n - 1].level, &all[n].level);
                let whole = intact.level(n as u32).level;
                let stairs: Vec<_> = (n - 1..all.len() - 1)
                    .chain(n..n)
                    .flat_map(|k| {
                        let up = Exit::Level(k as u32 + 1);
                        let stairs = all[k].stairs.iter().filter(move |s| s.to == up);
                        stairs.map(|s| (s.x, s.y))
                    })
                    .collect();
                for (x, y) in cells_of(level) {
                    let cell = level.cell(x, y);
                    assert!(!cell.is_outside(), "ruin {ruin} floor {n}: {cell:?}");
                    if is_gap(below.cell(x, y)) {
                        assert_eq!(cell, Cell::Hole, "ruin {ruin} floor {n} ({x}, {y})");
                    }
                    // Wall in the plan, on the border round any outline.
                    if x < 3 || y < 3 {
                        assert_eq!(whole.cell(x, y), Cell::Wall);
                        assert_eq!(cell, Cell::Hole);
                    }
                    if cell.is_walkable() && !stairs.iter().any(|&s| near(s, x, y)) {
                        assert!(supported(below, x, y), "ruin {ruin} floor {n} ({x}, {y})");
                    }
                }
            }
        }
    }

    /// The cells round a staircase stand whole on its floors and all below,
    /// and every floor kept can be walked from its arrival to its stairs up.
    #[test]
    fn stairs_are_kept_and_connect() {
        for (ruin, plan, intact) in ruined_plans(3) {
            let all = floors(&plan);
            for (n, floor) in all.iter().enumerate() {
                let least = usize::from(n > 0) + usize::from(n + 1 < all.len());
                assert!(floor.stairs.len() >= least, "ruin {ruin} floor {n}");
                assert!(stairs_connect(floor), "ruin {ruin} floor {n}");
                for stairs in &floor.stairs {
                    assert_eq!(floor.level.cell(stairs.x, stairs.y), Cell::Floor);
                    for (below, lower) in all.iter().enumerate().take(n + 1) {
                        let whole = intact.level(below as u32).level;
                        for (dx, dy) in (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))) {
                            let (x, y) = (stairs.x + dx, stairs.y + dy);
                            assert_eq!(
                                lower.level.cell(x, y).is_wall(),
                                whole.cell(x, y).is_wall()
                            );
                            assert!(!is_gap(lower.level.cell(x, y)));
                        }
                    }
                }
            }
        }
    }

    /// A floor whose stairs can't be walked between is the top one kept.
    #[test]
    fn floors_above_broken_stairs_are_dropped() {
        // Upper floors of a bare room: a hole right across cuts one stair off.
        let mut level = Level::new(0, 0, 9, 3);
        for x in 0..9 {
            level.set(x, 1, Cell::Floor);
        }
        let stairs = |x, direction| Stairs {
            x,
            y: 1,
            direction,
            to: Exit::Level(0),
        };
        let mut floor = SiteLevel {
            level,
            stairs: vec![
                stairs(1, StairsDirection::Down),
                stairs(7, StairsDirection::Up),
            ],
            entrance: None,
            buildings: Vec::new(),
            fields: Vec::new(),
        };
        assert!(stairs_connect(&floor));
        floor.level.set(4, 1, Cell::Hole);
        assert!(!stairs_connect(&floor));

        // And real buildings do lose floors this way.
        let dropped = ruined_plans(8)
            .iter()
            .any(|(_, plan, intact)| plan.floors() < intact.floors());
        assert!(dropped);
    }

    /// More ruin does more damage, on the whole.
    #[test]
    fn more_ruin_eats_more() {
        let eaten = |ruin: u8| -> usize {
            (0..6)
                .map(|seed| {
                    let plan = Plan::new(&options(PoiKind::Fort, ruin), Seed::new(seed));
                    let level = plan.level(0).level;
                    cells_of(&level)
                        .filter(|&(x, y)| level.cell(x, y) == Cell::Ground)
                        .count()
                })
                .sum()
        };
        for ruin in 1..MAX_RUIN {
            assert!(eaten(ruin) < eaten(ruin + 1), "ruin {ruin}");
        }
    }

    #[test]
    fn outlines_all_survive_ruin() {
        for outlines in [
            &[Outline::Rectangle][..],
            &[Outline::CutCorners],
            &[Outline::RoundCorners],
            &[Outline::Circle],
        ] {
            let options = BuildingOptions {
                outlines,
                ..options(PoiKind::Castle, 5)
            };
            for seed in 0..4 {
                assert!(Plan::new(&options, Seed::new(seed)).floors() >= 1);
            }
        }
    }
}
