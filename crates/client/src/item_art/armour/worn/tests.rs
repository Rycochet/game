use super::*;
use crate::item_art::body::{Girth, trunk_half};
use crate::item_art::samples;
use bevy::prelude::Vec3;
use shared::creature::CreatureKind;

/// How many sample items of every kind are tried: enough for each place armour goes,
/// footwear and underclothes to come up many times over.
const SAMPLES: usize = 1500;

/// How far into the skin a point may be, in millimetres: a quilted surface's dips, and
/// the edge of something flat on a chest that leans back (a giant's brigandine).
const TOLERANCE: f32 = 2.0;

/// A frame's trunk, looked up by height a millimetre at a time.
struct Trunk {
    low: f32,
    girths: Vec<Girth>,
}

impl Trunk {
    /// The trunk's front and back, where everything worn lies over them: from a little
    /// above the hip joints (below, what goes round each leg starts inside the trunk, as
    /// the thighs do, and tassets hang from under the fauld) to a little below the
    /// shoulders (above, sleeves start inside them, and neck pieces tuck in).
    fn of(frame: &Frame) -> Self {
        let (low, high) = (frame.hip + 60, frame.shoulder - 60);
        let girths = (low..=high).map(|y| trunk_half(frame, y as f32)).collect();
        Trunk {
            low: low as f32,
            girths,
        }
    }

    /// How far inside the trunk's skin a point is, in millimetres along the way to the
    /// spine: above 0 means inside. Only over the middle of the front and the back.
    fn inside(&self, point: Vec3) -> f32 {
        let at = (point.y - self.low).round();
        let Some(girth) = (at >= 0.0).then(|| self.girths.get(at as usize)).flatten() else {
            return f32::MIN;
        };
        if point.x.abs() >= girth.across * 0.5 {
            return f32::MIN;
        }
        let deep = girth.deep(point.z >= 0.0);
        let r = (point.x / girth.across).hypot(point.z / deep);
        (1.0 - r) * deep.min(girth.across)
    }
}

/// The look's sort and its form's first words, to tell pieces apart in a report.
fn name(look: &Look) -> String {
    let text = format!("{look:?}");
    text.chars().take(150).collect()
}

#[test]
fn everything_worn_stays_outside_the_trunk_on_every_people() {
    let looks = samples::looks(0, SAMPLES);
    let frames: Vec<(CreatureKind, Frame, Trunk)> = CreatureKind::ALL
        .into_iter()
        .filter_map(|kind| Frame::of(kind).map(|frame| (kind, frame, Trunk::of(&frame))))
        .collect();
    assert!(frames.len() > 5);
    let mut worn_looks = 0;
    let mut wrong = Vec::new();
    for look in &looks {
        for (kind, frame, trunk) in &frames {
            let parts = worn(look, frame, None);
            if parts.is_empty() {
                continue;
            }
            worn_looks += 1;
            let deepest = parts
                .iter()
                .flat_map(|(triangles, _)| triangles.iter().flatten())
                .map(|&point| (trunk.inside(point), point))
                .max_by(|a, b| a.0.total_cmp(&b.0))
                .expect("a worn piece has points");
            if deepest.0 > TOLERANCE {
                wrong.push(format!(
                    "{kind:?} {:.1} mm at {}: {}",
                    deepest.0,
                    deepest.1,
                    name(look)
                ));
            }
        }
    }
    assert!(worn_looks > 500, "{worn_looks}");
    let count = wrong.len();
    wrong.truncate(60);
    assert!(
        wrong.is_empty(),
        "{count} inside the trunk:\n{}",
        wrong.join("\n")
    );
}
