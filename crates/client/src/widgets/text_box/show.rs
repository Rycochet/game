//! Keeps what boxes say on show: the prefix as the label's own text, a span per
//! character after it (so each can be found on the screen), an inline picture
//! per item in a box that takes them, and the suffix in grey.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::units::units;
use super::{RaisesKeyboard, TextBox, TextBoxChar, TextBoxLabel, TextBoxSuffix};
use crate::widgets::item_text::{InlineHeight, spawn_inline_item};
use crate::widgets::soft_keyboard;
use crate::widgets::theme::HINT_TEXT;

/// The characters a label shows, to know when to make its spans again.
#[derive(Component, Default)]
pub(in crate::widgets) struct Shown(String);

#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_text_boxes(
    mut commands: Commands,
    boxes: Query<&TextBox>,
    mut labels: Query<(
        Entity,
        &TextBoxLabel,
        &mut Text,
        &TextFont,
        &TextColor,
        &InlineHeight,
        Option<&Shown>,
    )>,
    mut suffixes: Query<(&TextBoxSuffix, &mut TextSpan)>,
) {
    for (label, text_box, mut text, font, colour, height, shown) in &mut labels {
        let Ok(owner) = boxes.get(text_box.0) else {
            continue;
        };
        if text.0 != owner.prefix {
            text.0 = owner.prefix.to_owned();
        }
        if shown.is_none_or(|shown| shown.0 != owner.text) {
            let (words, suffix) = (owner.text.clone(), owner.suffix.clone());
            let (accepts_items, of) = (owner.accepts_items, text_box.0);
            let (font, colour, height) = (font.clone(), *colour, height.0);
            // Its panel may be closed this very frame: a label that has gone
            // by the time this is done is left alone (`queue_silenced`).
            let again = move |mut label: EntityWorldMut| {
                label.despawn_related::<Children>();
                label.insert(Shown(words.clone()));
                let id = label.id();
                label.world_scope(|world| {
                    let mut commands = world.commands();
                    commands.entity(id).with_children(|spans| {
                        for (place, unit) in units(&words, accepts_items).iter().enumerate() {
                            match unit.item {
                                Some(id) => {
                                    spawn_inline_item(spans, id, height, TextBoxChar(place))
                                }
                                None => {
                                    let span = TextSpan(unit.c.to_string());
                                    spans.spawn((TextBoxChar(place), span, font.clone(), colour));
                                }
                            }
                        }
                        spans.spawn((
                            TextBoxSuffix(of),
                            TextSpan(suffix),
                            font.clone(),
                            TextColor(HINT_TEXT),
                        ));
                    });
                });
            };
            commands.entity(label).queue_silenced(again);
        }
    }
    for (suffix, mut span) in &mut suffixes {
        if let Ok(owner) = boxes.get(suffix.0)
            && span.0 != owner.suffix
        {
            span.0.clone_from(&owner.suffix);
        }
    }
}

/// Tells the browser where a tap should raise the on-screen keyboard.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn place_soft_keyboard(
    ui_scale: Res<UiScale>,
    places: Query<
        (&ComputedNode, &UiGlobalTransform, &InheritedVisibility),
        Or<(With<TextBox>, With<RaisesKeyboard>)>,
    >,
) {
    let places = places
        .iter()
        .filter(|(.., visible)| visible.get())
        .map(|(node, transform, _)| {
            // In the browser's pixels, the window's logical ones.
            let scale = node.inverse_scale_factor * ui_scale.0;
            Rect::from_center_size(transform.translation * scale, node.size() * scale)
        })
        .collect();
    soft_keyboard::set_boxes(places);
}
