//! The controls made of `option`s: `select` (a dropdown), `toggle-row`,
//! `tab-bar` and `colour-picker`. Which option is chosen is the control's
//! `value`: the option's own `value`, or its words if it has none.

use bevy::prelude::*;

use super::{node_in, words_in};
use crate::dom::build::Builder;
use crate::dom::check::{Checked, CheckedNode};
use crate::dom::parts::Choices;
use crate::dom::pictures::picture_for;
use crate::dom::tag::Tag;
use crate::widgets::{
    BUTTON_PADDING, ColourPicker, Dropdown, Swatch, TextSize, spawn_colour_picker, spawn_dropdown,
    spawn_tabs, spawn_toggle_row,
};

/// A control's options, in order.
fn options(element: &Checked) -> impl Iterator<Item = &Checked> {
    element.children.iter().filter_map(|node| match node {
        CheckedNode::Element(option) => Some(option),
        CheckedNode::Text(_) => None,
    })
}

/// What choosing this option makes the control's `value`.
fn value_of(option: &Checked) -> String {
    let own = option.attrs.text("value").map(str::to_owned);
    own.unwrap_or_else(|| words_in(option))
}

/// Which option starts chosen: the one the control's `value` names, else
/// the first marked `selected`, else the first.
fn chosen(element: &Checked, values: &[String]) -> usize {
    let named = element.attrs.text("value");
    let named = named.and_then(|value| values.iter().position(|known| known == value));
    let marked = || options(element).position(|option| option.attrs.has("selected"));
    named.or_else(marked).unwrap_or(0)
}

impl Builder<'_, '_, '_> {
    /// Notes the options' values on the control, and says which is chosen.
    fn choices(&mut self, holder: Entity, element: &Checked) -> usize {
        let values: Vec<String> = options(element).map(value_of).collect();
        let selected = chosen(element, &values);
        self.commands.entity(holder).insert(Choices(values));
        selected
    }

    pub(super) fn select(&mut self, holder: Entity, element: &Checked, size: TextSize) -> Entity {
        let dropdown = Dropdown {
            options: options(element).map(words_in).collect(),
            selected: self.choices(holder, element),
            searchable: element.attrs.has("searchable"),
            greyed: options(element)
                .map(|option| option.attrs.disabled)
                .collect(),
            ..default()
        };
        let node = Node {
            padding: BUTTON_PADDING,
            ..node_in(&element.attrs, true)
        };
        self.spawn_in(holder, |inside| {
            spawn_dropdown(inside, dropdown, size, node).id()
        })
    }

    /// A toggle row or a tab bar.
    pub(super) fn row(&mut self, holder: Entity, element: &Checked, size: TextSize) -> Entity {
        let words: Vec<String> = options(element).map(words_in).collect();
        let labels: Vec<&str> = words.iter().map(String::as_str).collect();
        let selected = self.choices(holder, element);
        let tabs = element.tag == Tag::TabBar;
        self.spawn_in(holder, |inside| match tabs {
            true => spawn_tabs(inside, &labels, selected, size),
            false => spawn_toggle_row(inside, &labels, selected, size),
        })
    }

    /// Swatches: each option's `value` is a colour (`#be3c32`) or its `src`
    /// a picture, named by its `label` (or its words).
    pub(super) fn colour_picker(
        &mut self,
        holder: Entity,
        element: &Checked,
        size: TextSize,
    ) -> Entity {
        let swatch = |option: &Checked| {
            let picture = option.attrs.text("src");
            let picture = picture.and_then(|src| picture_for(src, self.pictures, self.assets));
            let colour = option
                .attrs
                .text("value")
                .and_then(|hex| Srgba::hex(hex).ok());
            match picture {
                Some((picture, _)) => Swatch::Picture(picture),
                None => Swatch::Colour(colour.map_or(Color::BLACK, Color::from)),
            }
        };
        let name = |option: &Checked| {
            let label = option.attrs.text("label").map(str::to_owned);
            label.unwrap_or_else(|| words_in(option))
        };
        let swatches: Vec<Swatch> = options(element).map(swatch).collect();
        let picker = ColourPicker {
            swatches,
            names: options(element).map(name).collect(),
            selected: self.choices(holder, element),
        };
        let node = node_in(&element.attrs, true);
        self.spawn_in(holder, |inside| {
            spawn_colour_picker(inside, picker, size, node).id()
        })
    }
}
