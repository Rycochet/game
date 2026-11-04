use super::*;
use shared::armour::Style;
use shared::armour::legs::{LegsKind, roll};
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = LegsLook> {
    let kinds = [LegsKind::Armour, LegsKind::Chausses, LegsKind::Padded];
    let rolled = kinds.into_iter().flat_map(|kind| {
        (0..150u64).map(move |seed| roll(&mut Seed::new(seed).rng(), kind, None, Quality::Rare))
    });
    let styled = Style::ALL.into_iter().flat_map(|style| {
        (0..15u64).map(move |seed| {
            roll(
                &mut Seed::new(seed).rng(),
                LegsKind::Armour,
                Some(style),
                Quality::Rare,
            )
        })
    });
    rolled.chain(styled).map(|form| {
        let shape = form.shape();
        LegsLook::new(form, (shape.width, shape.height))
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
fn worn_pieces_are_on_the_leg_off_its_skin_and_above_the_ground() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        for &side in look.form.sides.sides() {
            let leg = Leg {
                frame: &frame,
                side,
            };
            let axis = side.sign() as f32 * frame.hip_apart as f32;
            let model = build_leg(&look.form, &frame, side);
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                // Nothing above the hip but a haidate's top, nothing under the ground.
                assert!(
                    corner.y > -1.0 && corner.y < frame.hip as f32 + 45.0,
                    "{look:?}"
                );
                // The mail foot goes round the foot, not the leg.
                let foot = if look.form.footed() { 10.0 } else { -20.0 };
                if corner.y < frame.ankle as f32 + foot {
                    continue;
                }
                let away = Vec2::new(corner.x - axis, corner.z).length();
                assert!(away > leg.skin(corner.y) + 1.0, "{look:?} {corner}");
            }
        }
    }
}
