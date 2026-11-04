//! How each moon looks on the globe: a grey surface in one of a few styles,
//! tinted by its colour, and a turn about its own axis. Only for looks, but
//! from the terrain seed, so a world's moons always look the same.

mod pattern;

use bevy::asset::RenderAssetUsages;
use bevy::math::{Quat, Vec3};
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use shared::rng::Seed;
use std::f64::consts::TAU;

/// The moons' pictures, wrapped round them: longitude across, pole to pole down.
const WIDTH: u32 = 128;
const HEIGHT: u32 = 64;

/// The quickest and slowest a moon turns about its own axis, in game hours.
const QUICKEST_TURN: f64 = 8.0;
const SLOWEST_TURN: f64 = 60.0;

/// What a moon's surface is like.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Style {
    /// Round dips with bright rims, large and small.
    Cratered,
    /// Stripes along its equator, with wavy edges.
    Banded,
    /// Pale, with dark thin lines.
    CrackedIce,
    /// Large smooth light and dark areas.
    Patchy,
}

/// One moon's looks.
#[derive(Clone, Copy, Debug)]
pub(super) struct MoonLook {
    pub(super) style: Style,
    seed: Seed,
    /// What it turns about.
    axis: Vec3,
    /// Game hours to turn once; negative turns the other way.
    turn_hours: f64,
}

impl MoonLook {
    /// The looks of moon `number` (counting moons only) of a world.
    pub(super) fn of(terrain: Seed, number: usize) -> Self {
        let seed = terrain.derive(&format!("planet view moon {number}"));
        let mut rng = seed.rng();
        let style = [
            Style::Cratered,
            Style::Banded,
            Style::CrackedIce,
            Style::Patchy,
        ][rng.range_u64(0, 4) as usize];
        // Any way up: an even spread over the sphere.
        let up = rng.range_f64(-1.0, 1.0);
        let turn = rng.next_f64() * TAU;
        let flat = (1.0 - up * up).sqrt();
        let axis = Vec3::new(
            (flat * turn.cos()) as f32,
            up as f32,
            (flat * turn.sin()) as f32,
        );
        let mut turn_hours = rng.range_f64(QUICKEST_TURN, SLOWEST_TURN);
        if rng.next_f64() < 0.5 {
            turn_hours = -turn_hours;
        }
        Self {
            style,
            seed,
            axis,
            turn_hours,
        }
    }

    /// How it is turned at an hour.
    pub(super) fn turn(&self, hours: f64) -> Quat {
        let angle = (TAU * (hours / self.turn_hours).rem_euclid(1.0)) as f32;
        Quat::from_axis_angle(self.axis.normalize(), angle)
    }

    /// Its surface in greys, to be tinted by its colour.
    pub(super) fn image(&self) -> Image {
        let pattern = pattern::Pattern::new(self.style, self.seed);
        let mut data = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let grey = pattern.grey(sphere_point(column, row));
                let byte = (grey.clamp(0.0, 1.0) * 255.0).round() as u8;
                data.extend_from_slice(&[byte, byte, byte, 255]);
            }
        }
        Image::new(
            Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}

/// The point of a unit sphere a pixel's middle shows, so patterns run on
/// across the picture's edges and close neatly at the poles.
fn sphere_point(column: u32, row: u32) -> Vec3 {
    let longitude = TAU as f32 * (column as f32 + 0.5) / WIDTH as f32;
    let latitude = std::f32::consts::PI * (0.5 - (row as f32 + 0.5) / HEIGHT as f32);
    Vec3::new(
        latitude.cos() * longitude.cos(),
        latitude.sin(),
        -latitude.cos() * longitude.sin(),
    )
}
