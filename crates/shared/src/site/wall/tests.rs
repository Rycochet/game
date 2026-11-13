use crate::door::gate_partner;
use crate::dungeon::Cell;
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::site::{Exit, PoiLook, Site, WallMaterial};

fn forts() -> impl Iterator<Item = (Seed, Site)> {
    (0..40).map(|n| {
        let seed = Seed::new(n * 104_729 + 7);
        (seed, Site::new(PoiKind::Fort, seed))
    })
}

/// About half of forts are walled, and every fort has a basement.
#[test]
fn half_of_forts_have_a_wall_and_all_a_basement() {
    let walled = forts()
        .filter(|(_, site)| !site.level(0).walls.is_empty())
        .count();
    assert!((10..=30).contains(&walled), "{walled} of 40 walled");
    assert!(forts().all(|(_, site)| site.has_basement()));
}

/// A fort's wall: one cell of wall all round, its gate two doors in the middle
/// of the south side, 2 to 8 cells of ground (more by a pond) from the
/// building, arriving outside the gate, a pond inside, and the same on the
/// world map.
#[test]
fn a_forts_wall_rings_it_with_a_gate_south() {
    let mut materials = Vec::new();
    for (seed, site) in forts() {
        let ground = site.level(0);
        let Some(wall) = ground.walls.first() else {
            continue;
        };
        let level = &ground.level;
        let rect = wall.rect;
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        for y in rect.y..=y1 {
            for x in rect.x..=x1 {
                let cell = level.cell(x, y);
                if wall.is_gate(x, y) {
                    assert_eq!(cell, Cell::Door);
                    assert!(gate_partner(level, x, y).is_some());
                } else if wall.has(x, y) {
                    assert_eq!(cell, Cell::Wall, "({x}, {y})");
                }
            }
        }
        let [gate] = wall.gates[..] else {
            panic!("one gate");
        };
        assert_eq!(gate.y, y1);
        assert!((gate.x - rect.x - (x1 - gate.x - 1)).abs() <= 1, "centred");
        assert_eq!(ground.arrival(Exit::WorldMap), Some((gate.x, y1 + 1)));
        let water = (rect.y..=y1)
            .flat_map(|y| (rect.x..=x1).map(move |x| (x, y)))
            .filter(|&(x, y)| level.cell(x, y) == Cell::Water)
            .count();
        assert!((1..=9).contains(&water), "{water} cells of pond");
        let PoiLook::Building {
            wall: Some(look), ..
        } = PoiLook::of(PoiKind::Fort, seed, false, [false; 4])
        else {
            panic!("the world map shows the wall");
        };
        for gap in [look.west, look.north, look.east, look.south] {
            assert!((2..=8).contains(&gap), "gap {gap}");
        }
        assert_eq!(look.gates, [false, false, true, false]);
        assert_eq!(look.material, wall.material);
        materials.push(wall.material);
    }
    assert!(materials.contains(&WallMaterial::Plank));
    assert!(materials.contains(&WallMaterial::Stone));
    assert!(!materials.contains(&WallMaterial::Block));
}

/// No two doors stand side by side anywhere but in a gate.
#[test]
fn doors_are_only_side_by_side_in_gates() {
    for kind in PoiKind::ALL {
        for n in 0..4 {
            let site = Site::new(kind, Seed::new(n * 7919 + 13));
            for index in 0..site.levels() {
                let site_level = site.level(index);
                let level = &site_level.level;
                let (min_x, min_y, width, height) = level.bounds();
                for y in min_y..min_y + height {
                    for x in min_x..min_x + width {
                        let gate = site_level.walls.iter().any(|w| w.is_gate(x, y));
                        assert!(
                            gate || gate_partner(level, x, y).is_none(),
                            "{kind:?} level {index}: doors side by side at ({x}, {y})"
                        );
                    }
                }
            }
        }
    }
}
