use super::*;
use crate::item_art::body::trunk_depth;
use crate::item_art::{OUTLINE_PIXELS, ShouldersLook, VIEW_WIDTH};
use shared::armour::shoulders::{ShouldersKind, roll};
use shared::body::Frame;
use shared::creature::CreatureKind;
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = ShouldersLook> {
    ShouldersKind::ALL.into_iter().flat_map(|kind| {
        (0..60u64).map(move |seed| {
            let form = roll(&mut Seed::new(seed).rng(), Some(kind), None);
            let shape = form.shape();
            ShouldersLook::new(form, (shape.width, shape.height))
        })
    })
}

#[test]
fn every_pair_stays_inside_its_picture_with_room_for_its_edge() {
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
fn worn_pieces_are_at_the_shoulders() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        let model = build_pair(&look.form, &frame);
        let (low, high) = model.bounds();
        assert!(
            high.y < frame.height as f32 && low.y > frame.waist as f32 - 10.0,
            "{look:?}"
        );
        assert!(high.x < 420.0 && low.x > -420.0, "{look:?}");
    }
}

#[test]
fn rigid_pieces_stay_off_the_trunk() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    let kinds = [ShouldersKind::Pauldrons, ShouldersKind::Spaulders];
    for look in looks().filter(|look| kinds.contains(&look.form.kind())) {
        let model = build_pair(&look.form, &frame);
        for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
            // Below the shoulder and well inside its width, nothing is in the trunk.
            if corner.y < frame.shoulder as f32 - 20.0 && corner.x.abs() < 120.0 {
                let deep = trunk_depth(&frame, corner.x, corner.y);
                assert!(corner.z.abs() > deep + 10.0, "{look:?} {corner}");
            }
        }
    }
}
