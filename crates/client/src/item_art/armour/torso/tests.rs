use super::*;
use crate::item_art::body::trunk_half;
use shared::armour::torso::{TorsoKind, roll};
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = TorsoLook> {
    [TorsoKind::Armour, TorsoKind::ArmingVest]
        .into_iter()
        .flat_map(|kind| {
            (0..150u64).map(move |seed| {
                let form = roll(&mut Seed::new(seed).rng(), kind, None, Quality::Rare);
                let shape = form.shape();
                TorsoLook::new(form, (shape.width, shape.height))
            })
        })
}

#[test]
fn every_piece_stays_inside_its_picture_with_room_for_its_edge() {
    for look in looks() {
        let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
        let per_cell = VIEW_WIDTH / wide.max(long);
        let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
        let model = fitted(&look);
        assert!(!model.parts.is_empty(), "{look:?}");
        for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
            assert!(corner.is_finite(), "{look:?}");
            assert!(
                corner.x.abs() + edge <= wide * per_cell / 2.0 + 0.001,
                "{look:?}"
            );
            assert!(
                corner.y.abs() + edge <= long * per_cell / 2.0 + 0.001,
                "{look:?}"
            );
        }
    }
}

#[test]
fn worn_pieces_are_on_the_torso_and_off_its_skin() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        let model = build(&look.form, &frame);
        let (low, high) = model.bounds();
        // From just below the waist to the top of the shoulders.
        assert!(low.y > frame.hip as f32 - 40.0, "{look:?} {low}");
        assert!(high.y < frame.neck_base as f32 + 60.0, "{look:?} {high}");
        for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
            // Away from the arms, where sleeves are.
            let level = frame.waist as f32..frame.chest as f32;
            let (across, deep) = trunk_half(&frame, corner.y);
            if level.contains(&corner.y) && corner.x.abs() < across * 0.7 {
                let inside = (corner.x / across).powi(2) + (corner.z / deep).powi(2);
                assert!(inside > 1.0, "{look:?} {corner}");
            }
        }
    }
}
