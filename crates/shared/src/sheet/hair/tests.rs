use std::collections::BTreeMap;

use super::*;

#[test]
fn choices_name_the_styles_in_order() {
    for kind in CreatureKind::ALL {
        let names = |choices: &[Choice]| choices.iter().map(|c| c.name).collect::<Vec<_>>();
        let styles: Vec<_> = styles_for(kind).iter().map(|s| s.name()).collect();
        assert_eq!(names(style_choices(kind)), styles, "{kind:?}");
        let beards: Vec<_> = beards_for(kind).iter().map(|b| b.name()).collect();
        assert_eq!(names(beard_choices(kind)), beards, "{kind:?}");
    }
}

#[test]
fn kinds_with_hair_colour_have_a_style() {
    for kind in CreatureKind::ALL {
        let looks = kind.def().looks;
        assert_eq!(
            looks.contains(&Trait::HairColour),
            looks.contains(&Trait::HairStyle),
            "{kind:?}"
        );
        if looks.contains(&Trait::Beard) {
            assert!(looks.contains(&Trait::HairStyle), "{kind:?}");
        }
    }
}

#[test]
fn nothing_chosen_is_the_first() {
    let none = Appearance::default();
    assert_eq!(
        none.hair(CreatureKind::Dwarf),
        Hair {
            style: HairStyle::Long,
            beard: Beard::Long,
        }
    );
    assert_eq!(
        none.hair(CreatureKind::Human),
        Hair {
            style: HairStyle::Short,
            beard: Beard::None,
        }
    );
}

#[test]
fn kinds_without_hair_are_bald() {
    let chosen = Appearance(BTreeMap::from([(Trait::HairStyle, 1), (Trait::Beard, 2)]));
    assert_eq!(chosen.hair(CreatureKind::Goblin), Hair::NONE);
    assert_eq!(chosen.hair(CreatureKind::Horse), Hair::NONE);
    // An elf has styles but no beards.
    assert_eq!(chosen.hair(CreatureKind::Elf).beard, Beard::None);
    assert_eq!(chosen.hair(CreatureKind::Elf).style, HairStyle::Braids);
}
