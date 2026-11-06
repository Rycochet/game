//! The band behind what is selected: one piece from the selection's start to
//! its end (rather than a background per character, which leaves seams), paler
//! in a box without the keyboard, which keeps its selection.

use bevy::prelude::*;

use super::keys::current;
use super::layout::Labels;
use super::{BORDER, Cursor, SIDE_PADDING, TextBox, TextFocus};
use crate::widgets::theme::{SELECTION, SELECTION_WITHOUT_KEYBOARD};

/// The band, in the box it holds. Spawned before the writing, so behind it.
#[derive(Component)]
pub(in crate::widgets) struct TextBoxSelection(pub Entity);

/// The band's colour.
pub(super) fn band_colour(keyboard: bool) -> Color {
    if keyboard {
        SELECTION
    } else {
        SELECTION_WITHOUT_KEYBOARD
    }
}

/// Places each box's band. After `show_carets`, which slides the writing.
#[allow(clippy::type_complexity)]
pub(in crate::widgets) fn show_selections(
    focus: Res<TextFocus>,
    boxes: Query<(&TextBox, &Cursor, &ComputedNode)>,
    labels: Labels,
    mut bands: Query<(
        &TextBoxSelection,
        &mut Node,
        &mut BackgroundColor,
        &mut Visibility,
        &ComputedNode,
    )>,
) {
    for (band, mut node, mut colour, mut visibility, band_node) in &mut bands {
        let Ok((text_box, cursor, box_node)) = boxes.get(band.0) else {
            continue;
        };
        let edited = current(text_box, cursor);
        let places = labels.places(band.0, edited.len());
        let (Some((from, to)), Some((_, edges))) = (edited.selection(), places) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let (start, end) = (
            edges[from.min(edges.len() - 1)],
            edges[to.min(edges.len() - 1)],
        );
        let inside = box_node.size().y * box_node.inverse_scale_factor - BORDER * 2.0;
        let height = band_node.size().y * band_node.inverse_scale_factor;
        let place = (
            px(SIDE_PADDING + start - cursor.scroll),
            px((inside - height) / 2.0),
            px(end - start),
        );
        if (node.left, node.top, node.width) != place {
            (node.left, node.top, node.width) = place;
        }
        colour.set_if_neq(BackgroundColor(band_colour(focus.0 == Some(band.0))));
        visibility.set_if_neq(Visibility::Inherited);
    }
}

#[cfg(test)]
mod tests;
