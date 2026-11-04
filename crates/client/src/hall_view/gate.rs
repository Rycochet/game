//! Models of World Gates: a frame by the gate's design (`GateDesign`), a little
//! wider inside than a tile, and its code in runes along the top. The same
//! models serve the Hall of Worlds and, later, World Gate pois on the world
//! map.
//!
//! Units are cells: `x` along the frame, `y` up from the ground, `z` through
//! it, the gate standing on the origin. Shading is baked into the colours, so
//! they suit unlit materials, as the dungeon view's are.

mod frame;
mod profile;
mod runes;
mod water;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use shared::hall::{GateCode, GateDesign};

/// How thick the frame is, how wide its posts are beside the opening, and
/// the band above the opening its runes are on.
const DEPTH: f32 = 0.22;
const POST: f32 = 0.12;
const RUNE_BAND: f32 = 0.18;

/// Light on the faces, and the light they always get.
const LIGHT: [f32; 3] = [-0.45, 0.8, 0.4];
const AMBIENT: f32 = 0.55;

type Rgb = [u8; 3];

/// The frame's stone and the runes' gold.
const STONE: Rgb = [214, 208, 198];
const RUNE_GOLD: Rgb = [232, 190, 84];

/// Triangles with a normal and colour at each corner.
#[derive(Default)]
pub struct GateModel {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    colours: Vec<Rgb>,
}

impl GateModel {
    /// A gate's frame, without its runes.
    pub fn frame(design: GateDesign) -> Self {
        let mut model = Self::default();
        frame::build(&mut model, design, STONE);
        model
    }

    /// A gate's runes, to show over its frame (`frame`) when near enough to
    /// read.
    pub fn runes(design: GateDesign, code: GateCode) -> Self {
        let mut model = Self::default();
        runes::build(&mut model, code, frame::band(design), RUNE_GOLD);
        model
    }

    /// The sheet of water in its opening, scaled by `scale` (the size of a
    /// cell), with picture coordinates for `water_picture`; draw it from both
    /// sides, see-through.
    pub fn water(design: GateDesign, scale: f32) -> Mesh {
        water::mesh(design, scale)
    }

    /// Blue with paler streaks, repeating, to slide downwards over `water`.
    pub fn water_picture() -> Image {
        water::picture()
    }

    /// How far it reaches across from its middle, and how high.
    // For World Gate pois on the world map (owner), which don't exist yet.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn size(design: GateDesign) -> (f32, f32) {
        frame::outline(design)
    }

    /// Corners of each triangle in turn, for drawing it some other way (a
    /// poi's picture on the maps).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn triangles(&self) -> impl Iterator<Item = ([Vec3; 3], Rgb)> + '_ {
        self.positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(self.colours.as_chunks::<3>().0)
            .map(|(corners, colours)| (*corners, colours[0]))
    }

    /// Two triangles over four corners, facing `normal`.
    fn quad(&mut self, [a, b, c, d]: [Vec3; 4], normal: Vec3, colour: Rgb) {
        let faces = (b - a).cross(c - a).dot(normal) >= 0.0;
        let corners = if faces {
            [a, b, c, a, c, d]
        } else {
            [a, c, b, a, d, c]
        };
        for corner in corners {
            self.positions.push(corner);
            self.normals.push(normal);
            self.colours.push(colour);
        }
    }

    /// A mesh of it, scaled by `scale` (the size of a cell), its colours
    /// shaded by how its faces meet the light.
    pub fn mesh(&self, scale: f32) -> Mesh {
        let light = Vec3::from(LIGHT).normalize();
        let colours: Vec<[f32; 4]> = self
            .normals
            .iter()
            .zip(&self.colours)
            .map(|(normal, colour)| {
                let shade = AMBIENT + (1.0 - AMBIENT) * normal.dot(light).max(0.0);
                let [r, g, b] = colour.map(|c| f32::from(c) / 255.0 * shade);
                Color::srgb(r, g, b).to_linear().to_f32_array()
            })
            .collect();
        let positions: Vec<Vec3> = self.positions.iter().map(|p| *p * scale).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32((0..self.positions.len() as u32).collect()))
    }
}

#[cfg(test)]
mod tests;
