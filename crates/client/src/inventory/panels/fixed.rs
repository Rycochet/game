//! The parts that are always there: the inventory button, the held item, the bin and the ground.

use bevy::prelude::*;

use super::{BinIcon, DropIcon, InventoryButton, bin_icon, drop_icon, person_icon};
use crate::dom::{Pictures, Template, bottom_strip, document};
use crate::html;
use crate::widgets::{ButtonTooltip, UiId};

/// The button, in the window's bottom left corner (8 from it, 34 across). It
/// has no form round it: a form on a strip has no height, and the pointer is
/// never on what is outside its form.
static BUTTON: Template = html!(
    "inventory.button",
    r#"
<button id="character" form="hud" left="8" bottom="8" width="34" height="34">
  <img src="picture:person"/>
</button>
"#
);

/// The button, and the bin's and Drop's pictures.
pub(in crate::inventory) fn spawn_fixed_parts(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut pictures: ResMut<Pictures>,
) {
    pictures.name("person", images.add(person_icon()));
    commands.insert_resource(BinIcon(images.add(bin_icon())));
    commands.insert_resource(DropIcon(images.add(drop_icon())));
    // Above the maps, as the warning button is.
    let button = (document(&BUTTON), GlobalZIndex(15));
    commands.spawn(button).insert(bottom_strip());
}

/// Marks the button once it is built, with its tip.
pub(in crate::inventory) fn bind_button(
    new: Query<(Entity, &UiId), Added<UiId>>,
    mut commands: Commands,
) {
    for (entity, id) in &new {
        if id.parts() == ("hud", "character") {
            let tip = ButtonTooltip("Character".into());
            commands.entity(entity).insert((InventoryButton, tip));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_button_is_good_markup() {
        super::BUTTON.assert_good();
    }
}
