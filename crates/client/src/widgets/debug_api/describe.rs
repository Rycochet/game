//! `client.ui.list` and `client.ui.get`: what widgets there are and what they hold.

use bevy::prelude::*;
use serde::Deserialize;
use server::debug_api::{BrpResult, Value, json, params};

use super::super::{duplicate_ids, logical_rect};
use super::{Kind, Widgets};

#[derive(Deserialize)]
struct ListAsked {
    /// Only this form: its name, or with which one (`bag#<id>`).
    form: Option<String>,
    /// Hidden ones too.
    #[serde(default)]
    all: bool,
}

#[derive(Deserialize)]
pub(super) struct IdAsked {
    pub(super) id: String,
}

/// `client.ui.list {form?, all?}`: every widget on show (or all, with `all`),
/// sorted by id, and any id more than one on show has.
pub(super) fn list(In(request): In<Option<Value>>, widgets: Widgets) -> BrpResult {
    let asked: ListAsked = params(request)?;
    let mut found: Vec<(String, Value)> = widgets
        .ids
        .iter()
        .filter(|(_, id, ..)| asked.form.as_ref().is_none_or(|form| id.in_form(form)))
        .filter(|(entity, ..)| asked.all || widgets.shown(*entity))
        .map(|(entity, id, ..)| (id.to_string(), describe(&widgets, entity)))
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    let shown = widgets
        .ids
        .iter()
        .filter(|(entity, ..)| widgets.shown(*entity))
        .map(|(_, id, ..)| id);
    let duplicates = duplicate_ids(shown);
    Ok(json!({
        "duplicates": duplicates,
        "widgets": found.iter().map(|(_, about)| about).collect::<Vec<_>>(),
    }))
}

/// `client.ui.get {id}`: one widget on show.
pub(super) fn get(In(request): In<Option<Value>>, widgets: Widgets) -> BrpResult {
    let asked: IdAsked = params(request)?;
    let entity = widgets.find(&asked.id)?;
    Ok(describe(&widgets, entity))
}

/// A widget as JSON: its id, kind, words, state, value and where it is, in the
/// window's logical pixels as `client.click` takes them.
fn describe(widgets: &Widgets, entity: Entity) -> Value {
    let Ok((_, id, node, transform, _)) = widgets.ids.get(entity) else {
        return Value::Null;
    };
    let kind = widgets.kind(entity);
    let (corner, size) = logical_rect(node, transform);
    let scale = widgets.scale.0;
    let centre = (corner + size / 2.0) * scale;
    let mut about = json!({
        "id": id.to_string(),
        "kind": kind.name(),
        "shown": widgets.shown(entity),
        "disabled": widgets.disabled(entity),
        "covered": widgets.covered(entity),
        "at": [centre.x.round(), centre.y.round()],
        "size": [(size.x * scale).round(), (size.y * scale).round()],
    });
    if let Some(words) = widgets.label(entity) {
        about["label"] = json!(words);
    }
    if let Ok(state) = widgets.states.get(entity) {
        about["state"] = json!(format!("{state:?}"));
    }
    add_value(widgets, entity, kind, &mut about);
    about
}

/// What it holds, by kind.
fn add_value(widgets: &Widgets, entity: Entity, kind: Kind, about: &mut Value) {
    match kind {
        Kind::TextBox => {
            if let Ok(text_box) = widgets.text_boxes.get(entity) {
                about["value"] = json!(text_box.text);
                about["error"] = json!(text_box.error);
                about["focused"] = json!(widgets.focus.0 == Some(entity));
                if let Ok(cursor) = widgets.text_cursors.get(entity) {
                    // In characters; kept while the box hasn't the keyboard.
                    let len = text_box.text.chars().count();
                    let (caret, anchor) = (cursor.caret.min(len), cursor.anchor.min(len));
                    about["caret"] = json!(caret);
                    about["selection"] = if caret == anchor {
                        Value::Null
                    } else {
                        json!([caret.min(anchor), caret.max(anchor)])
                    };
                }
            }
        }
        Kind::KeyBox => {
            if let Ok(key_box) = widgets.key_boxes.get(entity) {
                about["value"] = json!(key_box.binding.map(|binding| binding.to_name()));
                about["shows"] = json!(key_box.binding.map(|binding| binding.label()));
                about["warned"] = json!(key_box.warned);
                about["waiting"] = json!(widgets.waiting.entity == Some(entity));
            }
        }
        Kind::Dropdown => {
            if let Ok(dropdown) = widgets.dropdowns.get(entity) {
                let greyed: Vec<usize> = (0..dropdown.options.len())
                    .filter(|&at| dropdown.is_greyed(at))
                    .collect();
                about["value"] = json!(dropdown.options.get(dropdown.selected));
                about["index"] = json!(dropdown.selected);
                about["options"] = json!(dropdown.options);
                about["greyed"] = json!(greyed);
            }
        }
        Kind::ColourPicker => {
            if let Ok(picker) = widgets.pickers.get(entity) {
                about["value"] = json!(picker.names.get(picker.selected));
                about["index"] = json!(picker.selected);
                about["options"] = json!(picker.names);
            }
        }
        Kind::Slider => {
            if let Ok(slider) = widgets.sliders.get(entity) {
                about["value"] = json!(slider.0);
            }
        }
        Kind::Tabs => {
            if let Ok(row) = widgets.rows.get(entity) {
                let labels: Vec<String> = widgets
                    .tabs_of(entity)
                    .into_iter()
                    .map(|(_, words)| words)
                    .collect();
                about["value"] = json!(labels.get(row.selected));
                about["index"] = json!(row.selected);
                about["options"] = json!(labels);
            }
        }
        Kind::Check => {
            if let Ok(checked) = widgets.checked.get(entity) {
                about["value"] = json!(checked.0);
            }
        }
        Kind::Switch => {
            if let Ok(switch) = widgets.switches.get(entity) {
                about["value"] = json!(switch.0);
            }
        }
        Kind::Scroll => {
            if let Ok(position) = widgets.scrolls.get(entity) {
                about["value"] = json!(position.y);
            }
        }
        Kind::Panel | Kind::Button | Kind::Other => {}
    }
}
