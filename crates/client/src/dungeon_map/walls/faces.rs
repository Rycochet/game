//! Which of a wall's faces show, and where they are.

use bevy::prelude::*;

use super::super::WALL_HEIGHT;
use super::super::mesh::faces_camera;
use super::{Patch, QUARTERS, Tile};

impl<'a> Tile<'a> {
    /// Where a wall stands on the floor in a quarter: the part running one
    /// way, then what the other way adds to it (as `wall_top`).
    pub(super) fn wall_foot(
        &self,
        (east_west, north_south): (usize, usize),
        reach: [f32; 4],
    ) -> Vec<Patch> {
        let (z_plane, _, x_end) = self.face_at(north_south, east_west, reach, 0.0);
        let (x_plane, _, z_end) = self.face_at(east_west, north_south, reach, 0.0);
        let mut foot = vec![Patch {
            east: (0.0, x_end),
            south: (0.0, z_plane),
        }];
        if z_end > z_plane {
            foot.push(Patch {
                east: (0.0, x_plane),
                south: (z_plane, z_end),
            });
        }
        foot
    }

    /// The sides (north, east, south, west) a face towards side `facing` of
    /// a quarter runs towards: `facing` itself and the side it runs out to.
    pub(super) fn quarter_sides((sx, sz): (f32, f32)) -> (usize, usize) {
        (if sx < 0.0 { 3 } else { 1 }, if sz < 0.0 { 0 } else { 2 })
    }

    /// Which stretch of wall a face is part of: all of it, unless the wall
    /// turns a corner here, when each way it runs is a stretch of its own.
    pub(super) fn stretch(&self, facing: usize) -> usize {
        let north_south = self.joins[0] || self.joins[2];
        let east_west = self.joins[1] || self.joins[3];
        if north_south && east_west {
            facing % 2
        } else {
            0
        }
    }

    /// For each stretch of this wall, whether a face someone sees is turned
    /// away from the camera: then all its seen faces are see-through, so both
    /// sides of a wall always look the same.
    pub(super) fn turned_away(&self, reach: [f32; 4]) -> [bool; 2] {
        let mut away = [false; 2];
        for quarter in QUARTERS {
            let (east_west, north_south) = Self::quarter_sides(quarter);
            for (facing, running) in [(north_south, east_west), (east_west, north_south)] {
                if self.face_seen(quarter, facing)
                    && let Some((middle, normal)) =
                        self.face_middle(quarter, facing, running, reach)
                    && !faces_camera(middle, normal, self.scene.camera)
                {
                    away[self.stretch(facing)] = true;
                }
            }
        }
        away
    }

    /// Whether a viewer sees a face towards side `facing` in a quarter: it
    /// sees that quarter, and isn't on the other side of the wall. Someone in
    /// line with a wall, side-on to it, sees both its sides.
    pub(super) fn face_seen(&self, quarter: (f32, f32), facing: usize) -> bool {
        let (dx, dy) = [(0, -1), (1, 0), (0, 1), (-1, 0)][facing];
        self.scene.views.iter().any(|view| {
            let (vx, vy) = view.cell;
            self.quarter_shows(&view.sight, quarter) && (vx - self.x) * dx + (vy - self.y) * dy >= 0
        })
    }

    /// The middle of a face and the way it faces, if the face exists.
    fn face_middle(
        &self,
        (sx, sz): (f32, f32),
        facing: usize,
        running: usize,
        reach: [f32; 4],
    ) -> Option<(Vec3, Vec3)> {
        let (plane, from, to) = self.face_at(facing, running, reach, WALL_HEIGHT / 2.0);
        if to <= from {
            return None;
        }
        let along = (from + to) / 2.0;
        let (east, south, normal) = if facing.is_multiple_of(2) {
            (sx * along, sz * plane, Vec3::Z * sz)
        } else {
            (sx * plane, sz * along, Vec3::X * sx)
        };
        Some((
            self.centre + Vec3::new(east, WALL_HEIGHT / 2.0, south),
            normal,
        ))
    }
}
