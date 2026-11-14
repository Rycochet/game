//! A `dark` panel: the window behind it is darkened while it shows, as behind
//! a dialog, and nothing under the darkening can be pointed at or pressed. A
//! press on the darkening is left for the panel's owner to read (`OwnPointer`),
//! to close it or drag what it shows. It goes with its document.

use bevy::prelude::*;

use super::parts::{Darkened, Element, OwnPointer, SurfaceOf};
use crate::widgets::darkening;

/// The darkening for a `dark` panel in `document`, as layer `depth` (its
/// `depth`): the document goes in front of it, a little above that layer
/// (`widgets::layer_z`).
pub(super) fn spawn(commands: &mut Commands, depth: i32, hidden: bool, document: Entity) -> Entity {
    let shown = match hidden {
        true => Visibility::Hidden,
        false => Visibility::Inherited,
    };
    commands
        .spawn((darkening(depth), OwnPointer, SurfaceOf(document), shown))
        .id()
}

/// Shows a darkening while its panel does, as its `hidden` says.
pub(super) fn follow_darkenings(
    changed: Query<(&Element, &Darkened), Changed<Element>>,
    mut darkenings: Query<&mut Visibility>,
) {
    for (element, Darkened(darkening)) in &changed {
        let wanted = match element.attrs.hidden {
            true => Visibility::Hidden,
            false => Visibility::Inherited,
        };
        if let Ok(mut shown) = darkenings.get_mut(*darkening) {
            shown.set_if_neq(wanted);
        }
    }
}

#[cfg(test)]
mod tests;
