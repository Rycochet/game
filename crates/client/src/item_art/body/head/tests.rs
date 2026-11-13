use shared::body::Layout;
use shared::creature::CreatureKind;

use super::*;

/// How far out of the oval a skull may stand, in millimetres: inside the 10 mm a
/// helmet keeps off the head (where a muzzle's top meets the brow, the most).
const LEEWAY: f32 = 8.0;

/// Every point round a measured head's rings, as drawn.
fn skin(layout: &Layout, shape: &HeadShape) -> Vec<Vec3> {
    let axes = HeadAxes::of(layout, shape);
    shape
        .rings
        .iter()
        .flat_map(|ring| {
            let middle = axes.middle(ring);
            let (across, deep) = (ring.half as f32, (ring.belly + ring.back) as f32 / 2.0);
            oval(middle, Vec3::X * across, axes.up * deep)
        })
        .collect()
}

/// A beast people's skull stays inside its frame's oval head, which every helmet and
/// coif is built round, so they still fit; only the muzzle reaches out of it, under the
/// brow.
#[test]
fn beast_peoples_skulls_stay_inside_the_oval() {
    for kind in [
        CreatureKind::Kobold,
        CreatureKind::Lizardfolk,
        CreatureKind::Minotaur,
    ] {
        let layout = Layout::of(kind).unwrap();
        let shape = layout.head.shape.expect("a measured head");
        let frame = layout.frame().unwrap();
        let (across, deep) = (frame.head.0 as f32, frame.head.1 as f32);
        let half_up = (frame.height - frame.chin) as f32 / 2.0;
        let centre = Vec3::new(0.0, frame.chin as f32 + half_up, 0.0);
        let brow = frame.chin as f32 + half_up * 2.0 * 0.6;
        for point in skin(&layout, &shape) {
            // The muzzle stays under the brow, where a helmet's face opens and its brim is.
            if point.z > deep * 0.3 {
                assert!(point.y <= brow + 1.0, "{kind:?}: {point} is above the brow");
                continue;
            }
            let off = point - centre;
            let reach =
                ((off.x / across).powi(2) + (off.y / half_up).powi(2) + (off.z / deep).powi(2))
                    .sqrt();
            // As far outside as the leeway, in millimetres at the widest.
            let outside = (reach - 1.0) * deep;
            assert!(
                outside <= LEEWAY,
                "{kind:?}: {point} is {outside} mm outside the oval"
            );
        }
    }
}
