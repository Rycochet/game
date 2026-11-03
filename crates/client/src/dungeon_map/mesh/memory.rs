//! Copying what is lit into the memory meshes, and the glow of what is hidden.

use bevy::prelude::*;

use super::super::walls::Layers;
use super::Builder;

/// What is hidden looks like: how much of each of red, green and blue is kept and
/// added, with its opacity unchanged.
pub(super) const HIDDEN_TINT: [(f32, f32); 4] = [(0.5, 0.05), (0.8, 0.3), (0.9, 0.4), (1.0, 0.0)];

/// How much each builder holds so far: its vertices and indices.
pub(super) fn sizes(builders: &[Builder]) -> Vec<(usize, usize)> {
    builders
        .iter()
        .map(|builder| (builder.positions.len(), builder.indices.len()))
        .collect()
}

/// Copies what was lit since `before` into the memory meshes, in grey, for the
/// part of it the mask says is out of sight. Tops stay out: the mask doesn't cut
/// them (`Builder::copy_grey`).
pub(super) fn copy_as_memory(layers: &mut Layers, before: &[(usize, usize)]) {
    for (i, &from) in before.iter().enumerate() {
        let Layers { solid, memory, .. } = &mut *layers;
        solid[i].copy_grey(from, &mut memory[i]);
    }
}

/// How many vertices each mesh holds so far.
pub(super) fn layer_sizes(layers: &Layers) -> Vec<usize> {
    all_builders(layers)
        .map(|builder| builder.positions.len())
        .collect()
}

pub(super) fn all_builders(layers: &Layers) -> impl Iterator<Item = &Builder> {
    layers
        .solid
        .iter()
        .chain(&layers.memory)
        .chain(std::iter::once(&layers.see_through))
}

/// Gives what was added since `before` the pale glow of something hidden, which
/// shows even in the dark.
pub(super) fn tint_since(layers: &mut Layers, before: &[usize]) {
    let Layers {
        solid,
        memory,
        see_through,
        ..
    } = layers;
    let builders = solid
        .iter_mut()
        .chain(memory.iter_mut())
        .chain(std::iter::once(see_through));
    for (builder, &from) in builders.zip(before) {
        for colour in &mut builder.colours[from..] {
            for (part, (keep, add)) in colour.iter_mut().zip(HIDDEN_TINT) {
                *part = *part * keep + add;
            }
        }
    }
}
