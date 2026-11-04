use super::*;
use crate::item_art::armour::legs::Leg;
use shared::footwear::{FootwearPiece, roll};
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = FootwearLook> {
    FootwearPiece::ALL.into_iter().flat_map(|piece| {
        (0..120u64).map(move |seed| {
            let form = roll(&mut Seed::new(seed).rng(), piece, Quality::Rare);
            let shape = form.shape();
            FootwearLook::new(form, (shape.width, shape.height))
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
fn worn_footwear_stays_on_its_own_side_and_off_the_leg() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        for side in Side::BOTH {
            let leg = Leg {
                frame: &frame,
                side,
            };
            let foot = Foot {
                frame: &frame,
                side,
            };
            let model = build_foot(&look.form, &frame, side);
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                // The sole and any heel or teeth are under the ground, by at most this.
                assert!(corner.y > -130.0, "{look:?} {corner}");
                assert!(corner.x * side.sign() as f32 > 0.0, "{look:?} {corner}");
                if corner.y > frame.ankle as f32 + 10.0 {
                    let away = Vec2::new(corner.x - foot.x(), corner.z).length();
                    assert!(away > leg.skin(corner.y), "{look:?} {corner}");
                }
            }
        }
    }
}

#[test]
fn boots_reach_as_high_as_their_shaft() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks().filter(|look| look.form.boot() && !look.form.open()) {
        let model = build_foot(&look.form, &frame, Side::Left);
        let (_, high) = model.bounds();
        let shaft = frame.foot_length as f32 * look.form.shaft as f32 / 100.0;
        assert!(high.y > shaft * 0.8, "{look:?} {high}");
    }
}
