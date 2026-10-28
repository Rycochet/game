//! The terrain window mesh and the front face, both rebuilt from a height
//! cache whenever the player moves.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use shared::surface::{Surface, SurfaceSample};

use super::{HALF_SIZE, HEIGHT_SCALE, SLOPE_SHADING};
use crate::surface_style::{SurfaceStyle, hsl_to_rgba};

/// Vertices along each side of the window.
const SIDE: usize = (2 * HALF_SIZE + 1) as usize;

/// Bottom of the front face, as a fraction of the water level (0 = height 0,
/// as in the prototype; raised to make the face shorter).
const FRONT_BASE: f32 = 0.3;

/// Undoes the sun's dimming of flat frozen water (lit flat ice measured at
/// ~71% of its colour in linear light).
const ICE_BOOST: f32 = 1.41;

/// Cache side length: larger than the window (plus a 1-unit border for
/// normals) so moving never evicts samples still in view.
const CACHE_SIDE: i64 = 512;

/// Surface samples by grid position, stored in a wrapping 2D grid. Moving one
/// step only samples the new edge; everything else is reused.
#[derive(Resource)]
pub struct HeightCache {
    keys: Vec<Option<(i64, i64)>>,
    samples: Vec<SurfaceSample>,
}

impl Default for HeightCache {
    fn default() -> Self {
        let len = (CACHE_SIDE * CACHE_SIDE) as usize;
        Self {
            keys: vec![None; len],
            samples: vec![
                SurfaceSample {
                    height: 0.0,
                    jitter: 0.0
                };
                len
            ],
        }
    }
}

impl HeightCache {
    pub fn get(&mut self, surface: &Surface, x: i64, y: i64) -> SurfaceSample {
        let index = (y.rem_euclid(CACHE_SIDE) * CACHE_SIDE + x.rem_euclid(CACHE_SIDE)) as usize;
        if self.keys[index] != Some((x, y)) {
            self.keys[index] = Some((x, y));
            self.samples[index] = surface.sample(x, y);
        }
        self.samples[index]
    }
}

/// An empty mesh for the terrain window: a fixed grid of `SIDE`×`SIDE`
/// vertices, centred on the entity. Heights are filled in by `update_terrain`.
pub fn new_terrain_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(SIDE * SIDE);
    for z in 0..SIDE {
        for x in 0..SIDE {
            positions.push([
                x as f32 - HALF_SIZE as f32,
                0.0,
                z as f32 - HALF_SIZE as f32,
            ]);
        }
    }

    let mut indices = Vec::with_capacity((SIDE - 1) * (SIDE - 1) * 6);
    for z in 0..SIDE - 1 {
        for x in 0..SIDE - 1 {
            let i = (z * SIDE + x) as u32;
            let below = i + SIDE as u32;
            // Two triangles per square, wound anticlockwise seen from above.
            indices.extend_from_slice(&[i, below, i + 1, i + 1, below, below + 1]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 1.0, 0.0]; SIDE * SIDE])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0; 4]; SIDE * SIDE])
    .with_inserted_indices(Indices::U32(indices))
}

/// Fills the terrain mesh with heights, normals and colours for the window
/// centred on `(cx, cy)`.
pub fn update_terrain(
    mesh: &mut Mesh,
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (cx, cy): (i64, i64),
) {
    // Water is part of the terrain: underwater points sit at the surface.
    let mut height = |dx: i64, dz: i64| {
        let sample = cache.get(surface, cx + dx, cy + dz);
        style.shown_height(sample, water_level) as f32 * HEIGHT_SCALE
    };

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
    {
        for (i, position) in positions.iter_mut().enumerate() {
            let (dx, dz) = offset(i);
            position[1] = height(dx, dz);
        }
    }

    if let Some(VertexAttributeValues::Float32x3(normals)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL)
    {
        for (i, normal) in normals.iter_mut().enumerate() {
            let (dx, dz) = offset(i);
            // Slope from the neighbours either side (central differences).
            let slope_x = height(dx + 1, dz) - height(dx - 1, dz);
            let slope_z = height(dx, dz + 1) - height(dx, dz - 1);
            *normal = Vec3::new(-slope_x, 2.0, -slope_z).normalize().to_array();
        }
    }

    if let Some(VertexAttributeValues::Float32x4(colours)) =
        mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR)
    {
        for (i, colour) in colours.iter_mut().enumerate() {
            let (dx, dz) = offset(i);
            let sample = cache.get(surface, cx + dx, cy + dz);
            let west = cache.get(surface, cx + dx - 1, cy + dz).height;
            // The prototype's strong slope shading, baked in per vertex
            // because real lighting alone is much smoother.
            let (band, lightness) =
                style.shade(sample, west, water_level, snow_line, SLOPE_SHADING);
            *colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness));
            if style.is_frozen_water(sample, water_level, snow_line) {
                // Brighter than white, so after the sun dims this flat surface
                // it shows at the style's ice colour.
                for channel in &mut colour[..3] {
                    *channel *= ICE_BOOST;
                }
            }
        }
    }
}

