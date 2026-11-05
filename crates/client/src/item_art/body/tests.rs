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
        .filter_map(Frame::of)
        .map(|frame| (frame, shared::body::NONE, 1750));
    let coming = FIRST_BIPEDS.map(|biped| (biped.proportions.frame(), biped.parts, biped.height));
    for (frame, parts, height) in kinds.chain(coming) {
        let (skin, feathers) = fitted(&frame, &parts, height);
        for corner in skin.iter().chain(&feathers).flatten() {
            assert!(corner.is_finite());
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{frame:?}");
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{frame:?}");
        }
    }
}
