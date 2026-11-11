//! From the widgets' own messages to `UiEvent`s by id, keeping each
//! element's attributes as its widget now is.

use std::borrow::Cow;

use bevy::prelude::*;

use super::event::{UiEvent, UiValue};
use super::fields::value_at;
use crate::dom::parts::{Choices, ControlOf, Element};
use crate::dom::tag::Tag;
use crate::widgets::{
    ButtonPressed, Checked, ColourChosen, DropdownChosen, KeyChosen, SliderMoved, SwitchFlipped,
    TabChosen, TextBox, TextEdited, UiId,
};

/// What the widgets said this frame.
#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct Said<'w, 's> {
    pressed: MessageReader<'w, 's, ButtonPressed>,
    flipped: MessageReader<'w, 's, SwitchFlipped>,
    moved: MessageReader<'w, 's, SliderMoved>,
    edited: MessageReader<'w, 's, TextEdited>,
    dropdowns: MessageReader<'w, 's, DropdownChosen>,
    tabs: MessageReader<'w, 's, TabChosen>,
    colours: MessageReader<'w, 's, ColourChosen>,
    keys: MessageReader<'w, 's, KeyChosen>,
}

/// One thing a widget said, by the kind of thing.
enum Did {
    Pressed,
    Flipped(bool),
    Moved(f32),
    Edited,
    Chose(usize),
    Key(Option<crate::common::Binding>),
}

impl Said<'_, '_> {
    /// Everything said, with the widget that said it.
    fn all(&mut self) -> Vec<(Entity, Did)> {
        let mut all: Vec<(Entity, Did)> = Vec::new();
        all.extend(self.pressed.read().map(|said| (said.0, Did::Pressed)));
        all.extend(
            self.flipped
                .read()
                .map(|said| (said.0, Did::Flipped(said.1))),
        );
        all.extend(self.moved.read().map(|said| (said.0, Did::Moved(said.1))));
        all.extend(self.edited.read().map(|said| (said.0, Did::Edited)));
        all.extend(
            self.dropdowns
                .read()
                .map(|said| (said.0, Did::Chose(said.1))),
        );
        all.extend(self.tabs.read().map(|said| (said.0, Did::Chose(said.1))));
        all.extend(self.colours.read().map(|said| (said.0, Did::Chose(said.1))));
        all.extend(self.keys.read().map(|said| (said.0, Did::Key(said.1))));
        all
    }
}

/// Tells what was done to each control, by its id. The element's attributes
/// are changed quietly (nothing is told they changed), as the widget is
/// already showing them.
pub(super) fn report(
    mut said: Said,
    widgets: Query<(&UiId, &ControlOf)>,
    mut elements: Query<(&mut Element, Option<&Choices>)>,
    mut checks: Query<&mut Checked>,
    boxes: Query<&TextBox>,
    mut events: MessageWriter<UiEvent>,
) {
    for (widget, did) in said.all() {
        let Ok((id, ControlOf(holder))) = widgets.get(widget) else {
            continue;
        };
        let Ok((mut element, choices)) = elements.get_mut(*holder) else {
            continue;
        };
        let tag = element.tag;
        let attrs = &mut element.bypass_change_detection().attrs;
        let value = match did {
            Did::Pressed if tag == Tag::Button => UiValue::Pressed,
            Did::Pressed if tag == Tag::Dialog => UiValue::Dismissed,
            // A checkbox is a button that ticks itself.
            Did::Pressed => {
                let Ok(mut checked) = checks.get_mut(widget) else {
                    continue;
                };
                checked.0 = !checked.0;
                attrs.set_flag("checked", checked.0);
                UiValue::Flag(checked.0)
            }
            Did::Flipped(on) => {
                attrs.set_flag("checked", on);
                UiValue::Flag(on)
            }
            Did::Moved(share) => {
                let value = value_at(attrs, share);
                attrs.put("value", Cow::Owned(value.to_string()));
                UiValue::Number(value)
            }
            Did::Edited => {
                let Ok(text_box) = boxes.get(widget) else {
                    continue;
                };
                attrs.put("value", Cow::Owned(text_box.text.clone()));
                UiValue::Text(text_box.text.clone())
            }
            Did::Chose(index) => {
                let values = choices
                    .map(|choices| choices.0.as_slice())
                    .unwrap_or_default();
                let value = values.get(index).cloned().unwrap_or_default();
                attrs.put("value", Cow::Owned(value.clone()));
                UiValue::Choice { index, value }
            }
            Did::Key(binding) => {
                let name = binding.map(|key| key.to_name()).unwrap_or_default();
                attrs.put("value", Cow::Owned(name));
                UiValue::Key(binding)
            }
        };
        events.write(UiEvent { id: *id, value });
    }
}
