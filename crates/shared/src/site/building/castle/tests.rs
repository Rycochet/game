use crate::door::gate_partner;
use crate::dungeon::{Cell, Level};
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::site::grid::Reach;
use crate::site::{
    BUILDING_VERSION, Exit, Generator, PoiLook, Site, SiteLevel, WallLook, WallMaterial,
    tower_spots,
};

fn seeds() -> impl Iterator<Item = Seed> {
    (0..30).map(|n| Seed::new(n * 104_729 + 11))
}

fn castles() -> impl Iterator<Item = (Seed, Site)> {
    seeds().map(|seed| (seed, Site::new(PoiKind::Castle, seed)))
}

fn ruins(ruin: u8) -> impl Iterator<Item = Site> {
    seeds().map(move |seed| {
        let generator = Generator::for_poi(PoiKind::Castle).with_ruin(ruin);
        Site::with_generator(PoiKind::Castle, seed, BUILDING_VERSION, generator).unwrap()
    })
}

fn cells(level: &Level) -> impl Iterator<Item = (i64, i64)> {
    let (min_x, min_y, width, height) = level.bounds();
    (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
}

fn wall(site: &Site) -> WallLook {
    let walls = site.level(0).walls;
    assert_eq!(walls.len(), 1, "a castle always has a wall");
    walls[0].clone()
}

/// Every castle: a wall of stone or blocks round 40–70 × 36–60, its cells wall
/// but for the gate in the middle of the south side and the towers on it; you
/// arrive outside the gate.
#[test]
fn every_castle_is_walled_with_a_gate_south() {
    let mut materials = Vec::new();
    for (_, site) in castles() {
        let wall = wall(&site);
        let ground = site.level(0);
        let level = &ground.level;
        let rect = wall.rect;
        assert!((40..=70).contains(&rect.width) && (36..=60).contains(&rect.height));
        let y1 = rect.y + rect.height - 1;
        for (x, y) in cells(level).filter(|&(x, y)| wall.has(x, y)) {
            if wall.is_gate(x, y) {
                assert_eq!(level.cell(x, y), Cell::Door);
                assert!(gate_partner(level, x, y).is_some());
            } else {
                assert_eq!(level.cell(x, y), Cell::Wall, "({x}, {y})");
            }
        }
        let [gate] = wall.gates[..] else {
            panic!("one gate");
        };
        assert_eq!(gate.y, y1);
        let (west, east) = (gate.x - rect.x, rect.x + rect.width - 2 - gate.x);
        assert!((west - east).abs() <= 1, "centred");
        let (x, y) = ground.arrival(Exit::WorldMap).unwrap();
        assert_eq!(x, gate.x);
        assert!(y > y1, "outside the gate");
        materials.push(wall.material);
    }
    assert!(materials.contains(&WallMaterial::Stone));
    assert!(materials.contains(&WallMaterial::Block));
    assert!(materials.iter().all(|m| !m.is_wood()));
}

/// Towers on every corner and either side of the gate, each with a door on
/// the ground floor and floors of 1 to 3, a room on each.
#[test]
fn towers_stand_on_the_corners_and_by_the_gate() {
    for (_, site) in castles() {
        let wall = wall(&site);
        let rect = wall.rect;
        let spots = tower_spots(rect);
        assert_eq!(spots.iter().filter(|spot| spot.by_gate).count(), 2);
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        for corner in [(rect.x, rect.y), (x1, rect.y), (rect.x, y1), (x1, y1)] {
            assert!(spots.iter().any(|spot| (spot.x, spot.y) == corner));
        }
        assert!(spots.len() > 6, "towers along a long wall");
        let ground = site.level(0).level;
        for spot in &spots {
            let tower = spot.rect();
            let doors = cells(&ground)
                .filter(|&(x, y)| tower.contains(x, y) && ground.cell(x, y) == Cell::Door)
                .count();
            assert_eq!(doors, 1, "a door into each tower");
            let floors = (0..site.levels())
                .filter(|&index| {
                    let level = site.level(index).level;
                    level.cell(spot.x, spot.y) == Cell::Floor
                        && level.room(spot.x, spot.y).is_some()
                })
                .count();
            assert!((1..=3).contains(&floors), "{floors} floors");
        }
    }
}

/// 5 to 8 cells (more on the pond's side) from the wall to the keep, a pond
/// inside, and a moat round some, with a bridge from the gate; the world map
/// is told of it all.
#[test]
fn a_courtyard_pond_and_sometimes_a_moat() {
    let (mut moats, mut ponds) = (0, 0);
    for (seed, site) in castles() {
        let wall = wall(&site);
        let level = site.level(0).level;
        let rect = wall.rect;
        let inside = |x: i64, y: i64| rect.contains(x, y);
        let water = |inner: bool| {
            cells(&level)
                .filter(|&(x, y)| inside(x, y) == inner && level.cell(x, y) == Cell::Water)
                .count()
        };
        let pond = water(true);
        assert!(pond <= 16, "{pond} cells of pond");
        ponds += usize::from(pond > 0);
        let gate = wall.gates[0];
        let bridge = (gate.y + 1..gate.y + 8).any(|y| level.cell(gate.x, y) == Cell::Bridge);
        let PoiLook::Building {
            wall: Some(look),
            castle: Some(castle),
            ..
        } = PoiLook::of(PoiKind::Castle, seed, false, [false; 4])
        else {
            panic!("the world map shows the castle");
        };
        assert_eq!(bridge, castle.moat > 0);
        assert_eq!(water(false) > 0, castle.moat > 0);
        moats += usize::from(castle.moat > 0);
        assert!((2..=4).contains(&castle.moat) || castle.moat == 0);
        for gap in [look.west, look.north, look.east, look.south] {
            assert!((5..=8).contains(&gap), "gap {gap}");
        }
        assert_eq!(look.gates, [false, false, true, false]);
        assert!((2..=3).contains(&castle.tower_floors));
    }
    assert!(ponds >= 28, "{ponds} of 30 with a pond");
    assert!((8..=26).contains(&moats), "{moats} of 30 with a moat");
}

/// A ruin's wall has gaps (ground), and its towers may have lost their tops;
/// everything walkable on the ground is still reached from where you arrive.
#[test]
fn ruins_have_gaps_and_fallen_towers() {
    let tallest = |site: &Site| {
        tower_spots(site.level(0).walls[0].rect)
            .iter()
            .map(|spot| {
                (0..site.levels())
                    .filter(|&i| site.level(i).level.room(spot.x, spot.y).is_some())
                    .count()
            })
            .sum::<usize>()
    };
    let intact: usize = castles().map(|(_, site)| tallest(&site)).sum();
    let fallen: usize = ruins(5).map(|site| tallest(&site)).sum();
    assert!(fallen < intact, "{fallen} tower floors left of {intact}");
    for ruin in [1, 3, 5] {
        for site in ruins(ruin) {
            let ground: SiteLevel = site.level(0);
            let wall = &ground.walls[0];
            assert!(!wall.gaps.is_empty());
            for &(x, y) in &wall.gaps {
                assert!(wall.on_ring(x, y) && !wall.has(x, y));
                assert_eq!(ground.level.cell(x, y), Cell::Ground);
            }
            let level = &ground.level;
            let mut reach = Reach::new(level);
            reach.spread(level, &[ground.arrival(Exit::WorldMap).unwrap()]);
            assert_eq!(reach.first_unreached(level), None, "ruin {ruin}");
        }
    }
}
