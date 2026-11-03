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
            cells_of(&below.level).all(|(x, y)| !below.level.is_walkable(x, y) || reach.is(x, y))
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
