use super::*;
use shared::body::FIRST_BIPEDS;
use shared::creature::CreatureKind;

#[test]
fn every_body_stays_inside_its_picture_with_room_for_its_edge() {
    let (wide, long) = (CELLS.0 as f32, CELLS.1 as f32);
    let per_cell = VIEW_WIDTH / wide.max(long);
    let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
    let kinds = CreatureKind::ALL.into_iter().filter_map(Frame::of);
    let coming = FIRST_BIPEDS.map(|biped| biped.proportions.frame());
    for frame in kinds.chain(coming) {
        for corner in fitted(&frame).iter().flatten() {
            assert!(corner.is_finite());
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{frame:?}");
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{frame:?}");
        }
    }
}
