//! Where the places between a box's characters are on the screen, from how
//! Bevy laid its writing out: each character is a span of its own, so each has
//! a run of its own, as wide as it advances (spaces too); each item is an inline
//! box.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::text::{ComputedTextBlock, TextLayoutInfo};
use bevy::ui::UiGlobalTransform;

use super::{TextBoxChar, TextBoxLabel};

/// What one run of the label is: part of the prefix, a character, or neither
/// (the suffix); and from and to where across, in the layout's own pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Run {
    Prefix(f32, f32),
    Char(usize, f32, f32),
    Other,
}

/// Where each of the `count + 1` places is, from the left of the writing, in
/// the layout's pixels. A character with no run of its own (not laid out yet)
/// takes no room.
pub(super) fn edges_of(runs: impl IntoIterator<Item = Run>, count: usize) -> Vec<f32> {
    let mut prefix_end = 0.0_f32;
    let mut spans: Vec<Option<(f32, f32)>> = vec![None; count];
    for run in runs {
        match run {
            Run::Prefix(_, to) => prefix_end = prefix_end.max(to),
            Run::Char(place, from, to) if place < count => {
                let span = spans[place].get_or_insert((from, to));
                *span = (span.0.min(from), span.1.max(to));
            }
            _ => {}
        }
    }
    let mut edges = Vec::with_capacity(count + 1);
    edges.push(spans.first().copied().flatten().map_or(prefix_end, |s| s.0));
    for span in &spans {
        let last = *edges.last().expect("one already");
        edges.push(span.map_or(last, |s| s.1.max(last)));
    }
    edges
}

/// The place nearest to `x`, among `edges`.
pub(super) fn nearest(edges: &[f32], x: f32) -> usize {
    edges
        .iter()
        .enumerate()
        .min_by(|a, b| (a.1 - x).abs().total_cmp(&(b.1 - x).abs()))
        .map_or(0, |(place, _)| place)
}

/// A label's places in interface pixels from its left edge, for `count`
/// characters.
pub(super) fn edges(
    info: &TextLayoutInfo,
    block: &ComputedTextBlock,
    node: &ComputedNode,
    chars: &Query<&TextBoxChar>,
    count: usize,
) -> Vec<f32> {
    let entities = block.entities();
    let runs = info.run_geometry.iter().map(|run| {
        let (from, to) = (run.bounds.min.x, run.bounds.max.x);
        match entities.get(run.section_index as usize) {
            Some(_) if run.section_index == 0 => Run::Prefix(from, to),
            Some(section) => chars
                .get(section.entity)
                .map_or(Run::Other, |c| Run::Char(c.0, from, to)),
            None => Run::Other,
        }
    });
    // Items are boxes in the layout, not runs of writing.
    let boxes = info.inline_boxes.iter().map(|(entity, _, rect)| {
        chars
            .get(*entity)
            .map_or(Run::Other, |c| Run::Char(c.0, rect.min.x, rect.max.x))
    });
    let runs = runs.chain(boxes);
    let scale = node.inverse_scale_factor;
    edges_of(runs, count)
        .into_iter()
        .map(|x| x * scale)
        .collect()
}

/// The boxes' labels as last laid out, to find places in them.
#[derive(SystemParam)]
pub(in crate::widgets) struct Labels<'w, 's> {
    labels: Query<
        'w,
        's,
        (
            &'static TextBoxLabel,
            &'static TextLayoutInfo,
            &'static ComputedTextBlock,
            &'static ComputedNode,
            &'static UiGlobalTransform,
        ),
    >,
    chars: Query<'w, 's, &'static TextBoxChar>,
}

impl Labels<'_, '_> {
    /// Where a box's label starts across the screen and where its `count + 1`
    /// places are from there, in interface pixels.
    pub fn places(&self, text_box: Entity, count: usize) -> Option<(f32, Vec<f32>)> {
        let (_, info, block, node, transform) =
            self.labels.iter().find(|(label, ..)| label.0 == text_box)?;
        Some((
            left_of(node, transform),
            edges(info, block, node, &self.chars, count),
        ))
    }

    /// The place in a box nearest to `x` across the screen.
    pub fn place_at(&self, text_box: Entity, count: usize, x: f32) -> usize {
        self.places(text_box, count)
            .map_or(count, |(left, edges)| nearest(&edges, x - left))
    }
}

/// Where a node's left edge is, in interface pixels.
pub(super) fn left_of(node: &ComputedNode, transform: &UiGlobalTransform) -> f32 {
    (transform.translation.x - node.size().x / 2.0) * node.inverse_scale_factor
}

#[cfg(test)]
mod tests;
