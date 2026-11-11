//! Labels, as in HTML: a click on a label is a click on what it is for,
//! whether the label is round it or names it with `for`.

use bevy::prelude::*;

use crate::dom::input::Click;
use crate::dom::parts::{Control, ControlOf, Element, LabelFor};
use crate::dom::tag::Tag;
use crate::widgets::{ButtonPressed, ButtonState, TextBox, TextFocus, UiId};

/// Presses the control a clicked label is for (a text box gets the keyboard
/// instead). Clicks pass up through what holds them, so this sees one on
/// anything in a label.
#[allow(clippy::too_many_arguments)]
pub(super) fn press_what_it_labels(
    click: On<Click>,
    labels: Query<(&Element, Option<&LabelFor>)>,
    children: Query<&Children>,
    controls: Query<&Control>,
    named: Query<(Entity, &UiId), With<ControlOf>>,
    parents: Query<&ChildOf>,
    states: Query<&ButtonState>,
    boxes: Query<(), With<TextBox>>,
    mut pressed: MessageWriter<ButtonPressed>,
    mut focus: ResMut<TextFocus>,
) {
    let label = click.entity;
    let Ok((element, wanted)) = labels.get(label) else {
        return;
    };
    if element.tag != Tag::Label {
        return;
    }
    let by_name = |LabelFor(wanted): &LabelFor| {
        let found = named.iter().find(|(_, id)| *id == wanted);
        found.map(|(widget, _)| widget)
    };
    let inside = || {
        let mut held = children.iter_descendants(label);
        held.find_map(|entity| controls.get(entity).ok().map(|control| control.0))
    };
    let Some(widget) = wanted.and_then(by_name).or_else(inside) else {
        return;
    };
    // A click on the control itself has already done its work.
    let from = click.original_event_target();
    let on_widget = from == widget || parents.iter_ancestors(from).any(|held| held == widget);
    let disabled = states.get(widget).is_ok_and(|state| state.is_disabled());
    if on_widget || disabled {
        return;
    }
    if boxes.contains(widget) {
        focus.0 = Some(widget);
    } else {
        pressed.write(ButtonPressed(widget));
    }
}
