use super::*;
use crate::body::Frame;
use crate::creature::Size;
use crate::sheet::Choice;

/// The size band a height in millimetres falls in (`Size::quarters`' bands).
fn band(height: i32) -> Size {
    const TOPS: [i32; 5] = [750, 1500, 2100, 3200, 4500];
    Size::ALL[TOPS.iter().filter(|&&top| height >= top).count()]
}

#[test]
fn shares_follow_the_choices_in_order() {
    let names: Vec<_> = Trait::Height
        .choices_for(CreatureKind::Human)
        .iter()
        .map(|choice: &Choice| choice.name)
        .collect();
    assert_eq!(
        names,
        ["Average", "Very short", "Short", "Tall", "Very tall"]
    );
    assert_eq!(names.len(), HEIGHT_SHARES.len());
    for (chosen, share) in HEIGHT_SHARES.into_iter().enumerate() {
        let looks = Appearance([(Trait::Height, chosen as u8)].into());
        assert_eq!(looks.height_share(CreatureKind::Human), share);
    }
    assert_eq!(
        Appearance::default().height_share(CreatureKind::Human),
        1000
    );
}

#[test]
fn kinds_without_a_height_look_stand_as_they_are() {
    let looks = Appearance([(Trait::Height, 4)].into());
    assert_eq!(looks.height_share(CreatureKind::Chicken), HEIGHT_WHOLE);
}

/// Owner's rule (2026-11-14): no character or mob more than two size steps from its
/// kind's own size, whatever its Height.
#[test]
fn every_height_stays_within_two_sizes_of_the_kind() {
    for kind in CreatureKind::ALL {
        if !kind.def().looks.contains(&Trait::Height) {
            continue;
        }
        let own = kind.def().size;
        // How tall it usually stands: a biped's own height, or else the middle of its
        // size's band.
        let usual = match Frame::of(kind) {
            Some(_) => Frame::usual_height(kind),
            None => own.quarters() as i32 * 1750 / 4,
        };
        for share in HEIGHT_SHARES {
            let size = band(usual * share / HEIGHT_WHOLE);
            let steps = (size as i32 - own as i32).abs();
            assert!(steps <= 2, "{kind:?} at {share}‰ is {size:?}, own {own:?}");
        }
    }
}
