//! Changing what a `select` offers, for lists only known while playing
//! (the difficulties a server allows, a monitor's resolutions).

use bevy::prelude::*;

use crate::dom::parts::{Choices, Control, Element};
use crate::widgets::Dropdown;

/// Gives a `select` these options instead of its own, each its value and
/// its words. Its `value` still says which is chosen, if it is among them;
/// if not, the first is.
pub(super) fn set_options(
    world: &mut World,
    entity: Entity,
    options: Vec<(String, String)>,
) -> Result<(), String> {
    if options.is_empty() {
        return Err("a select needs at least one option".into());
    }
    let control = world.get::<Control>(entity).copied();
    let Control(widget) = control.ok_or("only a select's options can be changed")?;
    let dropdown = world.get_mut::<Dropdown>(widget);
    let mut dropdown = dropdown.ok_or("only a select's options can be changed")?;
    let (values, words): (Vec<String>, Vec<String>) = options.into_iter().unzip();
    dropdown.options = words;
    dropdown.greyed.clear();
    dropdown.icons.clear();
    dropdown.selected = 0;
    world.entity_mut(entity).insert(Choices(values));
    // Marked as changed, so its `value` is looked up among the new options.
    if let Some(mut element) = world.get_mut::<Element>(entity) {
        element.set_changed();
    }
    Ok(())
}
