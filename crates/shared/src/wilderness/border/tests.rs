use super::*;
use crate::poi::{ChunkPos, PoiMap};
use crate::rng::Seed;
use crate::wilderness::{ChunkPlan, Wilderness};
use crate::world::WorldParams;

/// The first cutout near the middle of a world that `wanted` picks.
fn cutout(wanted: impl Fn(&Cutout) -> bool) -> Cutout {
    let params = WorldParams::from_seed(Seed::new(12345));
    let wild = Wilderness::new(&params);
    let pois = PoiMap::new(&params, Seed::new(999));
    (0..30)
        .flat_map(|x| (-15..15).map(move |y| ChunkPos { x, y }))
        .map(|chunk| ChunkPlan::with_new_site(&wild, &pois, chunk))
        .find_map(|plan| plan.cutout().filter(|cutout| wanted(cutout)).cloned())
        .expect("such a cutout")
}

#[test]
fn a_set_in_rectangle_has_a_ring_two_cells_wide() {
    let cutout = cutout(|c| c.poi().kind.is_town());
    let (_, _, width, height) = cutout.level().bounds();
    let border = cutout.border();
    let ring = (width + 2 * BORDER) * (height + 2 * BORDER) - width * height;
    assert_eq!(border.len() as i64, ring);
    assert!(border.iter().all(|&(x, y)| !cutout.contains(x, y)));
}

#[test]
fn a_way_down_alone_has_a_ring_round_its_clearing() {
    let cutout = cutout(|c| c.offset().is_none());
    let border = cutout.border();
    let clearing = 2 * crate::wilderness::ENTRANCE_CLEARING + 1;
    let side = clearing + 2 * BORDER;
    assert_eq!(border.len() as i64, side * side - clearing * clearing);
}
