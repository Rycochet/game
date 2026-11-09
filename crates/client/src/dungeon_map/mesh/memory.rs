//! Copying what is lit into the memory meshes.

use bevy::prelude::*;

use super::super::walls::Layers;
use super::Builder;

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
