//! From a control's attributes to its widget, whenever code changes them
//! (`Dom::set`): `disabled`, `selected`, `checked` and `value`.

use bevy::prelude::*;

use super::fields::{key_box_of, share_of, text_box_of};
use super::state_of;
use crate::dom::parts::{Choices, Control, Element};
use crate::dom::tag::Tag;
use crate::widgets::{
    ButtonState, Checked, ChooseTab, ColourPicker, Dropdown, KeyBox, Slider, Switch, TextBox,
};

/// The parts of widgets that attributes change.
#[derive(bevy::ecs::system::SystemParam)]
pub(in crate::dom) struct Shown<'w, 's> {
    states: Query<'w, 's, &'static mut ButtonState>,
    checks: Query<'w, 's, &'static mut Checked>,
    switches: Query<'w, 's, &'static mut Switch>,
    boxes: Query<'w, 's, &'static mut TextBox>,
    sliders: Query<'w, 's, &'static mut Slider>,
    dropdowns: Query<'w, 's, &'static mut Dropdown>,
    key_boxes: Query<'w, 's, &'static mut KeyBox>,
    pickers: Query<'w, 's, &'static mut ColourPicker>,
    tabs: MessageWriter<'w, ChooseTab>,
}

/// Sets `shown` to `wanted` only if it differs, so nothing is told a widget
/// changed when it didn't.
fn put<T: PartialEq>(shown: Result<Mut<T>, impl Sized>, wanted: T) {
    if let Ok(mut shown) = shown
        && *shown != wanted
    {
        *shown = wanted;
    }
}

/// Makes each control's widget as its element's attributes now say.
pub(in crate::dom) fn follow_elements(
    changed: Query<(&Element, &Control, Option<&Choices>), Changed<Element>>,
    mut shown: Shown,
) {
    for (element, Control(widget), choices) in &changed {
        let (attrs, widget) = (&element.attrs, *widget);
        put(shown.states.get_mut(widget), state_of(attrs));
        let on = attrs.has("checked");
        put(shown.checks.get_mut(widget), Checked(on));
        put(shown.switches.get_mut(widget), Switch(on));
        if let Ok(mut slider) = shown.sliders.get_mut(widget)
            && slider.0 != share_of(attrs)
        {
            slider.0 = share_of(attrs);
        }
        if let Ok(mut text_box) = shown.boxes.get_mut(widget) {
            let wanted = text_box_of(attrs);
            // Only what differs: changing a box's words moves its caret.
            if text_box.text != wanted.text {
                text_box.text = wanted.text;
            }
            if text_box.error != wanted.error {
                text_box.error = wanted.error;
            }
        }
        put(shown.key_boxes.get_mut(widget), key_box_of(attrs));
        // Which option the `value` names, among those there are.
        let values = choices
            .map(|choices| choices.0.as_slice())
            .unwrap_or_default();
        let named = attrs.text("value");
        let Some(index) = named.and_then(|value| values.iter().position(|known| known == value))
        else {
            continue;
        };
        if let Ok(mut dropdown) = shown.dropdowns.get_mut(widget)
            && dropdown.selected != index
        {
            dropdown.selected = index;
        }
        if let Ok(mut picker) = shown.pickers.get_mut(widget)
            && picker.selected != index
        {
            picker.selected = index;
        }
        if matches!(element.tag, Tag::ToggleRow | Tag::TabBar) {
            shown.tabs.write(ChooseTab(widget, index));
        }
    }
}
