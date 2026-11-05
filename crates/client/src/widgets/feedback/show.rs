//! Showing buttons' looks, text boxes' and sliders' edges, and tooltips.

use bevy::prelude::*;

use super::super::item::{ItemTip, ItemTooltip};
use super::super::slider::Slider;
use super::super::text_box::{TextBox, TextFocus};
use super::super::theme::{ERROR_EDGE, HOVER_EDGE};
use super::super::{ButtonState, INK, Switch, sunken_border};
use super::{ButtonStyle, ButtonTooltip, Pointed, TOOLTIP_DELAY, TOOLTIP_OFFSET, Touch, look};
use crate::common::UiPointer;

/// Makes buttons look as their state and the pointer say. After `Update`, so a
/// button spawned then is right from the first frame it is drawn.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_button_looks(
    pointed: Res<Pointed>,
    // A switch's track and thumb show its state instead (`switch`).
    mut buttons: Query<
        (
            Entity,
            &ButtonState,
            Option<&ButtonStyle>,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        Without<Switch>,
    >,
    mut labels: Query<&mut TextColor>,
) {
    for (entity, state, style, mut background, mut border, children) in &mut buttons {
        let touch = if pointed.pressed == Some(entity) && pointed.hovered == Some(entity) {
            Touch::Pressed
        } else if pointed.hovered == Some(entity) {
            Touch::Hovered
        } else {
            Touch::Away
        };
        let style = style.copied().unwrap_or_default();
        let (new_background, new_border, new_text) = look(*state, style, touch);
        background.set_if_neq(BackgroundColor(new_background));
        border.set_if_neq(new_border);
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.set_if_neq(TextColor(new_text));
            }
        }
    }
}

/// The edge of a text box: what is wrong, then who has it, then the pointer.
pub(super) fn text_box_edge(error: bool, focused: bool, hovered: bool) -> BorderColor {
    if error {
        BorderColor::from(ERROR_EDGE)
    } else if focused {
        BorderColor::from(INK)
    } else if hovered {
        BorderColor::from(HOVER_EDGE)
    } else {
        sunken_border()
    }
}

/// Text boxes and sliders have no state of their own to show: their edges change.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_edges(
    pointed: Res<Pointed>,
    focus: Res<TextFocus>,
    mut boxes: Query<(Entity, &TextBox, &mut BorderColor)>,
    mut sliders: Query<(Entity, &mut BorderColor), (With<Slider>, Without<TextBox>)>,
) {
    for (entity, text_box, mut border) in &mut boxes {
        border.set_if_neq(text_box_edge(
            text_box.error,
            focus.0 == Some(entity),
            pointed.hovered == Some(entity),
        ));
    }
    for (entity, mut border) in &mut sliders {
        let colour = if pointed.hovered == Some(entity) {
            HOVER_EDGE
        } else {
            INK
        };
        border.set_if_neq(BorderColor::from(colour));
    }
}

/// Asks for the name of the button the pointer has stayed on, if it has one.
pub(in crate::widgets) fn show_button_tooltips(
    time: Res<Time<Real>>,
    pointer: Res<UiPointer>,
    pointed: Res<Pointed>,
    names: Query<&ButtonTooltip>,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let (Some(at), Some(hovered), None) = (pointer.0, pointed.hovered, pointed.pressed) else {
        return;
    };
    if time.elapsed_secs() - pointed.since < TOOLTIP_DELAY {
        return;
    }
    if let Ok(name) = names.get(hovered) {
        tooltip.0 = Some(ItemTip {
            words: name.0.clone(),
            quality: None,
            set: false,
            at: at + TOOLTIP_OFFSET,
        });
    }
}
