use shared::id::Id;

use super::super::{ATTACK_GAP, Doing, FRIENDLY_GAP};

#[test]
fn only_going_after_an_enemy_targets_them() {
    let them = Id::for_test(7);
    let going = |gap| Doing::Going {
        after: them,
        last_seen: (1, 2),
        gap,
    };
    assert_eq!(going(ATTACK_GAP).target(), Some(them));
    assert_eq!(going(FRIENDLY_GAP).target(), None);
    let tracking = Doing::Tracking {
        after: them,
        gap: ATTACK_GAP,
        since: 0,
    };
    assert_eq!(tracking.target(), Some(them));
    let following = Doing::Following {
        after: them,
        stairs: (0, 0),
        to_level: 1,
        arrives: (0, 0),
        gap: ATTACK_GAP,
    };
    assert_eq!(following.target(), Some(them));
    for nobody in [
        Doing::Idle,
        Doing::Fleeing {
            from: them,
            struck: false,
        },
        Doing::Settling { steps_left: 3 },
        Doing::GoingHome { since: 0 },
        Doing::Guarding {
            against: them,
            last_seen: (0, 0),
        },
    ] {
        assert_eq!(nobody.target(), None, "{nobody:?}");
    }
}
