use super::*;

#[test]
fn every_body_stays_inside_its_picture_with_room_for_its_edge() {
    let (wide, long) = (CELLS.0 as f32, CELLS.1 as f32);
    let per_cell = VIEW_WIDTH / wide.max(long);
    let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
    for frame in CreatureKind::ALL.into_iter().filter_map(Frame::of) {
        for corner in fitted(&frame).iter().flatten() {
            assert!(corner.is_finite());
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{frame:?}");
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{frame:?}");
        }
    }
}
