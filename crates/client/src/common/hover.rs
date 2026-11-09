//! What the pointer is over, by name, for the hover box (`hover`).

use bevy::prelude::*;
use shared::protocol::SeenPoi;

/// What a token is called when the pointer is over it, such as a creature's
/// kind or another player's name. A token with this can be hovered even if it
/// can't be clicked (`view3d::HoveredToken`).
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct HoverName(pub String);

/// Another player shown as a marked role (`shared::role::is_marked`): what
/// the role is called, shown after their name with the staff mark before it.
/// Names can only hold letters, digits, spaces, `'` and `-`, so the mark, a
/// picture, can't be faked.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct StaffMark(pub String);

/// The name of the place on the site's map the pointer is over, such as a
/// building's door or stairs. Set by `dungeon_map`.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct HoveredOnMap(pub Option<String>);

/// The name of the sun or moon by the minimap the pointer is on. Set by
/// `minimap`.
#[derive(Resource, Default, Clone, Debug, PartialEq, Eq)]
pub struct HoveredInSky(pub Option<String>);

/// What a poi is called.
pub fn poi_name(poi: &SeenPoi) -> String {
    let ruined = if poi.look.is_ruined() { "Ruined" } else { "" };
    words(&format!("{ruined}{:?}", poi.kind))
}

/// Spaces out a name written in code style: "StoneTower" as "Stone tower".
pub fn words(name: &str) -> String {
    let mut out = String::new();
    for (index, letter) in name.chars().enumerate() {
        if letter.is_uppercase() && index > 0 {
            out.push(' ');
            out.extend(letter.to_lowercase());
        } else {
            out.push(letter);
        }
    }
    out
}
