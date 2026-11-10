use super::*;
use shared::rng::Seed;
use shared::underclothes::{UnderPiece, roll};

fn looks() -> impl Iterator<Item = UnderLook> {
    UnderPiece::ALL.into_iter().flat_map(|piece| {
        (0..80u64).map(move |seed| {
            let form = roll(&mut Seed::new(seed).rng(), piece, Quality::Rare);
            let shape = form.shape();
            UnderLook::new(form, (shape.width, shape.height))
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
fn every_piece_builds_on_every_people() {
    for (kind, frame) in CreatureKind::ALL
        .iter()
        .filter_map(|&kind| Some((kind, Frame::of(kind)?)))
    {
        for look in looks().step_by(7) {
            let model = build(&look.form, &frame, Fit::Tucked);
            let corners = model.parts.iter().flat_map(|(_, t)| t.iter().flatten());
            assert!(corners.clone().count() > 0, "{kind:?} {look:?}");
            assert!(
                corners.clone().all(|corner| corner.is_finite()),
                "{kind:?} {look:?}"
            );
        }
    }
}

#[test]
fn worn_it_is_never_looser_than_alone() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for look in looks() {
        let width = |fit| {
            let (low, high) = build(&look.form, &frame, fit).bounds();
            (high - low).x.max((high - low).z)
        };
        assert!(width(Fit::Tucked) <= width(Fit::Alone) + 0.01, "{look:?}");
    }
}
