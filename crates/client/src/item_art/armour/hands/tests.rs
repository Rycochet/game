use super::*;
use shared::armour::hands::{HandsPiece, roll};
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = HandsLook> {
    let pieces = [HandsPiece::Gauntlets, HandsPiece::Demi, HandsPiece::Padded];
    pieces.into_iter().flat_map(|piece| {
        (0..300u64).map(move |seed| HandsLook {
            form: roll(&mut Seed::new(seed).rng(), piece, None),
            digits: [5, 4][seed as usize % 2],
            cells: (2, 1),
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
        assert!(!model.parts.is_empty());
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
fn worn_pieces_are_at_the_hands() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks().take(300) {
        let model = build_pair(&look.form, &frame, 5);
        for side in Side::BOTH {
            let hand = Hand::of(&frame, side, 5);
            let wrist = Vec3::new(
                hand.wrist[0] as f32,
                hand.wrist[1] as f32,
                hand.wrist[2] as f32,
            );
            // Everything on this side is within reach of its wrist.
            let near = model
                .parts
                .iter()
                .flat_map(|(_, t)| t.iter().flatten())
                .filter(|corner| corner.x.signum() == side.sign() as f32);
            for corner in near {
                assert!(corner.distance(wrist) < 320.0, "{look:?}");
            }
        }
    }
}
