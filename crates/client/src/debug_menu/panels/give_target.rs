//! Who Give Item gives to: the target while it is in sight (a mob or another
//! player), else the character. What it gives is made for them, and the line by
//! Give says who it is.

use bevy::prelude::*;
use shared::creature::CreatureKind;
use shared::id::Id;

use crate::common::{Other, Target};
use crate::dom::Dom;

/// The target being given to, and what it is; `None` for the character.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::debug_menu) struct GiveTo(pub Option<(Id, CreatureKind)>);

/// The words by Give saying who gets it.
#[derive(Component)]
pub(in crate::debug_menu) struct GiveToWords;

/// Follows the target: only one in sight, as the server only gives to them.
pub(in crate::debug_menu) fn follow_target(
    target: Res<Target>,
    others: Query<&Other>,
    mut to: ResMut<GiveTo>,
    words: Query<(), With<GiveToWords>>,
    mut dom: Dom,
    mut shown: Local<Option<String>>,
) {
    let seen = target
        .0
        .and_then(|id| others.iter().find(|other| other.id == id))
        .map(|other| (other.id, other.kind));
    to.set_if_neq(GiveTo(seen));
    let said = match seen {
        Some((_, kind)) => format!("to the {}", kind.def().name),
        None => "to you".to_owned(),
    };
    // Only with a panel, which opens saying "to you".
    if words.is_empty() {
        *shown = None;
    } else if shown.as_ref() != Some(&said) {
        dom.set_text("debug.give_item/to", said.clone());
        *shown = Some(said);
    }
}
