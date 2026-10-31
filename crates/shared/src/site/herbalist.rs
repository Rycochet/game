//! Herbalists' huts: a small building of one storey over a basement that may
//! be bigger, with one herb patch outside: parallel strips of `Cell::HerbBed`
//! with bare ground to walk between them. What grows there is the client's to
//! choose; it is only looks.

use super::SiteLevel;
use super::building::{self, BuildingOptions};
use super::farm::{Plot, surroundings};
use super::grid::roll;
use crate::dungeon::{Cell, Level};
use crate::rng::Seed;

/// Options for the herbalist's hut generator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HerbalistOptions {
    pub house: BuildingOptions,
    /// How many strips the patch has, and how long they are, inclusive.
    pub strips: (i64, i64),
    pub length: (i64, i64),
}

/// A hut's plan. Its ground floor is made at once and kept, as a farm's is.
#[derive(Clone, Debug)]
pub struct Plan {
    house: building::Plan,
    ground: SiteLevel,
}

impl Plan {
    pub fn new(options: &HerbalistOptions, seed: Seed) -> Self {
        let house = building::Plan::new(&options.house, seed);
        let ground = lay_out(options, seed, &house);
        Self { house, ground }
    }

    pub fn house(&self) -> &building::Plan {
        &self.house
    }

    pub fn level(&self, index: u32) -> SiteLevel {
        match index {
            0 => self.ground.clone(),
            _ => self.house.level(index),
        }
    }
}

