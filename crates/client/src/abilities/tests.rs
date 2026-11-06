use shared::ability::Ability;

use super::icons::icon_path;
use super::page::ability_id;

/// Every ability's icon has been drawn into the assets (`cargo xtask icons`).
#[test]
fn every_ability_has_an_icon() {
    for ability in Ability::ALL {
        let file = format!(
            "{}/assets/{}",
            env!("CARGO_MANIFEST_DIR"),
            icon_path(ability)
        );
        assert!(std::path::Path::new(&file).exists(), "{file}");
    }
}

#[test]
fn ids_are_plain_names() {
    assert_eq!(ability_id(Ability::NightVision), "night_vision");
    assert_eq!(ability_id(Ability::Attack), "attack");
}

mod dropping {
    use shared::ability::{Ability, BarChange};

    use super::super::drag::{From, drop_change};
    use super::super::spots::Spot;

    const ATTACK: Ability = Ability::Attack;

    #[test]
    fn from_the_page_onto_a_slot_puts_it_there() {
        assert_eq!(
            drop_change(ATTACK, From::Page, Some(Spot::Slot(3))),
            Some(BarChange::Place {
                slot: 3,
                ability: ATTACK
            })
        );
    }

    #[test]
    fn from_slot_to_slot_swaps_and_onto_itself_does_nothing() {
        assert_eq!(
            drop_change(ATTACK, From::Slot(1), Some(Spot::Slot(4))),
            Some(BarChange::Swap { a: 1, b: 4 })
        );
        assert_eq!(
            drop_change(ATTACK, From::Slot(4), Some(Spot::Slot(4))),
            None
        );
    }

    #[test]
    fn from_a_slot_onto_the_page_takes_it_off_and_elsewhere_nothing() {
        for spot in [Spot::Page, Spot::Listed(Ability::Kick)] {
            assert_eq!(
                drop_change(ATTACK, From::Slot(2), Some(spot)),
                Some(BarChange::Clear { slot: 2 })
            );
        }
        assert_eq!(drop_change(ATTACK, From::Slot(2), None), None);
        assert_eq!(drop_change(ATTACK, From::Page, Some(Spot::Page)), None);
        assert_eq!(drop_change(ATTACK, From::Page, None), None);
    }
}
