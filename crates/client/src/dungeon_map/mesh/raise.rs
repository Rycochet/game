//! Raised wilderness cells: lifting what a cell draws onto its ground's
//! height, and the upright faces down from a higher cell to a lower one beside
//! it (the higher cell's, in the land's colour, darker, with a black line
//! along its top edge).

use bevy::prelude::*;
use shared::dungeon::Cell;

use super::super::CELL_SIZE;
use super::super::walls::{Layers, Scene};
use super::{MEMORY_SHADE, Surface, with_ambient};

/// How bright an edge's face is against the ground on top: darker towards
/// its foot.
const EDGE_TOP_SHADE: f32 = 0.7;
const EDGE_FOOT_SHADE: f32 = 0.45;

/// The black line along an edge's top: how tall, how dark and how far in front
/// of the face.
const LINE_HEIGHT: f32 = CELL_SIZE * 0.04;
const LINE_SHADE: f32 = 0.03;
const LINE_OUT: f32 = 0.05;

/// The smallest drop worth a face.
const LEAST_DROP: f32 = 0.01;

/// How much every mesh held before a cell was built.
pub(in crate::dungeon_map) struct Marks {
    solid: Vec<usize>,
    memory: Vec<usize>,
    see_through: usize,
    stairs_through: Vec<usize>,
    faces: usize,
}

impl Marks {
    pub(in crate::dungeon_map) fn of(layers: &Layers) -> Self {
        let lengths = |builders: &[super::Builder]| -> Vec<usize> {
            builders.iter().map(|b| b.positions.len()).collect()
        };
        Self {
            solid: lengths(&layers.solid),
            memory: lengths(&layers.memory),
            see_through: layers.see_through.positions.len(),
            stairs_through: lengths(&layers.stairs_through),
            faces: layers.solid_faces.len(),
        }
    }

    /// Raises everything built since onto ground `by` high.
    pub(in crate::dungeon_map) fn lift(&self, layers: &mut Layers, by: f32) {
        if by == 0.0 {
            return;
        }
        for (builder, &from) in layers.solid.iter_mut().zip(&self.solid) {
            builder.lift(from, by);
        }
        for (builder, &from) in layers.memory.iter_mut().zip(&self.memory) {
            builder.lift(from, by);
        }
        layers.see_through.lift(self.see_through, by);
        for (builder, &from) in layers.stairs_through.iter_mut().zip(&self.stairs_through) {
            builder.lift(from, by);
        }
        for face in &mut layers.solid_faces[self.faces..] {
            face.centre.y += by;
        }
    }
}

/// The faces down from a cell to each lower cell beside it (not diagonals):
/// lit when the cell is in sight, and always in grey for the part out of it
/// (the mask, read just in front of a face, goes by the lower cell).
pub(super) fn build_edges(layers: &mut Layers, scene: &Scene, (x, y): (i64, i64)) {
    let Some(tints) = scene.tint else {
        return;
    };
    let top = scene.ground(x, y);
    let centre = Vec3::new(x as f32 * CELL_SIZE, 0.0, y as f32 * CELL_SIZE);
    let grass = scene.level.cell(x, y) == Cell::Grass;
    let lit = with_ambient(scene.light.at(x, y)) * tints.at(x, y, grass);
    let seen = scene.sight.is_seen(x, y);
    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let foot = scene.ground(x + dx, y + dy);
        if top - foot < LEAST_DROP {
            continue;
        }
        let normal = Vec3::new(dx as f32, 0.0, dy as f32);
        let side = centre + normal * (CELL_SIZE / 2.0);
        let edge = Edge {
            side,
            normal,
            top,
            foot,
        };
        if seen {
            edge.draw(layers, false, lit);
        }
        edge.draw(layers, true, Vec3::splat(MEMORY_SHADE));
    }
}

/// One face down from a cell's side.
struct Edge {
    /// The middle of the cell's side, at height 0.
    side: Vec3,
    /// Outwards, towards the lower cell.
    normal: Vec3,
    top: f32,
    foot: f32,
}

impl Edge {
    fn draw(&self, layers: &mut Layers, memory: bool, colour: Vec3) {
        let builders = match memory {
            true => &mut layers.memory,
            false => &mut layers.solid,
        };
        let builder = &mut builders[Surface::Ground as usize];
        let along = Vec3::Y.cross(self.normal) * (CELL_SIZE / 2.0);
        let at = |height: f32, out: f32| self.side + self.normal * out + Vec3::Y * height;
        let drop = (self.top - self.foot) / CELL_SIZE;
        let (top, foot) = (at(self.top, 0.0), at(self.foot, 0.0));
        builder.free(
            [top - along, top + along, foot + along, foot - along],
            self.normal,
            [[0.0, 0.0], [1.0, 0.0], [1.0, drop], [0.0, drop]],
            [
                colour * EDGE_TOP_SHADE,
                colour * EDGE_TOP_SHADE,
                colour * EDGE_FOOT_SHADE,
                colour * EDGE_FOOT_SHADE,
            ],
        );
        let line_foot = (self.top - LINE_HEIGHT).max(self.foot);
        let (high, low) = (at(self.top, LINE_OUT), at(line_foot, LINE_OUT));
        let black = Vec3::splat(LINE_SHADE);
        builder.free(
            [high - along, high + along, low + along, low - along],
            self.normal,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 0.04], [0.0, 0.04]],
            [black; 4],
        );
    }
}
