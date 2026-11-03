use super::*;
use shared::armour::head::roll;
use shared::rng::Seed;

const KINDS: [HeadKind; 4] = [
    HeadKind::Cap,
    HeadKind::Coif,
    HeadKind::Helmet,
    HeadKind::BevoredHelm,
];

fn looks() -> impl Iterator<Item = HeadLook> {
    KINDS.into_iter().flat_map(|kind| {
        (0..150u64).map(move |seed| {
            let form = roll(&mut Seed::new(seed).rng(), kind, None, Quality::Rare);
            let shape = form.shape();
            HeadLook::new(form, (shape.width, shape.height))
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
            assert!(corner.x.abs() + edge <= wide * per_cell / 2.0 + 0.001);
            assert!(corner.y.abs() + edge <= long * per_cell / 2.0 + 0.001);
        }
    }
}

#[test]
fn worn_pieces_are_on_the_head_and_off_its_skin() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    let centre = Vec3::new(0.0, (frame.height + frame.chin) as f32 / 2.0, 0.0);
    let half = Vec3::new(
        frame.head.0 as f32,
        (frame.height - frame.chin) as f32 / 2.0,
        frame.head.1 as f32,
    );
    for look in looks() {
        let model = build(&look.form, &frame);
        let (low, high) = model.bounds();
        // Nothing lower than a coif's skirt on the shoulders.
        assert!(low.y > frame.chest as f32, "{look:?} {low}");
        assert!(high.y > frame.height as f32, "{look:?} {high}");
        for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
            assert!(corner.is_finite(), "{look:?}");
            // Above the chin, nothing cuts into the head.
            let inside = ((*corner - centre) / half).length();
            if corner.y > frame.chin as f32 + 20.0 {
                assert!(inside > 0.98, "{look:?} {corner}");
            }
        }
    }
}
