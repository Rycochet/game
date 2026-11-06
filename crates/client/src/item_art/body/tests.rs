use super::*;
use shared::body::{BEASTS, FIRST_BIPEDS};
use shared::creature::CreatureKind;

#[test]
fn every_body_stays_inside_its_picture_with_room_for_its_edge() {
    let kinds = CreatureKind::ALL
        .into_iter()
        .filter_map(Layout::of)
        .map(|layout| (layout, 1750));
    let coming = FIRST_BIPEDS.map(|biped| {
        let layout = Layout::biped(&biped.proportions.frame(), biped.parts);
        (layout, biped.height)
    });
    let beasts = BEASTS.map(|beast| (beast.layout(), 1750));
    for (layout, height) in kinds.chain(coming).chain(beasts) {
        let (wide, long) = cells(&layout);
        let (wide, long) = (wide as f32, long as f32);
        let per_cell = VIEW_WIDTH / wide.max(long);
        let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
        let (skin, feathers) = fitted(&layout, height);
        for corner in skin.iter().chain(&feathers).flatten() {
            assert!(corner.is_finite());
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{layout:?}");
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{layout:?}");
        }
    }
}
