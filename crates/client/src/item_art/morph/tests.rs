use super::super::VIEW_WIDTH;
use super::super::body::morph_placed;
use super::*;

/// Every kind with a body drawn.
fn kinds() -> Vec<CreatureKind> {
    CreatureKind::ALL
        .into_iter()
        .filter(|&kind| Layout::of(kind).is_some())
        .collect()
}

#[test]
fn every_step_between_any_two_bodies_stays_in_its_picture() {
    let plain = Appearance::default();
    let kinds = kinds();
    for &from in &kinds {
        for &to in &kinds {
            for step in 1..MORPH_STEPS {
                let Some(Look::Morph(look)) = morph_look((from, &plain), (to, &plain), step) else {
                    panic!("{from:?} into {to:?}");
                };
                let placed = morph_placed(&look);
                assert!(!placed.is_empty(), "{from:?} into {to:?} at {step}");
                for corner in placed.iter().flat_map(|(t, _)| t.iter().flatten()) {
                    assert!(corner.is_finite(), "{from:?} into {to:?} at {step}");
                    // Square, two cells: half the view each way, less the edge's room.
                    let reach = corner.x.abs().max(corner.y.abs());
                    assert!(reach <= VIEW_WIDTH / 2.0, "{from:?} into {to:?} at {step}");
                }
            }
        }
    }
}

#[test]
fn steps_run_from_one_body_to_the_other() {
    let plain = Appearance::default();
    let along = |step| match morph_look(
        (CreatureKind::Human, &plain),
        (CreatureKind::Horse, &plain),
        step,
    ) {
        Some(Look::Morph(look)) => look.along(),
        _ => panic!("a person and a horse have bodies"),
    };
    assert_eq!(along(0), 0);
    assert_eq!(along(MORPH_STEPS), BLEND_WHOLE);
    assert!((1..MORPH_STEPS).all(|step| along(step) > along(step - 1)));
}

#[test]
fn colours_blend_from_one_to_the_other() {
    let a = Colours::PLAIN;
    let b = Colours {
        skin: [0, 0, 0],
        ..Colours::PLAIN
    };
    assert_eq!(blend_colours(&a, &b, 0), a);
    assert_eq!(blend_colours(&a, &b, BLEND_WHOLE), b);
    // 199 halfway to 0.
    assert_eq!(blend_colours(&a, &b, BLEND_WHOLE / 2).skin[0], 100);
}
