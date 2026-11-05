use super::*;
use shared::armour::Style;
use shared::armour::arms::roll;
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = ArmsLook> {
    let rolled = (0..200u64).map(|seed| roll(&mut Seed::new(seed).rng(), None, Quality::Rare));
    let styled = Style::ALL.into_iter().flat_map(|style| {
        (0..12u64).map(move |seed| roll(&mut Seed::new(seed).rng(), Some(style), Quality::Rare))
    });
    rolled.chain(styled).map(|form| {
        let shape = form.shape();
        ArmsLook::new(form, (shape.width, shape.height))
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
fn worn_pieces_are_on_the_arm_off_its_skin_and_above_the_wrist() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        for &side in look.form.sides.sides() {
            let arm = Arm {
                frame: &frame,
                side,
                bend: 0.0,
            };
            let model = build_arm(&look.form, &frame, side);
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                // Down the arm, as `Arm` measures it, to the nearest point of its middle.
                let (down, away) = (0..=200)
                    .map(|step| step as f32 * 5.0)
                    .map(|down| (down, (*corner - arm.centre(down)).length()))
                    .min_by(|a, b| a.1.total_cmp(&b.1))
                    .unwrap();
                assert!(down > 0.0 && down < arm.elbow() + 255.0, "{look:?} {down}");
                assert!(away > arm.skin(down) + 1.0, "{look:?} {down} {away}");
            }
        }
    }
}
