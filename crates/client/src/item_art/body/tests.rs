use super::*;
use shared::body::{BEASTS, Beard, FIRST_BIPEDS, HairStyle};
use shared::creature::CreatureKind;

#[test]
fn every_body_stays_inside_its_picture_with_room_for_its_edge() {
    let kinds = CreatureKind::ALL
        .into_iter()
        .filter_map(Layout::of)
        .map(|layout| (layout, 1750));
    let coming = FIRST_BIPEDS.map(|biped| {
        let layout = Layout::biped(&biped.proportions.frame(), biped.parts);
        (layout, biped.height)
    });
    let beasts = BEASTS.map(|beast| (beast.layout(), 1750));
    for (layout, height) in kinds.chain(coming).chain(beasts) {
        let (wide, long) = cells(&layout);
        let (wide, long) = (wide as f32, long as f32);
        let per_cell = VIEW_WIDTH / wide.max(long);
        let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
        for styled in hairs() {
            let parts = fitted(&layout, height, styled, None);
            for corner in parts.iter().flatten().flatten() {
                assert!(corner.is_finite());
                assert!(corner.x.abs() + edge <= wide * per_cell / 2.0, "{layout:?}");
                assert!(corner.y.abs() + edge <= long * per_cell / 2.0, "{layout:?}");
            }
        }
    }
}

/// No hair, and every style and beard at least once, the largest together.
fn hairs() -> [Hair; 5] {
    let with = |style, beard| Hair { style, beard };
    [
        Hair::NONE,
        with(HairStyle::Long, Beard::Long),
        with(HairStyle::Braids, Beard::Braided),
        with(HairStyle::Curly, Beard::Forked),
        with(HairStyle::TiedBack, Beard::Full),
    ]
}

#[test]
fn horses_have_manes_and_tails_of_hair() {
    let drawn = |kind| coat_hair(&Layout::of(kind).expect("drawn"));
    let horse = drawn(CreatureKind::Horse);
    assert!(!horse.is_empty());
    // A centaur keeps the horse's tail but has no horse's neck for a mane, so less.
    let centaur = drawn(CreatureKind::Centaur);
    assert!(!centaur.is_empty() && centaur.len() < horse.len());
    assert!(!drawn(CreatureKind::Cattle).is_empty());
    assert!(drawn(CreatureKind::Human).is_empty());
    assert!(drawn(CreatureKind::Wolf).is_empty());
    // Hair hangs off the ground.
    assert!(horse.iter().flatten().all(|corner| corner.y > 0.0));
}
