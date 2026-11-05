//! `client.ui.set`, `client.ui.press` and `client.ui.close`: changing widgets as
//! a player would, by the messages their own systems send.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params, refused};

use super::super::slider::Slider;
use super::super::text_box::allowed;
use super::super::{ButtonPressed, Dropdown, DropdownChosen, SliderMoved, TextBox, TextEdited};
use super::describe::IdAsked;
use super::{Kind, Widgets};

#[derive(Deserialize)]
struct SetAsked {
    id: String,
    value: Value,
}

/// What a set comes to, worked out before anything changes.
enum Change {
    Text(String),
    Choose(usize),
    Slide(f32),
    Press(Entity),
    Nothing,
    Scroll(f32),
}

/// `client.ui.set {id, value}`: a text box's words, a dropdown's or tab row's
/// option (its index or words), a slider's place (0 to 1), whether a check
/// button is ticked, or how far a scroll area is scrolled (interface pixels).
#[allow(clippy::type_complexity)]
pub(super) fn set(
    In(request): In<Option<Value>>,
    mut all: ParamSet<(
        Widgets,
        Query<&mut TextBox>,
        Query<&mut Dropdown>,
        Query<&mut Slider>,
        Query<&mut ScrollPosition>,
    )>,
    mut pressed: MessageWriter<ButtonPressed>,
    mut edited: MessageWriter<TextEdited>,
    mut chosen: MessageWriter<DropdownChosen>,
    mut moved: MessageWriter<SliderMoved>,
) -> BrpResult {
    let asked: SetAsked = params(request)?;
    let (entity, change) = {
        let widgets = all.p0();
        let entity = widgets.find(&asked.id)?;
        (entity, change_of(&widgets, entity, &asked)?)
    };
    match change {
        Change::Text(text) => {
            if let Ok(mut text_box) = all.p1().get_mut(entity) {
                text_box.text = allowed(&text, text_box.digits_only);
                edited.write(TextEdited(entity));
            }
        }
        Change::Choose(at) => {
            if let Ok(mut dropdown) = all.p2().get_mut(entity) {
                dropdown.selected = at;
                chosen.write(DropdownChosen(entity, at));
            }
        }
        Change::Slide(value) => {
            if let Ok(mut slider) = all.p3().get_mut(entity) {
                slider.0 = value;
                moved.write(SliderMoved(entity, value));
            }
        }
        Change::Scroll(y) => {
            if let Ok(mut position) = all.p4().get_mut(entity) {
                position.y = y;
            }
        }
        Change::Press(button) => {
            pressed.write(ButtonPressed(button));
        }
        Change::Nothing => {}
    }
    Ok(json!("ok"))
}

fn change_of(widgets: &Widgets, entity: Entity, asked: &SetAsked) -> BrpResult<Change> {
    if widgets.disabled(entity) {
        return Err(refused(format!("{} is disabled", asked.id)));
    }
    let value = &asked.value;
    match widgets.kind(entity) {
        Kind::TextBox => {
            let text = match value {
                Value::String(words) => words.clone(),
                Value::Number(number) => number.to_string(),
                _ => return Err(refused("a text box takes words or a number")),
            };
            Ok(Change::Text(text))
        }
        Kind::Dropdown => {
            let dropdown = widgets.dropdowns.get(entity).map_err(|_| refused("gone"))?;
            let at = option_at(&dropdown.options, value)?;
            if dropdown.is_greyed(at) {
                return Err(refused(format!("{} is greyed", dropdown.options[at])));
            }
            Ok(Change::Choose(at))
        }
        Kind::Slider => {
            let to = value
                .as_f64()
                .ok_or_else(|| refused("a slider takes 0 to 1"))?;
            Ok(Change::Slide((to as f32).clamp(0.0, 1.0)))
        }
        Kind::Tabs => {
            let tabs = widgets.tabs_of(entity);
            let labels: Vec<String> = tabs.iter().map(|(_, words)| words.clone()).collect();
            let at = option_at(&labels, value)?;
            Ok(Change::Press(tabs[at].0))
        }
        Kind::Check => {
            let ticked = value
                .as_bool()
                .ok_or_else(|| refused("a check takes true or false"))?;
            let was = widgets.checked.get(entity).is_ok_and(|checked| checked.0);
            Ok(if was == ticked {
                Change::Nothing
            } else {
                Change::Press(entity)
            })
        }
        Kind::Scroll => {
            let to = value
                .as_f64()
                .ok_or_else(|| refused("a scroll area takes pixels"))?;
            Ok(Change::Scroll(to.max(0.0) as f32))
        }
        Kind::Panel | Kind::Button | Kind::Other => Err(refused(format!(
            "{} has no value: use client.ui.press or client.ui.close",
            asked.id
        ))),
    }
}

/// Which option this is: an index, or the option's words in any case.
fn option_at(options: &[String], value: &Value) -> BrpResult<usize> {
    let at = match value {
        Value::Number(number) => number.as_u64().map(|at| at as usize),
        Value::String(words) => options.iter().position(|o| o.eq_ignore_ascii_case(words)),
        _ => None,
    };
    at.filter(|&at| at < options.len())
        .ok_or_else(|| refused(format!("options are {options:?}, or their index")))
}

/// `client.ui.press {id}`: presses a button (or a dropdown, opening its list).
pub(super) fn press(
    In(request): In<Option<Value>>,
    widgets: Widgets,
    mut pressed: MessageWriter<ButtonPressed>,
) -> BrpResult {
    let asked: IdAsked = params(request)?;
    let entity = widgets.find(&asked.id)?;
    if widgets.disabled(entity) {
        return Err(refused(format!("{} is disabled", asked.id)));
    }
    if !widgets.pressable.contains(entity) {
        return Err(refused(format!("{} can't be pressed", asked.id)));
    }
    pressed.write(ButtonPressed(entity));
    Ok(json!("ok"))
}

/// `client.ui.close {id}`: closes a moveable panel, as its close button does.
pub(super) fn close(
    In(request): In<Option<Value>>,
    widgets: Widgets,
    mut commands: Commands,
) -> BrpResult {
    let asked: IdAsked = params(request)?;
    let entity = widgets.find(&asked.id)?;
    if widgets.kind(entity) != Kind::Panel {
        return Err(refused(format!("{} isn't a moveable panel", asked.id)));
    }
    commands.entity(entity).despawn();
    Ok(json!("ok"))
}
