use super::*;
use crate::body::{FIRST_BIPEDS, HUMAN, Layout, NONE};

/// Every frame people are built on: a person's, the giant's, and the first peoples'.
fn frames() -> Vec<Frame> {
    let mut frames = vec![HUMAN, crate::body::GIANT];
    frames.extend(FIRST_BIPEDS.iter().map(|biped| biped.proportions.frame()));
    frames
}

#[test]
fn rings_rise_in_order_from_the_crotch_to_the_neck() {
    for frame in frames() {
        let rings = PERSON_TRUNK.rings(&frame);
        assert_eq!(rings[0].along, -frame.thigh, "{frame:?}");
        assert_eq!(rings[RINGS - 1].along, frame.neck_base - frame.hip);
        for pair in rings.windows(2) {
            assert!(pair[0].along < pair[1].along, "{rings:?}");
        }
        for ring in rings {
            assert!(
                ring.half > 0 && ring.belly > 0 && ring.back > 0,
                "{rings:?}"
            );
        }
    }
}

#[test]
fn the_ends_are_as_they_were_and_the_chest_stands_forwards() {
    let rings = PERSON_TRUNK.rings(&HUMAN);
    // The crotch as wide and deep as the hips, the neck's base round the neck.
    assert_eq!(rings[0].half, HUMAN.hips_half.0);
    assert_eq!(rings[0].belly, rings[0].back);
    assert_eq!(rings[RINGS - 1].belly, rings[RINGS - 1].back);
    // Between, the front stands out further than the back, most at the belly and
    // chest, where the trunk is as deep as the frame says.
    let chest = rings
        .iter()
        .min_by_key(|ring| (ring.along - (HUMAN.chest - HUMAN.hip)).abs())
        .unwrap();
    assert!(chest.belly > chest.back + 50, "{chest:?}");
    let deep = chest.belly + chest.back;
    assert!((deep - 2 * HUMAN.chest_half.1).abs() < 20, "{chest:?}");
}

#[test]
fn a_biped_layout_is_shaped_by_its_frame() {
    for frame in frames() {
        let layout = Layout::biped(&frame, NONE);
        assert_eq!(layout.spine.rings, PERSON_TRUNK.rings(&frame));
        assert_eq!(layout.frame(), Some(frame));
    }
}
