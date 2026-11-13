//! Changing what a `select`, a `tab-bar` or a `toggle-row` offers, for lists
//! only known while playing (the difficulties a server allows, a monitor's
//! resolutions, the tabs a role may see).

use bevy::prelude::*;

use crate::dom::parts::{Choices, Control, Element};
use crate::widgets::{Dropdown, set_tabs};

/// Gives a `select`, a `tab-bar` or a `toggle-row` these options instead of
/// its own, each its value and its words. Its `value` still says which is
/// chosen, if it is among them; if not, the first is.
pub(super) fn set_options(
    world: &mut World,
    entity: Entity,
    options: Vec<(String, String)>,
) -> Result<(), String> {
    const WRONG: &str = "only a select's, a tab bar's or a toggle row's options can be changed";
    if options.is_empty() {
        return Err("it needs at least one option".into());
    }
    let control = world.get::<Control>(entity).copied();
    let Control(widget) = control.ok_or(WRONG)?;
    let (values, words): (Vec<String>, Vec<String>) = options.into_iter().unzip();
    if let Some(mut dropdown) = world.get_mut::<Dropdown>(widget) {
        dropdown.options = words;
        dropdown.greyed.clear();
        dropdown.icons.clear();
        dropdown.selected = 0;
    } else if !set_tabs(world, widget, &words) {
        return Err(WRONG.into());
    }
    world.entity_mut(entity).insert(Choices(values));
    // Marked as changed, so its `value` is looked up among the new options.
    if let Some(mut element) = world.get_mut::<Element>(entity) {
        element.set_changed();
    }
    Ok(())
}

/// Gives a `select`'s options a picture each.
pub(super) fn set_option_pictures(
    world: &mut World,
    entity: Entity,
    pictures: Vec<Handle<Image>>,
) -> Result<(), String> {
    const WRONG: &str = "only a select's options can have pictures";
    let control = world.get::<Control>(entity).copied();
    let Control(widget) = control.ok_or(WRONG)?;
    let mut dropdown = world.get_mut::<Dropdown>(widget).ok_or(WRONG)?;
    dropdown.icons = pictures;
    Ok(())
}