/// The flat front face along the window's southern edge: water depth (where
/// underwater) above earth, down to `FRONT_BASE` × the water level.
pub fn build_front(
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    snow_line: f64,
    (cx, cy): (i64, i64),
) -> Mesh {
    let water_top = water_level as f32 * HEIGHT_SCALE;
    let base = water_top * FRONT_BASE;
    let (h, s, l) = style.earth_face;
    let earth_colour = linear(hsl_to_rgba(h, s, l));
    let z = HALF_SIZE as f32;

    let mut positions = Vec::with_capacity(SIDE * 4);
    let mut colours = Vec::with_capacity(SIDE * 4);
    for dx in -HALF_SIZE..=HALF_SIZE {
        let sample = cache.get(surface, cx + dx, cy + HALF_SIZE);
        let west = cache.get(surface, cx + dx - 1, cy + HALF_SIZE).height;
        let ground = (sample.height as f32 * HEIGHT_SCALE).max(base);
        // Half the surface lightness, like the prototype's front slice (this
        // also ices over with the water).
        let (band, lightness) = style.shade(sample, west, water_level, snow_line, 0.0);
        let water_colour = linear(hsl_to_rgba(band.hue, band.saturation, lightness / 2.0));
        let x = dx as f32;
        // Earth strip: ground down to the base.
        positions.extend_from_slice(&[[x, ground, z], [x, base, z]]);
        colours.extend_from_slice(&[earth_colour, earth_colour]);
        // Water strip: water surface down to the ground (zero height on land).
        positions.extend_from_slice(&[[x, water_top.max(ground), z], [x, ground, z]]);
        colours.extend_from_slice(&[water_colour, water_colour]);
    }

    let mut indices = Vec::with_capacity((SIDE - 1) * 12);
    for column in 0..SIDE as u32 - 1 {
        let here = column * 4;
        let next = here + 4;
        for strip in [0, 2] {
            let (top, bottom) = (here + strip, here + strip + 1);
            let (next_top, next_bottom) = (next + strip, next + strip + 1);
            // Facing south (towards the camera).
            indices.extend_from_slice(&[top, bottom, next_top, next_top, bottom, next_bottom]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// The highest drawn surface within `radius` of a fractional position, so
/// something that wide can rest on top without sinking into a slope.
pub fn highest_shown_height(
    cache: &mut HeightCache,
    surface: &Surface,
    style: &SurfaceStyle,
    water_level: f64,
    (x, y): (f64, f64),
    radius: f64,
) -> f32 {
    let mut highest = f64::MIN;
    let reach = radius.ceil() as i64;
    for dy in -reach..=reach {
        for dx in -reach..=reach {
            let (px, py) = (x.round() as i64 + dx, y.round() as i64 + dy);
            if (px as f64 - x).hypot(py as f64 - y) <= radius {
                let sample = cache.get(surface, px, py);
                highest = highest.max(style.shown_height(sample, water_level));
            }
        }
    }
    highest as f32 * HEIGHT_SCALE
}

/// Grid offset from the window centre of vertex `i`.
fn offset(i: usize) -> (i64, i64) {
    ((i % SIDE) as i64 - HALF_SIZE, (i / SIDE) as i64 - HALF_SIZE)
}

/// sRGB bytes to the linear floats meshes use.
fn linear([r, g, b, _]: [u8; 4]) -> [f32; 4] {
    Color::srgb_u8(r, g, b).to_linear().to_f32_array()
}
