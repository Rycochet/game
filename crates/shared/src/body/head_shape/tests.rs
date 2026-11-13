use super::*;
use crate::body::{BEASTS, BLEND_WHOLE, Layout, NONE, PERSON};
use crate::creature::CreatureKind;

#[test]
fn a_plain_head_is_its_oval_above_the_neck() {
    let person = Layout::biped(&PERSON.frame(), NONE);
    let plain = HeadShape::plain(&person.head, &person.parts);
    let (across, deep) = person.head.half;
    assert_eq!(plain.rings[0].along, -deep);
    assert_eq!(plain.rings[HEAD_RINGS - 1].along, deep);
    // Nothing below the neck's top, nothing above the crown.
    for ring in plain.rings {
        assert!(
            ring.belly <= 0 && ring.back <= person.head.length,
            "{ring:?}"
        );
        assert!(ring.half <= across);
    }
}

#[test]
fn a_snout_reaches_past_the_oval() {
    let lizard = Layout::of(CreatureKind::Lizardfolk).expect("drawn");
    let plain = HeadShape::plain(&lizard.head, &lizard.parts);
    let reach = lizard.head.half.1 + lizard.parts.snout_length;
    assert_eq!(plain.rings[HEAD_RINGS - 1].along, reach);
    assert!(plain.rings[HEAD_RINGS - 1].half > 0);
}

#[test]
fn every_beast_on_four_legs_has_a_measured_head() {
    for beast in BEASTS {
        let layout = beast.layout();
        let shape = layout.head.shape.expect(beast.name);
        let rings = shape.rings;
        assert!(
            rings.windows(2).all(|pair| pair[0].along < pair[1].along),
            "{}",
            beast.name
        );
        assert!(
            rings[0].along < 0,
            "{}: the skull reaches behind the neck",
            beast.name
        );
        assert_eq!((layout.head.length, layout.head.half), shape.bounds());
    }
}

#[test]
fn a_person_grows_a_wolfs_head_and_is_still_itself_at_each_end() {
    let person = Layout::at_size(CreatureKind::Human).expect("drawn");
    let wolf = Layout::at_size(CreatureKind::Wolf).expect("drawn");
    assert_eq!(Layout::blend(&person, &wolf, 0).head, person.head);
    assert_eq!(Layout::blend(&person, &wolf, BLEND_WHOLE).head, wolf.head);
    let half = Layout::blend(&person, &wolf, BLEND_WHOLE / 2).head.shape;
    assert!(half.is_some());
}

#[test]
fn the_neck_starts_at_the_spines_end_of_a_body_from_its_stations() {
    for kind in CreatureKind::ALL {
        let Some(layout) = Layout::of(kind) else {
            continue;
        };
        if layout.head.shape.is_none() && layout.upper.is_none() {
            let spine = &layout.spine;
            assert_eq!(spine.neck_root(), spine.at(spine.neck_base), "{kind:?}");
        }
    }
}
