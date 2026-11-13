use super::super::grid::Reach;
use super::super::wall::WallMaterial;
use super::super::{Generator, PoiLook, Site, leads_out};
use super::arms::choose_arms;
use super::wall::INSIDE;
use super::*;
use crate::door::gate_partner;
use crate::dungeon::Cell;
use crate::poi::PoiKind;
use crate::rng::Seed;

/// Walled towns of every size, from a few seeds.
fn walled() -> impl Iterator<Item = (i64, u64, SiteLevel)> {
    [3, 5, 8, 12, 20, 30, 50, 80, 120, 150]
        .into_iter()
        .flat_map(|target| {
            (0..8).map(move |seed| {
                let seed = seed * 37 + 11;
                let mut rng = Seed::new(seed).derive("test arms").rng();
                let arms = choose_arms(&mut rng, target, [false; 4]);
                let town = Town::new(Seed::new(seed), target, arms, Some(WallMaterial::Stone));
                (target, seed, town.generate())
            })
        })
}

/// The wall: one ring of wall cells with gates of two doors, everything built
/// at least `INSIDE` cells inside it, 3 cells of level outside it, every gate
/// on a road, and the way in outside it with everything reachable.
#[test]
fn a_walled_town_has_its_ring_and_gates() {
    for (target, seed, site) in walled() {
        let [wall] = &site.walls[..] else {
            panic!("{target}, {seed}: one wall");
        };
        let level = &site.level;
        let rect = wall.rect;
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        assert_eq!(
            level.bounds(),
            (rect.x - 3, rect.y - 3, rect.width + 6, rect.height + 6),
            "{target}, {seed}: 3 cells outside"
        );
        assert!(!wall.gates.is_empty(), "{target}, {seed}: a gate");
        let (min_x, min_y, width, height) = level.bounds();
        for y in min_y..min_y + height {
            for x in min_x..min_x + width {
                let cell = level.cell(x, y);
                if wall.is_gate(x, y) {
                    assert_eq!(cell, Cell::Door, "{target}, {seed}: ({x}, {y})");
                    assert!(gate_partner(level, x, y).is_some());
                } else if wall.has(x, y) {
                    assert_eq!(cell, Cell::Wall, "{target}, {seed}: ({x}, {y})");
                }
                // Buildings and gardens keep their distance from it.
                if level.room(x, y).is_some() || cell == Cell::Garden {
                    let inner = (rect.x + 1 + INSIDE..x1 - INSIDE).contains(&x)
                        && (rect.y + 1 + INSIDE..y1 - INSIDE).contains(&y);
                    assert!(inner, "{target}, {seed}: ({x}, {y}) too near the wall");
                }
            }
        }
        for gate in &wall.gates {
            let [a, b] = gate.cells();
            let (dx, dy) = if gate.runs_east_west { (0, 1) } else { (1, 0) };
            let road = |(x, y): (i64, i64)| level.cell(x, y).is_road();
            let crosses = [a, b]
                .iter()
                .any(|&(x, y)| road((x - dx, y - dy)) && road((x + dx, y + dy)));
            assert!(crosses, "{target}, {seed}: {gate:?} on a road");
        }
        let entrance = site.entrance.unwrap();
        assert!(!rect.contains(entrance.0, entrance.1), "{target}, {seed}");
        let (ex, ey) = entrance;
        assert!(
            leads_out(level, ex, ey),
            "{target}, {seed}: {entrance:?} {:?} {:?}",
            level.cell(ex, ey),
            level.bounds()
        );
        let mut reach = Reach::new(level);
        reach.spread(level, &[entrance]);
        assert_eq!(reach.first_unreached(level), None, "{target}, {seed}");
    }
}

/// No road runs into the wall without a gate: a ring cell with road either
/// side of it is beside a gate (a road wider than two narrowed to its gate).
#[test]
fn no_road_is_walled_off() {
    for (target, seed, site) in walled() {
        let wall = &site.walls[0];
        let rect = wall.rect;
        let level = &site.level;
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                if !wall.on_ring(x, y) || wall.is_gate(x, y) {
                    continue;
                }
                let north_or_south = y == rect.y || y == rect.y + rect.height - 1;
                let (dx, dy) = if north_or_south { (0, 1) } else { (1, 0) };
                let through =
                    level.cell(x - dx, y - dy).is_road() && level.cell(x + dx, y + dy).is_road();
                let by_gate = (-2..=2).any(|d| wall.is_gate(x + d * dy, y + d * dx));
                assert!(!through || by_gate, "{target}, {seed}: ({x}, {y})");
            }
        }
    }
}

/// Smaller towns are walled more often, each size mostly in its own material.
#[test]
fn walls_by_size() {
    let count = 400;
    for (kind, percent, own) in [
        (PoiKind::Hamlet, 60, WallMaterial::Log),
        (PoiKind::Village, 45, WallMaterial::Plank),
        (PoiKind::Town, 30, WallMaterial::Stone),
        (PoiKind::City, 15, WallMaterial::Block),
    ] {
        let walls: Vec<WallMaterial> = (0..count)
            .filter_map(|n| {
                let look = PoiLook::of(kind, Seed::new(n * 7919 + 3), false, [false; 4]);
                let PoiLook::Town { wall, .. } = look else {
                    panic!("a town");
                };
                wall.map(|wall| wall.material)
            })
            .collect();
        let share = walls.len() as u64 * 100 / count;
        assert!(share.abs_diff(percent) <= 8, "{kind:?}: {share}% walled");
        let mine = walls.iter().filter(|&&m| m == own).count() * 100 / walls.len();
        assert!((36..=64).contains(&mine), "{kind:?}: {mine}% {own:?}");
    }
}

/// Main roads leave by every side roads come in from, the world map's wall
/// has its gates there, and the site's wall agrees with the look.
#[test]
fn main_roads_follow_the_roads_coming_in() {
    for n in 0..40 {
        let seed = Seed::new(n * 104_729 + 5);
        let sides = [n % 2 == 0, n % 3 == 0, n % 5 == 0, n % 7 == 0];
        for kind in [
            PoiKind::Hamlet,
            PoiKind::Village,
            PoiKind::Town,
            PoiKind::City,
        ] {
            let generator = Generator::for_site(kind, seed, false, sides);
            let Generator::Town(options) = generator else {
                panic!("a town");
            };
            let arms = Plan::new(&options, seed).arms();
            for side in 0..4 {
                assert!(
                    !sides[side] || arms[side],
                    "{kind:?} {n}: {sides:?} {arms:?}"
                );
            }
            assert!(arms.iter().filter(|&&arm| arm).count() >= 2);
            let PoiLook::Town { wall, .. } = PoiLook::of(kind, seed, false, sides) else {
                panic!("a town");
            };
            let site = Site::with_generator(kind, seed, generator.version(), generator).unwrap();
            let level = site.level(0);
            assert_eq!(wall.is_some(), !level.walls.is_empty(), "{kind:?} {n}");
            if let (Some(wall), Some(built)) = (wall, level.walls.first()) {
                assert_eq!(wall.gates, arms);
                assert_eq!(wall.material, built.material);
            }
        }
    }
}

/// One road coming in to a town of one straight road carries on out the far
/// side.
#[test]
fn one_side_on_a_straight_road_goes_through() {
    for n in 0..40 {
        let mut rng = Seed::new(n).rng();
        let arms = choose_arms(&mut rng, 3, [false, true, false, false]);
        assert_eq!(arms, [false, true, false, true]);
    }
}
