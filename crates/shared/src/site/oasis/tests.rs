use super::*;
use crate::poi::PoiKind;
use crate::site::{Generator, Site, leads_out};

fn options() -> OasisOptions {
    match Generator::for_poi(PoiKind::Oasis) {
        Generator::Oasis(options) => options,
        other => panic!("{other:?}"),
    }
}

#[test]
fn every_oasis_has_its_pools_grass_and_trees_and_can_be_walked_round() {
    for n in 0..40 {
        let seed = Seed::new(n * 31 + 7);
        let site = Site::new(PoiKind::Oasis, seed);
        assert_eq!(site.levels(), 1);
        let ground = site.level(0);
        let level = &ground.level;
        let count = |kind: Cell| {
            let (_, _, width, height) = level.bounds();
            (0..height)
                .flat_map(|y| (0..width).map(move |x| (x, y)))
                .filter(|&(x, y)| level.cell(x, y) == kind)
                .count()
        };
        assert!(count(Cell::Water) > 50, "seed {n}: little water");
        assert!(count(Cell::Grass) > 20 && count(Cell::Tree) > 3, "seed {n}");
        let entrance = ground.entrance.unwrap();
        assert!(leads_out(level, entrance.0, entrance.1));
        assert!(
            super::super::grid::all_reached(level, &[entrance]),
            "seed {n}"
        );
        let pools = Plan::pools(&options(), seed);
        assert!((1..=2).contains(&pools));
    }
}
