use super::*;

fn frames() -> impl Iterator<Item = Frame> {
    CreatureKind::ALL.into_iter().filter_map(Frame::of)
}

#[test]
fn every_kind_on_two_legs_has_a_frame() {
    for kind in CreatureKind::ALL {
        let def = kind.def();
        let biped = def.part(crate::creature::Location::Legs).is_some()
            && def.part(crate::creature::Location::Arms).is_some();
        assert_eq!(Frame::of(kind).is_some(), biped, "{kind:?}");
    }
}

#[test]
fn levels_go_down_in_order() {
    for frame in frames() {
        let levels = [
            frame.height,
            frame.chin,
            frame.neck_base,
            frame.shoulder,
            frame.chest,
            frame.waist,
            frame.hip,
            frame.knee,
            frame.ankle,
            0,
        ];
        assert!(levels.windows(2).all(|pair| pair[0] > pair[1]), "{frame:?}");
        // The head is as tall as it is said to be deep.
        assert!(frame.height - frame.chin >= frame.head.1 * 2);
    }
}

#[test]
fn hands_hang_beside_the_thighs_and_sides_mirror() {
    for frame in frames() {
        let tips = frame.joint(Joint::Fingertips(Side::Left));
        assert!(tips[1] > frame.knee && tips[1] < frame.hip, "{frame:?}");
        assert!(tips[0] > frame.hips_half.0, "{frame:?}");
        let right = frame.joint(Joint::Elbow(Side::Right));
        let left = frame.joint(Joint::Elbow(Side::Left));
        assert_eq!(right, [-left[0], left[1], left[2]]);
    }
}

#[test]
fn a_person_from_proportions_is_the_person_drawn_before() {
    assert_eq!(PERSON.frame(), HUMAN);
}

#[test]
fn other_proportions_keep_the_levels_in_order() {
    // Rough dwarf, elf and goblin rows from the creature research.
    let samples = [
        Proportions {
            heads: 53,
            breadth: 1250,
            limbs: 1300,
            arms: 950,
            legs: 863,
        },
        Proportions {
            heads: 85,
            breadth: 900,
            limbs: 800,
            arms: 1030,
            legs: 1059,
        },
        Proportions {
            heads: 53,
            breadth: 850,
            limbs: 800,
            arms: 1100,
            legs: 902,
        },
    ];
    for frame in samples.map(|p| p.frame()) {
        let levels = [
            frame.height,
            frame.chin,
            frame.neck_base,
            frame.shoulder,
            frame.chest,
            frame.waist,
            frame.hip,
            frame.knee,
            frame.ankle,
            0,
        ];
        assert!(levels.windows(2).all(|pair| pair[0] > pair[1]), "{frame:?}");
        assert!(frame.height - frame.chin >= frame.head.1 * 2, "{frame:?}");
    }
}
