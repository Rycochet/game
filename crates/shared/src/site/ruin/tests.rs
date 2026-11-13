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
    // The building's own ruin: a castle's keep, without the castle round it.
    BuildingOptions {
        castle: None,
        ..options
    }
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

/// Its floors from the ground up, without any basement (forts have one).
fn floors(plan: &Plan) -> Vec<SiteLevel> {
    let storeys = plan.floors() - u32::from(plan.has_basement());
    (0..storeys).map(|n| plan.level(n)).collect()
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
        cells_of(&plan.level(0).level).all(|(x, y)| plan.level(0).level.cell(x, y) != Cell::Hole)
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
                        assert_eq!(lower.level.cell(x, y).is_wall(), whole.cell(x, y).is_wall());
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
        walls: Vec::new(),
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
