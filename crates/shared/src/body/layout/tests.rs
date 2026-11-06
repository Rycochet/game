use super::super::trig::{cos, sin};
use super::super::{Frame, HUMAN, Joint};
use super::*;

/// Every kind's frame as it was worked out before layouts.
fn frame_before(kind: CreatureKind) -> Option<Frame> {
    match kind {
        CreatureKind::Human | CreatureKind::Beggar => Some(PERSON.frame()),
        CreatureKind::Giant => Some(GIANT),
        _ => Biped::of(kind).map(|biped| biped.proportions.frame()),
    }
}

#[test]
fn every_frame_is_the_same_seen_through_its_layout() {
    for kind in CreatureKind::ALL {
        assert_eq!(Frame::of(kind), frame_before(kind), "{kind:?}");
    }
    assert_eq!(Frame::of(CreatureKind::Human), Some(HUMAN));
}

#[test]
fn limbs_joints_are_the_frames_joints() {
    for kind in CreatureKind::ALL {
        let Some(layout) = Layout::of(kind) else {
            continue;
        };
        let Some(frame) = layout.frame() else {
            continue;
        };
        for side in Side::BOTH {
            let arm = layout
                .limb(Role::Arm, side)
                .or_else(|| layout.limb(Role::WingArm, side))
                .expect("arms");
            let expected = [
                Joint::Shoulder(side),
                Joint::Elbow(side),
                Joint::Wrist(side),
                Joint::Fingertips(side),
            ]
            .map(|joint| frame.joint(joint));
            let joints = layout.joints(arm);
            for (got, wanted) in joints.iter().zip(expected) {
                // Whole millimetres rounded along the way may differ by one.
                let off = (0..3).map(|i| (got[i] - wanted[i]).abs()).max();
                assert!(off <= Some(1), "{kind:?} {got:?} {wanted:?}");
            }
            let leg = layout.limb(Role::Leg, side).expect("legs");
            let joints = layout.joints(leg);
            let expected = [Joint::Hip(side), Joint::Knee(side), Joint::Ankle(side)]
                .map(|joint| frame.joint(joint));
            assert_eq!(joints[..3], expected, "{kind:?}");
            assert_eq!(joints[3][1], 0, "{kind:?}: the foot stands on the ground");
        }
    }
}

#[test]
fn bipeds_and_the_first_animals_have_layouts() {
    for kind in CreatureKind::ALL {
        let drawn = Frame::of(kind).is_some() || Beast::of(kind).is_some();
        assert_eq!(Layout::of(kind).is_some(), drawn, "{kind:?}");
    }
    assert!(Layout::of(CreatureKind::Horse).is_some());
    assert!(Layout::of(CreatureKind::Chicken).is_some());
    assert!(Layout::of(CreatureKind::Centaur).is_none());
}

#[test]
fn every_beast_stands_on_the_ground_without_a_frame() {
    for beast in super::super::BEASTS {
        let layout = beast.layout();
        assert_eq!(layout.frame(), None, "{}", beast.name);
        let mut legs = 0;
        for limb in layout.limbs() {
            let joints = layout.joints(limb);
            assert_eq!(
                joints.last().map(|joint| joint[1]),
                Some(0),
                "{}",
                beast.name
            );
            legs += 1;
        }
        let four = matches!(beast.plan, super::super::Plan::Fourfoot(_));
        assert_eq!(legs, if four { 4 } else { 2 }, "{}", beast.name);
        // The head is above the back.
        assert!(layout.chin()[1] > layout.spine.at(layout.spine.neck_base)[1]);
    }
}

#[test]
fn a_spine_leans_forwards_as_it_tilts() {
    let mut spine = Layout::of(CreatureKind::Human).expect("people").spine;
    let [x, y, z] = spine.base;
    assert_eq!(spine.at(500), [x, y + 500, z]);
    spine.pitch = 90;
    assert_eq!(spine.at(500), [x, y, z + 500]);
    spine.pitch = 30;
    assert_eq!(spine.at(1000), [x, y + 866, z + 500]);
}

#[test]
fn sines_from_the_table() {
    assert_eq!(sin(0), 0);
    assert_eq!(sin(30), 5000);
    assert_eq!(sin(90), 10_000);
    assert_eq!(sin(150), 5000);
    assert_eq!(sin(-90), -10_000);
    assert_eq!(sin(390), 5000);
    assert_eq!(cos(0), 10_000);
    assert_eq!(cos(60), 5000);
    assert_eq!(cos(180), -10_000);
}
