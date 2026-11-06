//! Each ability's icon: a game-icons.net icon drawn by `cargo xtask icons` (each one
//! must be listed in `xtask/icons.txt`).

use shared::ability::Ability;

/// Where an ability's icon is, in the assets.
pub(super) fn icon_path(ability: Ability) -> &'static str {
    match ability {
        Ability::Attack => "icons/lorc/pointy-sword.png",
        Ability::Flutter => "icons/lorc/feathered-wing.png",
        Ability::Kick => "icons/delapouite/high-kick.png",
        Ability::NightVision => "icons/delapouite/night-vision.png",
        Ability::Trample => "icons/lorc/boot-stomp.png",
    }
}
