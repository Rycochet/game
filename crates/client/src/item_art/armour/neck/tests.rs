use super::*;
use crate::item_art::body::trunk_half;
use shared::armour::Style;
use shared::armour::neck::roll;
use shared::body::{Joint, Side};
use shared::rng::Seed;

fn looks() -> impl Iterator<Item = NeckLook> {
    NeckKind::ALL.into_iter().flat_map(|kind| {
        let rolled =
            (0..80u64).map(move |seed| roll(&mut Seed::new(seed).rng(), kind, None, Quality::Rare));
        let styled = Style::ALL.into_iter().flat_map(move |style| {
            (0..6u64)
                .map(move |seed| roll(&mut Seed::new(seed).rng(), kind, Some(style), Quality::Rare))
        });
        rolled.chain(styled).map(|form| {
            let shape = form.shape();
            NeckLook::new(form, (shape.width, shape.height))
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

/// How far inside the body a point is, in millimetres: above 0 means inside.
fn inside(frame: &Frame, point: Vec3) -> f32 {
    let f = |value: i32| value as f32;
    let mut deepest = f32::MIN;
    let across = Vec2::new(point.x, point.z).length();
    if point.y > f(frame.neck_base) - 20.0 && point.y < f(frame.chin) + 20.0 {
        deepest = deepest.max(f(frame.neck) - across);
    }
    let half_high = f(frame.height - frame.chin) / 2.0;
    let centre = Vec3::new(0.0, f(frame.chin) + half_high, 0.0);
    let scaled = (point - centre) / Vec3::new(f(frame.head.0), half_high, f(frame.head.1));
    if scaled.length() < 1.0 {
        deepest = deepest.max((1.0 - scaled.length()) * f(frame.head.0));
    }
    if point.y <= f(frame.neck_base) && point.y > f(frame.waist) {
        let (a, d) = trunk_half(frame, point.y);
        let ellipse = Vec2::new(point.x / a, point.z / d).length();
        if ellipse < 1.0 {
            deepest = deepest.max((1.0 - ellipse) * d);
        }
    }
    for side in Side::BOTH {
        let joint = |joint: Joint| {
            let [x, y, z] = frame.joint(joint);
            Vec3::new(x as f32, y as f32, z as f32)
        };
        let (from, to) = (joint(Joint::Shoulder(side)), joint(Joint::Elbow(side)));
        let along = ((point - from).dot(to - from) / (to - from).length_squared()).clamp(0.0, 1.0);
        let away = (point - from.lerp(to, along)).length();
        deepest = deepest.max(f(frame.upper_arm) - away);
    }
    deepest
}

#[test]
fn worn_pieces_stay_off_the_body() {
    for kind in [
        CreatureKind::Human,
        CreatureKind::Giant,
        CreatureKind::Kobold,
    ] {
        let frame = Frame::of(kind).unwrap();
        for look in looks() {
            let model = build(&look.form, &frame);
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                let depth = inside(&frame, *corner);
                assert!(depth < 1.0, "{kind:?} {:?} {corner} {depth}", look.form);
            }
        }
    }
}
