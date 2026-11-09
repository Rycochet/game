use crate::movement::{WildHeights, WorldGround};
use crate::poi::{ChunkPos, PoiMap};
use crate::rng::Seed;
use crate::wilderness::{ChunkPlan, Cutout, Wilderness};
use crate::world::WorldParams;

/// A town's cutout near the middle of a world whose land under it rises by
/// at least `rise` 16ths, and the world's ground.
fn town_on_a_slope(rise: i32) -> (Cutout, WorldGround) {
    let params = WorldParams::from_seed(Seed::new(12345));
    let wild = Wilderness::new(&params);
    let pois = PoiMap::new(&params, Seed::new(999));
    let ground = WorldGround::new(&params);
    let cutout = (0..60)
        .flat_map(|x| (-30..30).map(move |y| ChunkPos { x, y }))
        .map(|chunk| ChunkPlan::with_new_site(&wild, &pois, chunk))
        .filter_map(|plan| plan.cutout().filter(|c| c.poi().kind.is_town()).cloned())
        .find(|cutout| {
            let land = WildHeights::new(&ground, cutout.level().bounds());
            let (min_x, min_y, width, height) = cutout.level().bounds();
            let all = (min_y..min_y + height)
                .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
                .map(|(x, y)| land.at(x, y));
            let (low, high) = all.fold((i32::MAX, i32::MIN), |(l, h), v| (l.min(v), h.max(v)));
            high - low >= rise
        })
        .expect("a town on a slope");
    (cutout, ground)
}

#[test]
fn covered_cells_are_level_and_open_ones_follow_the_land() {
    let (cutout, ground) = town_on_a_slope(3);
    let flats = cutout.flats(&ground);
    let bounds = cutout.level().bounds();
    let land = WildHeights::new(&ground, bounds);
    let heights = land.clone().with_flats(&flats, cutout.level());
    let (min_x, min_y, width, height) = bounds;
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            if !cutout.contains(x, y) {
                continue;
            }
            if cutout.level().cell(x, y).is_outside() {
                assert_eq!(heights.at(x, y), land.at(x, y), "({x}, {y}) open");
            } else {
                let flat = flats.iter().find(|flat| flat.contains(x, y));
                let flat = flat.expect("every covered cell is in a flat");
                assert_eq!(heights.at(x, y), flat.height, "({x}, {y}) covered");
            }
        }
    }
}

#[test]
fn buildings_on_a_slope_stand_at_heights_of_their_own() {
    let (cutout, ground) = town_on_a_slope(3);
    let flats = cutout.flats(&ground);
    assert!(flats.len() > 1, "a town has buildings");
    let first = flats[0].height;
    assert!(flats.iter().any(|flat| flat.height != first), "{flats:?}");
    // Each is near the land round it: within its range just outside.
    let land = WildHeights::new(&ground, {
        let (x, y, w, h) = cutout.level().bounds();
        (x - 2, y - 2, w + 4, h + 4)
    });
    for flat in &flats {
        let (min_x, min_y, width, height) = flat.bounds;
        let ring = (min_y - 1..=min_y + height)
            .flat_map(|y| (min_x - 1..=min_x + width).map(move |x| (x, y)))
            .map(|(x, y)| land.at(x, y));
        let (low, high) = ring.fold((i32::MAX, i32::MIN), |(l, h), v| (l.min(v), h.max(v)));
        assert!(
            (low..=high).contains(&flat.height),
            "{flat:?} {low}..{high}"
        );
    }
}

#[test]
fn no_two_flats_overlap() {
    let (cutout, ground) = town_on_a_slope(1);
    let flats = cutout.flats(&ground);
    for (i, a) in flats.iter().enumerate() {
        for b in &flats[i + 1..] {
            let (ax, ay, aw, ah) = a.bounds;
            let (bx, by, bw, bh) = b.bounds;
            let apart = ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay;
            assert!(apart, "{a:?} {b:?}");
        }
    }
}
