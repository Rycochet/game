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
