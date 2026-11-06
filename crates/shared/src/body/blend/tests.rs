use super::super::layout::MAX_LIMBS;
use super::super::{Horns, Side, Snout};
use super::*;
use crate::creature::CreatureKind;

/// Every kind with a body, at its real size.
fn bodies() -> Vec<(CreatureKind, Layout)> {
    CreatureKind::ALL
        .into_iter()
        .filter_map(|kind| Layout::at_size(kind).map(|layout| (kind, layout)))
        .collect()
}

#[test]
fn each_end_of_a_blend_is_its_own_body() {
    let bodies = bodies();
    for (ka, a) in &bodies {
        for (kb, b) in &bodies {
            assert_eq!(Layout::blend(a, b, 0), *a, "{ka:?} into {kb:?} at 0");
            assert_eq!(
                Layout::blend(a, b, WHOLE),
                *b,
                "{ka:?} into {kb:?} at the end"
            );
        }
    }
}

#[test]
fn every_limb_has_a_slot_all_the_way() {
    let bodies = bodies();
    for (ka, a) in &bodies {
        for (kb, b) in &bodies {
            // Each side needs as many slots as the body with more limbs on it has.
            let on = |layout: &Layout, side| layout.limbs().filter(|l| l.side == side).count();
            let needed: usize = Side::BOTH
                .map(|side| on(a, side).max(on(b, side)))
                .iter()
                .sum();
            assert!(needed <= MAX_LIMBS, "{ka:?} into {kb:?}");
            let fewest = a.limbs().count().min(b.limbs().count());
            for t in (0..=WHOLE).step_by(50) {
                let blended = Layout::blend(a, b, t);
                let count = blended.limbs().count();
                assert!(
                    (fewest..=needed).contains(&count),
                    "{ka:?} into {kb:?} at {t}"
                );
                // Every limb that is there hangs from something and has a first bone.
                for limb in blended.limbs() {
                    assert!(limb.bones[0].is_some_and(|bone| bone.span > 0));
                }
            }
        }
    }
}

#[test]
fn arms_become_forelegs_and_legs_hind_legs() {
    let person = Layout::at_size(CreatureKind::Human).expect("a person");
    let horse = Layout::at_size(CreatureKind::Horse).expect("a horse");
    let halfway = Layout::blend(&person, &horse, HALF);
    // Four limbs paired four ways: none grows or shrinks alone.
    assert_eq!(halfway.limbs().count(), 4);
    let roles: Vec<_> = halfway.limbs().map(|limb| limb.role).collect();
    use super::super::Role::*;
    assert!(
        roles.iter().all(|role| matches!(role, Foreleg | Hindleg)),
        "{roles:?}"
    );
    // Just before halfway they are still the person's.
    let before = Layout::blend(&person, &horse, HALF - 1);
    assert!(before.limbs().all(|limb| matches!(limb.role, Arm | Leg)));
}

#[test]
fn limbs_with_no_partner_grow_from_nothing() {
    let person = Layout::at_size(CreatureKind::Human).expect("a person");
    let drider = Layout::at_size(CreatureKind::Drider).expect("a drider");
    assert_eq!(Layout::blend(&person, &drider, 0).limbs().count(), 4);
    assert_eq!(Layout::blend(&person, &drider, 1).limbs().count(), 4);
    assert_eq!(Layout::blend(&person, &drider, 100).limbs().count(), 8);
    // The spare legs are short early on, and full grown only at the end.
    let reach = |layout: &Layout| -> i32 {
        layout
            .limbs()
            .map(|limb| limb.bones.iter().flatten().map(|b| b.span).sum::<i32>())
            .min()
            .unwrap_or(0)
    };
    let early = reach(&Layout::blend(&person, &drider, 100));
    let late = reach(&Layout::blend(&person, &drider, 900));
    assert!(early < late, "{early} {late}");
}

#[test]
fn snouts_change_kind_only_at_nothing() {
    let person = Layout::at_size(CreatureKind::Human).expect("a person");
    let lizard = Layout::at_size(CreatureKind::Lizardfolk).expect("lizardfolk");
    assert_ne!(lizard.parts.snout, Snout::Flat);
    for t in (0..=WHOLE).step_by(25) {
        let parts = Layout::blend(&person, &lizard, t).parts;
        if parts.snout != person.parts.snout && parts.snout != lizard.parts.snout {
            panic!("a third kind of snout at {t}");
        }
    }
    let half = Layout::blend(&person, &lizard, HALF).parts;
    assert_eq!((half.snout, half.snout_length), (lizard.parts.snout, 0));
    // Horns of one kind only grow.
    let minotaur = Layout::at_size(CreatureKind::Minotaur).expect("a minotaur");
    let quarter = Layout::blend(&person, &minotaur, HALF / 2).parts;
    assert_eq!(quarter.horns, Horns::None);
    assert_eq!(quarter.horn_length, 0);
}

#[test]
fn a_taurs_upper_body_grows_out_of_the_back() {
    let person = Layout::at_size(CreatureKind::Human).expect("a person");
    let centaur = Layout::at_size(CreatureKind::Centaur).expect("a centaur");
    assert!(Layout::blend(&person, &centaur, 0).upper.is_none());
    let mut last = 0;
    for t in (100..=WHOLE).step_by(100) {
        let upper = Layout::blend(&person, &centaur, t).upper.expect("growing");
        assert!(upper.frame.height > last, "{t}");
        last = upper.frame.height;
    }
    // And sinks back in the other way.
    assert!(Layout::blend(&centaur, &person, WHOLE).upper.is_none());
    assert!(Layout::blend(&centaur, &person, WHOLE - 1).upper.is_some());
}

#[test]
fn whole_roots() {
    for n in [
        0u64,
        1,
        2,
        3,
        4,
        15,
        16,
        17,
        1_000_000,
        2_000_000,
        u32::MAX as u64,
    ] {
        let root = limbs::whole_root(n);
        assert!(root * root <= n && (root + 1) * (root + 1) > n, "{n}");
    }
}
