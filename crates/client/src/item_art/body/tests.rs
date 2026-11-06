use super::*;
use shared::body::FIRST_BIPEDS;
use shared::creature::CreatureKind;

#[test]
fn every_body_stays_inside_its_picture_with_room_for_its_edge() {
    let (wide, long) = (CELLS.0 as f32, CELLS.1 as f32);
    let per_cell = VIEW_WIDTH / wide.max(long);
    let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
    let kinds = CreatureKind::ALL
        .into_iter()
        .filter_map(Layout::of)
        .map(|layout| (layout, 1750));
    let coming = FIRST_BIPEDS.map(|biped| {
        let layout = Layout::biped(&biped.proportions.frame(), biped.parts);
        (layout, biped.height)
    });
    for (layout, height) in kinds.chain(coming) {
        let (skin, feathers) = fitted(&layout, height);
        for corner in skin.iter().chain(&feathers).flatten() {
            assert!(corner.is_finite());
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{layout:?}");
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{layout:?}");
        }
    }
}