/// The ground floor with its patch, on a side of the hut away from its door.
fn lay_out(options: &HerbalistOptions, seed: Seed, house: &building::Plan) -> SiteLevel {
    let mut rng = seed.derive("herb patch").rng();
    let inner = house.level(0);
    let (_, _, width, height) = inner.level.bounds();
    let door = inner.entrance.expect("a hut has a way in");

    let strips = roll(&mut rng, options.strips.0, options.strips.1);
    let length = roll(&mut rng, options.length.0, options.length.1);
    let across = 2 * strips - 1;
    let east_west = rng.range_u64(0, 2) == 0;
    let (wide, tall) = if east_west {
        (length, across)
    } else {
        (across, length)
    };
    // The sides not nearest the door: right up against the yard round the hut.
    let by_door = |(dx, dy): (i64, i64)| {
        let (x, y) = (door.0 - width / 2, door.1 - height / 2);
        let (toward, other) = (x * dx + y * dy, (x * dy).abs() + (y * dx).abs());
        toward > other
    };
    let sides: Vec<(i64, i64)> = [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .filter(|&side| !by_door(side))
        .collect();
    let side = sides[rng.range_u64(0, sides.len() as u64) as usize];
    let along = |rng: &mut _, room: i64, size: i64| roll(rng, -size / 2, room - size / 2);
    let x0 = match side.0 {
        -1 => -wide,
        1 => width,
        _ => along(&mut rng, width, wide),
    };
    let y0 = match side.1 {
        -1 => -tall,
        1 => height,
        _ => along(&mut rng, height, tall),
    };
    let plot = Plot {
        x0,
        y0,
        x1: x0 + wide,
        y1: y0 + tall,
    };

    let mut level: Level = surroundings(&inner.level, &[plot]);
    for y in plot.y0..plot.y1 {
        for x in plot.x0..plot.x1 {
            let strip = if east_west { y - plot.y0 } else { x - plot.x0 };
            if strip % 2 == 0 {
                level.set(x, y, Cell::HerbBed);
            }
        }
    }
    SiteLevel { level, ..inner }
}

#[cfg(test)]
mod tests {
    use super::super::grid::Reach;
    use super::super::{Generator, Site, StairsDirection, leads_out};
    use super::*;
    use crate::poi::PoiKind;

    fn huts(ruin: u8) -> impl Iterator<Item = Site> {
        (0..200).map(move |n| {
            let kind = PoiKind::HerbalistHut;
            let generator = Generator::for_poi(kind).with_ruin(ruin);
            Site::with_generator(kind, Seed::new(n * 41 + 3), generator.version(), generator)
                .expect("the current version runs")
        })
    }

    fn cells_of(level: &Level) -> impl Iterator<Item = (i64, i64)> + use<> {
        let (min_x, min_y, width, height) = level.bounds();
        (min_y..min_y + height).flat_map(move |y| (min_x..min_x + width).map(move |x| (x, y)))
    }

    fn box_of(cells: impl Iterator<Item = (i64, i64)>) -> (i64, i64, i64, i64) {
        cells.fold((i64::MAX, i64::MAX, i64::MIN, i64::MIN), |b, (x, y)| {
            (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y))
        })
    }

    /// One storey of 5 to 8 a side, and always a basement, sometimes bigger.
    #[test]
    fn huts_are_small_over_a_basement() {
        let mut bigger = 0;
        for site in huts(0) {
            assert_eq!(site.levels(), 2);
            assert!(site.has_basement());
            let ground = site.level(0);
            let walls = |level: &Level| {
                let cells: Vec<_> = cells_of(level)
                    .filter(|&(x, y)| level.cell(x, y).is_wall() || level.cell(x, y).is_opening())
                    .collect();
                box_of(cells.into_iter())
            };
            let (x0, y0, x1, y1) = walls(&ground.level);
            assert!((5..=8).contains(&(x1 - x0 + 1)) && (5..=8).contains(&(y1 - y0 + 1)));

            let below = site.level(1);
            let open = cells_of(&below.level).filter(|&(x, y)| below.level.is_walkable(x, y));
            let (bx0, by0, bx1, by1) = box_of(open);
            if bx1 - bx0 > x1 - x0 || by1 - by0 > y1 - y0 {
                bigger += 1;
            }
            let up = below.stairs[0];
            assert_eq!(up.direction, StairsDirection::Up);
            let mut reach = Reach::new(&below.level);
            reach.spread(&below.level, &[(up.x, up.y)]);
            assert!(
                cells_of(&below.level)
                    .all(|(x, y)| !below.level.is_walkable(x, y) || reach.is(x, y))
            );
            let down = &ground.stairs[0];
            assert_eq!((down.x, down.y), (up.x, up.y));
        }
        assert!(bigger > 100, "{bigger}");
    }

    /// 3 to 5 straight parallel strips of 7 to 12, with ground between them,
    /// clear of the hut, reachable, and the way out still there.
    #[test]
    fn the_patch_is_parallel_strips() {
        let mut counts = [0; 6];
        for ruin in [0, 5] {
            for site in huts(ruin) {
                let ground = site.level(0);
                let level = &ground.level;
                let bed = |x, y| level.cell(x, y) == Cell::HerbBed;
                let beds: Vec<_> = cells_of(level).filter(|&(x, y)| bed(x, y)).collect();
                let (x0, y0, x1, y1) = box_of(beds.iter().copied());
                let (wide, tall) = (x1 - x0 + 1, y1 - y0 + 1);
                // Which way the strips run: along the sides they fill.
                let east_west = (x0..=x1).all(|x| bed(x, y0));
                let (across, length) = if east_west {
                    (tall, wide)
                } else {
                    (wide, tall)
                };
                assert_eq!(across % 2, 1);
                let strips = (across + 1) / 2;
                assert!((3..=5).contains(&strips), "{strips}");
                assert!((7..=12).contains(&length), "{length}");
                counts[strips as usize] += 1;
                assert_eq!(beds.len() as i64, strips * length);
                for &(x, y) in &beds {
                    let line = if east_west { y - y0 } else { x - x0 };
                    assert_eq!(line % 2, 0);
                    for (dx, dy) in (-1..=1).flat_map(|d| (-1..=1).map(move |e| (d, e))) {
                        let near = level.cell(x + dx, y + dy);
                        assert!(!near.is_wall() && !near.is_opening() && near != Cell::Floor);
                    }
                }
                let mut reach = Reach::new(level);
                reach.spread(level, &[ground.entrance.expect("a way in")]);
                assert!(cells_of(level).all(|(x, y)| !level.is_walkable(x, y) || reach.is(x, y)));
                assert!(cells_of(level).any(|(x, y)| leads_out(level, x, y)));
            }
        }
        assert!(
            counts[3] > 0 && counts[4] > 0 && counts[5] > 0,
            "{counts:?}"
        );
    }

    #[test]
    fn the_same_seed_makes_the_same_hut() {
        let site = || Site::new(PoiKind::HerbalistHut, Seed::new(9));
        assert_eq!(site().level(0), site().level(0));
        assert_eq!(site().level(1), site().level(1));
    }
}
