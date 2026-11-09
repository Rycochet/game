//! The hole a door or window leaves in its wall. The wall is drawn round it, not
//! behind it: a picture just in front of a wall fights it for the nearer depth,
//! and on the web (WebGL keeps less depth precision) it flickers and shows in
//! part as the camera moves.

use bevy::prelude::*;
use shared::dungeon::Cell;
use shared::site::BuildingLook;

use super::super::WALL_HEIGHT;
use super::super::walls::Scene;
use super::{HALF_CELL, Piece, WINDOW_HALF, WINDOW_SILL, WINDOW_TOP, cell_centre, window_faces};

/// Where a door or window is on its cell's wall: the side it faces (a grid
/// step), its half width either side of the cell's middle, and its bottom and
/// top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Opening {
    pub(super) faces: (i64, i64),
    pub(super) door: bool,
    pub(super) half: f32,
    pub(super) low: f32,
    pub(super) high: f32,
}

impl Opening {
    /// The door or window on a wall cell of `building`, if it has one.
    pub(super) fn at(scene: &Scene, building: &BuildingLook, (x, y): (i64, i64)) -> Option<Self> {
        match scene.level.cell(x, y) {
            Cell::BuildingDoor if building.door == (x, y) => Some(Self {
                faces: building.faces,
                door: true,
                half: HALF_CELL,
                low: 0.0,
                high: WALL_HEIGHT,
            }),
            Cell::BlindWindow => window_faces(scene.level, (x, y)).map(|faces| Self {
                faces,
                door: false,
                half: WINDOW_HALF,
                low: WINDOW_SILL,
                high: WINDOW_TOP,
            }),
            _ => None,
        }
    }

    pub(super) fn normal(&self) -> Vec3 {
        Vec3::new(self.faces.0 as f32, 0.0, self.faces.1 as f32)
    }

    /// The parts of `piece`, a wall face on cell `(x, y)`, left round this
    /// hole, each with its bottom and top: all of it if it is on another side.
    pub(super) fn around(&self, piece: &Piece, (x, y): (i64, i64)) -> Vec<(Piece, (f32, f32))> {
        let whole = (piece.clone(), (0.0, WALL_HEIGHT));
        if piece.normal != self.normal() {
            return vec![whole];
        }
        // Along the wall, from the cell's middle.
        let along = Vec3::new(self.faces.1 as f32, 0.0, -self.faces.0 as f32);
        let middle = cell_centre(x, y);
        let (a, b) = (along.dot(piece.from - middle), along.dot(piece.to - middle));
        if (a - b).abs() < f32::EPSILON {
            return vec![whole];
        }
        // Where the hole's sides fall, as shares of the way from `from` to `to`.
        let share = |at: f32| ((at - a) / (b - a)).clamp(0.0, 1.0);
        let (first, second) = (share(-self.half), share(self.half));
        let (start, end) = (first.min(second), first.max(second));
        let mut parts = Vec::new();
        if self.low > 0.0 {
            parts.push((piece.clone(), (0.0, self.low)));
        }
        if self.high < WALL_HEIGHT {
            parts.push((piece.clone(), (self.high, WALL_HEIGHT)));
        }
        for (from, to) in [(0.0, start), (end, 1.0)] {
            if to - from > f32::EPSILON {
                parts.push((piece.part(from, to), (self.low, self.high)));
            }
        }
        parts
    }
}

impl Piece {
    /// The part of it from share `from` to share `to` of the way along.
    fn part(&self, from: f32, to: f32) -> Piece {
        let across = |share: f32| self.across.0 + (self.across.1 - self.across.0) * share;
        Piece {
            from: self.from.lerp(self.to, from),
            to: self.from.lerp(self.to, to),
            normal: self.normal,
            across: (across(from), across(to)),
        }
    }
}

#[cfg(test)]
mod tests;
