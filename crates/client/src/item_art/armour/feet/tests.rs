use super::*;
use crate::item_art::armour::legs::Leg;
use shared::armour::feet::roll;
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = FeetLook> {
    let rolled = (0..300u64).map(|seed| roll(&mut Seed::new(seed).rng(), None, Quality::Rare));
    let styled = Style::ALL.into_iter().flat_map(|style| {
        (0..15u64).map(move |seed| roll(&mut Seed::new(seed).rng(), Some(style), Quality::Rare))
    });
    rolled.chain(styled).map(|form| {
        let shape = form.shape();
        FeetLook::new(form, (shape.width, shape.height))
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
fn worn_pieces_are_off_the_foot_and_ankle_and_above_the_ground() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        for side in Side::BOTH {
            let foot = Foot {
                frame: &frame,
                side,
            };
            let leg = Leg {
                frame: &frame,
                side,
            };
            let model = build_foot(&look.form, &frame, side);
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                // On the ground at most, and under the greave's top.
                assert!(corner.y > -0.5 && corner.y < 160.0, "{look:?} {corner}");
                // Never on the other foot's side of the middle.
                assert!(corner.x * side.sign() as f32 > 0.0, "{look:?} {corner}");
                let across = (corner.x - foot.x()).abs();
                let reach = foot.bare_reach(corner.z, corner.y);
                // Straps and soles lie on the ground, which the bare foot touches too.
                let under = corner.y < 4.0;
                assert!(
                    under || reach < 0.0 || across > reach + 1.0,
                    "{look:?} {corner}"
                );
                if corner.y > frame.ankle as f32 {
                    let away = Vec2::new(corner.x - foot.x(), corner.z).length();
                    assert!(away > leg.skin(corner.y) + 1.0, "{look:?} {corner}");
                }
            }
        }
    }
}
