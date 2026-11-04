//! How much light reaches each cell of a level.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};

/// How much light reaches each cell of a level, as linear RGB from 0 (only
/// the dull ambient light) to 1 (fully lit).
pub struct LightGrid {
    min_x: i64,
    min_y: i64,
    width: i64,
    height: i64,
    light: Vec<Vec3>,
}

impl LightGrid {
    pub fn new(level: &Level) -> Self {
        Self::filled(level, Vec3::ZERO)
    }

    /// The same light on every cell.
    pub fn filled(level: &Level, light: Vec3) -> Self {
        let (min_x, min_y, width, height) = level.bounds();
        Self {
            min_x,
            min_y,
            width,
            height,
            light: vec![light; (width * height) as usize],
        }
    }

    fn index(&self, x: i64, y: i64) -> Option<usize> {
        let (col, row) = (x - self.min_x, y - self.min_y);
        ((0..self.width).contains(&col) && (0..self.height).contains(&row))
            .then_some((row * self.width + col) as usize)
    }

    /// Sets the light on a cell.
    pub fn set(&mut self, x: i64, y: i64, light: Vec3) {
        if let Some(i) = self.index(x, y) {
            self.light[i] = light;
        }
    }

    /// Adds as much of `colour` as `by` says at each cell.
    pub fn add(&mut self, colour: Vec3, by: impl Fn(i64, i64) -> f32) {
        for row in 0..self.height {
            for col in 0..self.width {
                let i = (row * self.width + col) as usize;
                let amount = by(col + self.min_x, row + self.min_y);
                self.light[i] = (self.light[i] + colour * amount).min(Vec3::ONE);
            }
        }
    }

    /// Raises every cell to at least `least` of full light (Brightness). Only
    /// what is in sight is drawn lit, so nothing unseen shows.
    pub fn raise_to(&mut self, least: f32) {
        for light in &mut self.light {
            *light = light.max(Vec3::splat(least));
        }
    }

    /// The light at a cell (none outside the level).
    pub fn at(&self, x: i64, y: i64) -> Vec3 {
        self.index(x, y).map_or(Vec3::ZERO, |i| self.light[i])
    }

    /// Adds a light at `(x, y)` reaching `radius` cells. Walls block it (the
    /// same rule as sight, from `shared`); it fades slowly at first, then
    /// faster, to nothing just beyond the radius.
    pub fn add_light(&mut self, level: &Level, (x, y): (i64, i64), radius: u32, colour: Vec3) {
        let mut reached = Sight::new(level);
        reached.add_view(level, x, y, Some(radius));
        let reach = radius as f32 + 1.0;
        let r = radius as i64;
        for cy in y - r..=y + r {
            for cx in x - r..=x + r {
                if !reached.is_seen(cx, cy) {
                    continue;
                }
                let distance = Vec2::new((cx - x) as f32, (cy - y) as f32).length();
                let strength = 1.0 - (distance / reach).powi(2);
                if let Some(i) = self.index(cx, cy) {
                    self.light[i] = (self.light[i] + colour * strength).min(Vec3::ONE);
                }
            }
        }
    }

    /// The light at a grid corner (between four cells; corner `(x, y)` is the
    /// north-west corner of cell `(x, y)`): the average over the open cells
    /// around it, so light fades smoothly across floors without leaking out
    /// of walls. `None` if no open cell touches it.
    pub fn corner(&self, level: &Level, x: i64, y: i64) -> Option<Vec3> {
        let mut total = Vec3::ZERO;
        let mut count = 0.0;
        for (cx, cy) in [(x - 1, y - 1), (x, y - 1), (x - 1, y), (x, y)] {
            if level.is_open(cx, cy) {
                total += self.at(cx, cy);
                count += 1.0;
            }
        }
        (count > 0.0).then(|| total / count)
    }
}
