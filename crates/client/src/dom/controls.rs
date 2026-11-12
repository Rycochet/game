//! Controls: the elements a player uses. Each is shown by one of the old
//! widgets, spawned inside the element, so it looks and works as it always
//! has; what it does comes back by its `id` as a `UiEvent`, and its
//! attributes (`value`, `checked`, `disabled`) and the widget follow each
//! other.

use bevy::prelude::*;

use super::attrs::Attrs;
use super::build::{Builder, Inside};
use super::check::{Checked, CheckedNode};
use super::parts::{Control, ControlOf};
use super::tag::Tag;
use crate::widgets::ButtonState;

mod bridge;
mod buttons;
mod choices;
mod event;
mod fields;
mod label;
mod sync;

pub use event::{UiEvent, UiValue};
pub(super) use sync::follow_elements;

pub(super) fn add(app: &mut App) {
    app.add_message::<UiEvent>()
        .add_observer(label::press_what_it_labels)
        // After `Update`, where the widgets say what was done to them.
        .add_systems(
            PostUpdate,
            bridge::report.before(super::build::build_documents),
        );
}

/// How a control's widget looks, from its element's attributes.
fn state_of(attrs: &Attrs) -> ButtonState {
    match (attrs.disabled, attrs.has("selected")) {
        (true, true) => ButtonState::DisabledSelected,
        (true, false) => ButtonState::Disabled,
        (false, true) => ButtonState::Selected,
        (false, false) => ButtonState::Normal,
    }
}

/// The words straight inside an element.
fn words_in(element: &Checked) -> String {
    let words = element.children.iter().filter_map(|node| match node {
        CheckedNode::Text(words) => Some(words.as_str()),
        CheckedNode::Element(_) => None,
    });
    words.collect::<Vec<_>>().join(" ")
}

impl Builder<'_, '_, '_> {
    /// Builds the widget that shows a control, inside its element.
    pub(super) fn control(&mut self, holder: Entity, element: &Checked, inside: Inside) {
        let attrs = &element.attrs;
        let size = inside.style.size;
        let node = self.widget_node(element, inside);
        let widget = match (element.tag, attrs.text("type")) {
            (Tag::Button, _) => self.button(holder, element, size, node),
            (Tag::Input, Some("checkbox")) => self.check(holder, attrs, size, node),
            (Tag::Input, Some("range")) => self.range(holder, attrs),
            (Tag::Input, _) => self.text_box(holder, attrs, size, node),
            (Tag::KeyBox, _) => self.key_box(holder, attrs, size, node),
            (Tag::Select, _) => self.select(holder, element, size, node),
            (Tag::ToggleRow | Tag::TabBar, _) => self.row(holder, element, size),
            (Tag::ColourPicker, _) => self.colour_picker(holder, element, size, node),
            _ => return,
        };
        // The widget carries the id, as widgets always have, so everything
        // that finds widgets by id (the debug API, scroll memory) still does.
        let mut made = self.commands.entity(widget);
        made.insert(ControlOf(holder));
        if let Some(id) = element.id {
            made.insert(inside.named(id));
        }
        self.commands.entity(holder).insert(Control(widget));
    }

    /// A widget's `Node`. It is as wide as its element, which is as wide as
    /// what holds it when stacked and as itself when in a row, so nothing
    /// needs a width to fill its room. It is as tall as the theme's row for
    /// the writing round it (the menus' is taller), or as its element when
    /// that is given a height. A button in a dialog's footer is at least the
    /// theme's width, so a footer's buttons match.
    fn widget_node(&self, element: &Checked, inside: Inside) -> Node {
        let attrs = &element.attrs;
        let is_box = element.tag == Tag::Input && attrs.text("type") == Some("text");
        let row = match is_box {
            true => self.theme.box_row(inside.style.size),
            false => self.theme.row(inside.style.size),
        };
        let in_footer = inside.footer && element.tag == Tag::Button;
        Node {
            width: percent(100),
            height: attrs.height.map_or(Val::Auto, |_| percent(100)),
            min_height: match attrs.height {
                Some(_) => Val::Auto,
                None => row.map_or(Val::Auto, px),
            },
            min_width: match in_footer {
                true => px(self.theme.footer_button),
                false => Val::Auto,
            },
            ..default()
        }
    }

    /// Spawns something inside `holder`, and says what.
    fn spawn_in(
        &mut self,
        holder: Entity,
        spawn: impl FnOnce(&mut ChildSpawnerCommands) -> Entity,
    ) -> Entity {
        let mut made = Entity::PLACEHOLDER;
        self.commands
            .entity(holder)
            .with_children(|inside| made = spawn(inside));
        made
    }
}

#[cfg(test)]
mod tests;
