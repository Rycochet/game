use super::*;
use crate::body::{COW_HEAD, REPTILE_HEAD};
use crate::creature::CreatureKind;

/// The person's own head fills the frame's oval: from its back to its front, from the
/// chin to the crown, as wide as it, within a few millimetres (kept inside it).
#[test]
fn a_persons_head_fills_the_oval() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    let shape = PERSON_HEAD.on_frame(&frame, 0);
    let (across, deep) = frame.head;
    let tall = frame.height - frame.chin;
    assert!((shape.rings[0].along + deep).abs() <= 1, "{shape:?}");
    let nose = shape.rings[7].along;
    assert!((deep - 10..=deep).contains(&nose), "{shape:?}");
    let top = shape.rings.iter().map(|ring| ring.back).max().unwrap();
    assert!((top - tall).abs() <= 5, "{shape:?}");
    let bottom = shape.rings.iter().map(|ring| -ring.belly).min().unwrap();
    assert!(bottom.abs() <= 5, "{shape:?}");
    let widest = shape.rings.iter().map(|ring| ring.half).max().unwrap();
    assert!((widest - across).abs() <= 5, "{shape:?}");
}

/// A beast people's muzzle reaches out in front of a person's face.
#[test]
fn muzzles_reach_past_the_face() {
    let frame = Frame::of(CreatureKind::Human).unwrap();
    for head in [COW_HEAD, REPTILE_HEAD] {
        let shape = head.on_frame(&frame, 120);
        assert!(shape.rings[7].along > frame.head.1 + 30, "{shape:?}");
    }
}
