//! How key boxes look: their binding or "Press key", and their edge.

use bevy::prelude::*;

use super::super::feedback::Pointed;
use super::super::theme::{
    DISABLED_BACKGROUND, DISABLED_EDGE, DISABLED_TEXT, ERROR_EDGE, HINT_TEXT, HOVER_EDGE,
    SUNKEN_BACKGROUND,
};
use super::super::{ButtonState, INK, sunken_border};
use super::{KeyBox, KeyBoxLabel, KeyWaiting};

const WAITING: &str = "Press key";

/// What a box says: its binding, or while waiting what is held so far.
pub(super) fn words(key_box: &KeyBox, waiting: Option<&KeyWaiting>) -> String {
    match waiting {
        Some(waiting) if waiting.held.is_empty() => WAITING.into(),
        Some(waiting) => format!("{} + …", waiting.held.labels().join(" + ")),
        None => key_box
            .binding
            .map(|binding| binding.label())
            .unwrap_or_default(),
    }
}

/// After `Update`, before layout, so a box spawned then is right at once.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_key_boxes(
    waiting: Res<KeyWaiting>,
    pointed: Res<Pointed>,
    mut boxes: Query<(
        Entity,
        &KeyBox,
        &ButtonState,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
    mut labels: Query<(&KeyBoxLabel, &mut Text, &mut TextColor)>,
) {
    for (entity, key_box, state, mut background, mut border) in &mut boxes {
        let is_waiting = waiting.entity == Some(entity);
        let (colour, edge) = if state.is_disabled() {
            (DISABLED_BACKGROUND, BorderColor::from(DISABLED_EDGE))
        } else if key_box.warned {
            (SUNKEN_BACKGROUND, BorderColor::from(ERROR_EDGE))
        } else if is_waiting {
            (SUNKEN_BACKGROUND, BorderColor::from(INK))
        } else if pointed.hovered == Some(entity) {
            (SUNKEN_BACKGROUND, BorderColor::from(HOVER_EDGE))
        } else {
            (SUNKEN_BACKGROUND, sunken_border())
        };
        background.set_if_neq(BackgroundColor(colour));
        border.set_if_neq(edge);
    }
    for (label, mut text, mut text_colour) in &mut labels {
        let Ok((entity, key_box, state, ..)) = boxes.get(label.0) else {
            continue;
        };
        let is_waiting = waiting.entity == Some(entity);
        let wanted = words(key_box, is_waiting.then_some(&*waiting));
        if text.0 != wanted {
            text.0 = wanted;
        }
        let colour = if state.is_disabled() {
            DISABLED_TEXT
        } else if is_waiting {
            HINT_TEXT
        } else {
            INK
        };
        text_colour.set_if_neq(TextColor(colour));
    }
}
