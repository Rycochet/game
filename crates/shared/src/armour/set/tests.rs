use super::*;
use crate::creature::Size;
use crate::item::Quality;
use crate::rng::Seed;

fn rolled(kind: SetKind) -> impl Iterator<Item = SetForm> {
    (0..300u64).map(move |seed| {
        let mut rng = Seed::new(seed).derive("set").rng();
        roll(&mut rng, kind, None, Quality::Uncommon)
    })
}

/// The locations a set has pieces for.
fn covered(set: &SetForm) -> Vec<Location> {
    use Location::{Arms, Feet, Hands, Head, Legs, Neck, Shoulders, Torso, Waist};
    let mut have = vec![Torso];
    let pieces = [
        (Head, set.head.is_some()),
        (Neck, set.neck.is_some()),
        (Shoulders, set.shoulders.is_some()),
        (Arms, set.arms.is_some()),
        (Waist, set.waist.is_some()),
        (Hands, set.hands.is_some()),
        (Legs, set.legs.is_some()),
        (Feet, set.feet.is_some()),
    ];
    have.extend(pieces.into_iter().filter(|(_, has)| *has).map(|(at, _)| at));
    have
}

#[test]
fn every_set_has_a_sensible_piece_for_each_location_it_covers() {
    for kind in SetKind::ALL {
        for set in rolled(kind) {
            assert!(set.allowed(), "{set:?}");
            let mut want = kind.locations().to_vec();
            let mut have = covered(&set);
            want.sort();
            have.sort();
            assert_eq!(have, want, "{kind:?}");
            assert!(set.torso.back, "{set:?}");
            assert_ne!(set.construction(), Construction::Padded);
            assert!(!set.name().is_empty());
            assert!(set.grams_at(Size::Medium) > set.torso.grams_at(Size::Medium));
        }
    }
}

#[test]
fn every_piece_shares_the_style_and_nearly_all_the_material() {
    let (mut pieces, mut matching) = (0, 0);
    for kind in SetKind::ALL {
        for set in rolled(kind) {
            let style = set.style();
            let materials = [
                set.head.map(|p| (p.style, p.material)),
                set.neck.map(|p| (p.style, p.material)),
                set.shoulders.map(|p| (p.style, p.material)),
                set.arms.map(|p| (p.style, p.material)),
                set.waist.map(|p| (p.style, p.material)),
                set.hands.map(|p| (p.style, p.material)),
                set.legs.map(|p| (p.style, p.material)),
                set.feet.map(|p| (p.style, p.material)),
            ];
            for (piece_style, material) in materials.into_iter().flatten() {
                assert_eq!(piece_style, style);
                pieces += 1;
                matching += u32::from(material == set.material());
            }
        }
    }
    // Padding and some regional pieces can't take every metal.
    assert!(matching * 10 >= pieces * 8, "{matching} of {pieces}");
}

#[test]
fn a_coat_is_one_construction_with_a_skirt() {
    for set in rolled(SetKind::Coat) {
        let arms = set.arms.expect("a coat has sleeves");
        let waist = set.waist.expect("and a skirt");
        if arms.construction == set.construction() && waist.construction == set.construction() {
            continue;
        }
        // Only where the style makes no such sleeves or skirt.
        assert!(
            !set.torso.rigid() || set.construction() != Construction::Mail,
            "{set:?}"
        );
    }
}

#[test]
fn harnesses_are_named_by_their_style() {
    let names: Vec<String> = rolled(SetKind::Full).map(|set| set.name()).collect();
    assert!(names.iter().any(|name| name == "Full Mail"), "{names:?}");
    assert!(
        names.iter().any(|name| name.ends_with("Harness")),
        "{names:?}"
    );
    assert!(rolled(SetKind::Coat).any(|set| set.name() == "Hauberk"));
}
