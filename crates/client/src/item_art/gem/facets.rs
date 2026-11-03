//! Building a cut gem's facets from its outline: brilliant and step cuts.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::item_form::Cut;
use shared::rng::Seed;

use super::{CROWN_HEIGHT, PAVILION_DEPTH, TABLE, outline};

/// Triangles with their own flat faces, each a little lighter or darker.
pub(super) struct Facets {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    /// How far from white each face's shade may go.
    variation: f32,
    shades: shared::rng::Rng,
}

impl Facets {
    pub(super) fn new(seed: u64, variation: f32) -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colours: Vec::new(),
            variation,
            shades: Seed::new(seed).rng(),
        }
    }

    /// A face, turned to look away from the middle, as the whole is convex.
    pub(super) fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
        let mut normal = (b - a).cross(c - a).normalize_or_zero();
        let (b, c) = if normal.dot(a + b + c) < 0.0 {
            normal = -normal;
            (c, b)
        } else {
            (b, c)
        };
        let shade = 1.0 + self.shades.range_f64(-1.0, 1.0) as f32 * self.variation;
        for point in [a, b, c] {
            self.positions.push(point.to_array());
            self.normals.push(normal.to_array());
            self.colours.push([shade, shade, shade, 1.0]);
        }
    }

    fn quad(&mut self, a: Vec3, b: Vec3, c: Vec3, d: Vec3) {
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    pub(super) fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
    }
}

pub(super) fn at(point: Vec2, scale: f32, height: f32) -> Vec3 {
    Vec3::new(point.x * scale, height, point.y * scale)
}

pub(super) fn cut_mesh(cut: Cut, variation: f32) -> Mesh {
    let mut facets = Facets::new(cut as u64 + 1, variation);
    let outline = outline(cut);
    if cut == Cut::Emerald {
        stepped(&mut facets, &outline);
    } else {
        brilliant(&mut facets, &outline);
    }
    facets.into_mesh()
}

/// Kites and stars from a flat top down to the outline, and long faces to a point
/// below.
pub(super) fn brilliant(facets: &mut Facets, outline: &[Vec2]) {
    let n = outline.len();
    let middle = Vec3::Y * -PAVILION_DEPTH;
    let girdle = |i: usize| at(outline[i % n], 1.0, 0.0);
    let table = |i: usize| at(outline[i % n], TABLE, CROWN_HEIGHT);
    let centre = Vec3::Y * CROWN_HEIGHT;
    for i in (0..n).step_by(2) {
        facets.triangle(table(i), girdle(i), girdle(i + 1));
        facets.triangle(table(i), girdle(i + 1), table(i + 2));
        facets.triangle(table(i + 2), girdle(i + 1), girdle(i + 2));
        facets.triangle(centre, table(i), table(i + 2));
    }
    for i in 0..n {
        facets.triangle(middle, girdle(i), girdle(i + 1));
    }
}

/// Long steps, in rings of the same outline, above and below.
pub(super) fn stepped(facets: &mut Facets, outline: &[Vec2]) {
    /// How wide each ring is, and how high.
    const RINGS: [(f32, f32); 4] = [(1.0, 0.0), (0.84, 0.13), (0.68, 0.23), (0.55, CROWN_HEIGHT)];
    const BELOW: [(f32, f32); 4] = [
        (1.0, 0.0),
        (0.72, -0.28),
        (0.42, -0.52),
        (0.14, -PAVILION_DEPTH),
    ];
    for rings in [&RINGS, &BELOW] {
        for pair in rings.windows(2) {
            for i in 0..outline.len() {
                let (here, next) = (outline[i], outline[(i + 1) % outline.len()]);
                facets.quad(
                    at(here, pair[0].0, pair[0].1),
                    at(next, pair[0].0, pair[0].1),
                    at(next, pair[1].0, pair[1].1),
                    at(here, pair[1].0, pair[1].1),
                );
            }
        }
        // Closes the ring at its end: the flat top, or the keel below.
        let (scale, height) = rings[3];
        for i in 0..outline.len() {
            let (here, next) = (outline[i], outline[(i + 1) % outline.len()]);
            facets.triangle(
                Vec3::Y * height,
                at(here, scale, height),
                at(next, scale, height),
            );
        }
    }
}
