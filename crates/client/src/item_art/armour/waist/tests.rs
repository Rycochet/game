use shared::armour::Style;
use shared::armour::waist::{self, WaistKind, WaistPiece};
use shared::item::Quality;
use shared::rng::Seed;

use super::*;

fn forms() -> impl Iterator<Item = WaistForm> {
    let kinds = [WaistKind::Armour, WaistKind::Cuirassier];
    kinds.into_iter().flat_map(|kind| {
        (0..400u64)
            .map(move |seed| waist::roll(&mut Seed::new(seed).rng(), kind, None, Quality::Common))
    })
}

#[test]
fn every_piece_builds_on_the_body_below_the_torso() {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    for form in forms() {
        let model = build(&form, &frame);
        assert!(!model.parts.is_empty(), "{form:?}");
        let (low, high) = model.bounds();
        assert!(low.is_finite() && high.is_finite(), "{form:?}");
        // From just under the torso piece's hem down to above the knee, or round it.
        assert!(high.y <= (frame.waist + 45) as f32, "{form:?} {high}");
        assert!(low.y >= (frame.knee - 100) as f32, "{form:?} {low}");
        assert!(high.x.abs().max(low.x.abs()) < 330.0, "{form:?}");
    }
}

#[test]
fn every_shape_and_style_draws() {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    for style in Style::ALL {
        for seed in 0..60u64 {
            let mut rng = Seed::new(seed).rng();
            let form = waist::roll(&mut rng, WaistKind::Armour, Some(style), Quality::Rare);
            let model = build(&form, &frame);
            let triangles: usize = model.parts.iter().map(|(_, t)| t.len()).sum();
            assert!(triangles > 20, "{form:?}");
        }
    }
    let pieces: Vec<WaistPiece> = forms().map(|form| form.piece).collect();
    for piece in WaistPiece::ALL {
        assert!(pieces.contains(&piece), "{piece:?}");
    }
}

#[test]
fn every_picture_fits_its_cells() {
    for form in forms().step_by(7) {
        let shape = form.shape();
        let look = WaistLook::new(form, (shape.width, shape.height));
        let (low, high) = fitted(&look).bounds();
        let (wide, long) = (shape.width as f32, shape.height as f32);
        let per_cell = VIEW_WIDTH / wide.max(long);
        assert!(high.x - low.x <= wide * per_cell + 0.01, "{form:?}");
        assert!(high.y - low.y <= long * per_cell + 0.01, "{form:?}");
    }
}
