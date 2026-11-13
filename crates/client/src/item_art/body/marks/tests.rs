use std::collections::BTreeMap;

use shared::body::Layout;
use shared::creature::CreatureKind;
use shared::sheet::{Appearance, Trait};

use super::super::build;
use super::*;

fn marks(kind: CreatureKind, choice: u8) -> Marks {
    Appearance(BTreeMap::from([(Trait::Markings, choice)]))
        .marks(kind)
        .unwrap()
}

/// The middles of the marked triangles.
fn marked(kind: CreatureKind, choice: u8) -> (Layout, Vec<Vec3>) {
    let layout = Layout::at_size(kind).unwrap();
    let (_, marked) = split(&layout, build(&layout), Some(marks(kind, choice)));
    let middles = marked
        .iter()
        .map(|[a, b, c]| (*a + *b + *c) / 3.0)
        .collect();
    (layout, middles)
}

#[test]
fn a_wolf_is_pale_underneath_and_on_its_face_not_on_its_back() {
    let (layout, middles) = marked(CreatureKind::Wolf, 0);
    assert!(!middles.is_empty());
    let bones = Bones::of(&layout);
    let on = |part: u32| -> Vec<Vec3> {
        middles
            .iter()
            .copied()
            .filter(|at| bones.part_at(*at) == part)
            .collect()
    };
    // The trunk's markings are all on its lower half, none on its back.
    assert!(on(BodyPart::TRUNK).len() > 10);
    assert!(
        on(BodyPart::TRUNK)
            .iter()
            .all(|at| trunk::underside(&layout, *at) > 0.0)
    );
    assert!(on(BodyPart::HEAD).len() > 5);
}

#[test]
fn no_markings_leave_the_skin_as_it_was() {
    let layout = Layout::at_size(CreatureKind::Wolf).unwrap();
    let skin = build(&layout);
    let (kept, marked) = split(&layout, skin.clone(), None);
    assert_eq!(kept, skin);
    assert!(marked.is_empty());
}

#[test]
fn cattle_patches_differ_by_their_layout() {
    let (_, first) = marked(CreatureKind::Cattle, 2);
    let (_, second) = marked(CreatureKind::Cattle, 3);
    assert!(first.len() > 20 && second.len() > 20);
    assert_ne!(first, second);
}

#[test]
fn every_marking_marks_something_and_leaves_some_coat() {
    for kind in CreatureKind::ALL {
        for choice in 0..Trait::Markings.choices_for(kind).len() as u8 {
            let Some(marks) = Appearance(BTreeMap::from([(Trait::Markings, choice)])).marks(kind)
            else {
                continue;
            };
            let layout = Layout::at_size(kind).unwrap();
            let (kept, marked) = split(&layout, build(&layout), Some(marks));
            assert!(!marked.is_empty(), "{kind:?} {choice}");
            assert!(kept.len() > marked.len(), "{kind:?} {choice}");
        }
    }
}
