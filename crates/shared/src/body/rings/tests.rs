use super::*;
use crate::body::{BEASTS, Layout, PERSON, Plan};
use crate::creature::CreatureKind;

#[test]
fn a_body_from_its_stations_keeps_its_six_levels() {
    // Bipeds are shaped as measured now; their stations still make rings like this.
    let layout = Layout::biped(&PERSON.frame(), crate::body::NONE);
    let rings = from_stations(&layout);
    let s = &layout.spine;
    assert_eq!(rings[1], Ring::round(0, s.hips.half));
    assert_eq!(rings[3], Ring::round(s.waist.along, s.waist.half));
    assert_eq!(rings[5], Ring::round(s.chest.along, s.chest.half));
    assert_eq!(rings[7].along, s.shoulders);
    assert_eq!(rings[9].along, s.neck_base);
    // The rings between are halfway, and every ring is as deep in front as behind.
    for at in [2, 4, 6, 8] {
        assert_eq!(rings[at], Ring::between(rings[at - 1], rings[at + 1]));
    }
    assert!(rings.iter().all(|ring| ring.belly == ring.back));
}

#[test]
fn rings_run_from_the_rump_to_the_neck() {
    for kind in CreatureKind::ALL {
        let Some(layout) = Layout::at_size(kind) else {
            continue;
        };
        let rings = layout.spine.rings;
        assert!(
            rings.windows(2).all(|pair| pair[0].along < pair[1].along),
            "{kind:?}"
        );
        assert!(rings[0].along <= 0, "{kind:?}");
        assert!(
            rings
                .iter()
                .all(|ring| ring.half >= 0 && ring.belly + ring.back >= 0),
            "{kind:?}"
        );
    }
}

/// A beast's rings, by name.
fn beast(name: &str) -> [Ring; RINGS] {
    let beast = BEASTS
        .into_iter()
        .find(|beast| beast.name == name)
        .expect("a beast");
    assert!(matches!(beast.plan, Plan::Fourfoot(_)));
    beast.layout().spine.rings
}

#[test]
fn a_wolfs_belly_is_tucked_up_behind_its_chest() {
    let rings = beast("Wolf");
    // Below a level spine, the belly is the ring's front: less of it is a higher belly.
    let chest = rings[7].belly;
    let loin = rings[3].belly;
    assert!(loin * 2 < chest, "loin {loin}, chest {chest}");
}

#[test]
fn a_cows_belly_rises_far_less_than_a_wolfs() {
    let rise = |rings: [Ring; RINGS]| rings[3].belly * 100 / rings[7].belly;
    let (cow, wolf) = (rise(beast("Cattle")), rise(beast("Wolf")));
    assert!(cow > 60, "cow {cow}");
    assert!(wolf < 30, "wolf {wolf}");
}

#[test]
fn a_beasts_rings_reach_its_withers() {
    for (name, withers) in [("Horse", 1580), ("Wolf", 740), ("Cattle", 1350)] {
        let layout = BEASTS
            .into_iter()
            .find(|beast| beast.name == name)
            .expect("a beast")
            .layout();
        // The top of the back, as high as the spine plus the ring's back, near the
        // shoulders is about the withers.
        let top = layout.spine.base[1] + layout.spine.rings[7].back;
        assert!((top - withers).abs() * 10 < withers, "{name}: {top}");
    }
}
