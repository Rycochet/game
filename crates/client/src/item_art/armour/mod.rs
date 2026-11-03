//! Armour models, one file per body location. Each piece is built in millimetres in
//! place on `shared::body::Frame`, so its own picture, a set and a dressed character
//! all use the same parts. What every location's models share is here.

pub(super) mod hands;
pub(super) mod shoulders;
mod surface;
pub(super) mod torso;

use bevy::prelude::*;
use shared::material::Material;

use crate::item_art::body::Triangle;

/// What each part is made of, drawn with its own material.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::item_art) enum Stuff {
    /// The plates, rings, scales, leather or padding.
    Main,
    /// Main a shade darker: the rows of mail, the lines of quilting.
    Shade,
    /// The leather glove under plates and scales.
    Backing,
    /// Cloth over or under the plates.
    Cloth,
    Trim,
    /// Rivet heads and studs.
    Rivet,
    /// Greenwich's darkened, etched bands.
    Etch,
    /// Heraldry's field and charge, painted.
    Field,
    Charge,
    /// Skulls and horns.
    Bone,
}

/// Parts by what they are made of.
#[derive(Default)]
pub(in crate::item_art) struct Model {
    pub parts: Vec<(Stuff, Vec<Triangle>)>,
}

impl Model {
    pub(in crate::item_art) fn of(&mut self, stuff: Stuff) -> &mut Vec<Triangle> {
        let at = match self.parts.iter().position(|(have, _)| *have == stuff) {
            Some(at) => at,
            None => {
                self.parts.push((stuff, Vec::new()));
                self.parts.len() - 1
            }
        };
        &mut self.parts[at].1
    }

    pub(in crate::item_art) fn join(&mut self, other: Model) {
        for (stuff, triangles) in other.parts {
            self.of(stuff).extend(triangles);
        }
    }

    pub(in crate::item_art) fn map(&mut self, change: impl Fn(Vec3) -> Vec3) {
        for (_, triangles) in &mut self.parts {
            for triangle in triangles.iter_mut() {
                *triangle = triangle.map(&change);
            }
        }
    }

    pub(in crate::item_art) fn bounds(&self) -> (Vec3, Vec3) {
        let corners = self.parts.iter().flat_map(|(_, t)| t.iter().flatten());
        corners.fold(
            (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN)),
            |(low, high), &corner| (low.min(corner), high.max(corner)),
        )
    }
}

/// Turns a triangle to face `out`.
pub(in crate::item_art) fn face(
    triangles: &mut Vec<Triangle>,
    a: Vec3,
    b: Vec3,
    c: Vec3,
    out: Vec3,
) {
    let forward = (b - a).cross(c - a).dot(out) >= 0.0;
    triangles.push(if forward { [a, b, c] } else { [a, c, b] });
}

/// A plate of some thickness from rows of points, each with the way out from it.
/// `closed` joins the last of each row to its first, as round a cuff.
// Indices, as rows wrap round a closed cuff.
#[allow(clippy::needless_range_loop)]
pub(in crate::item_art) fn plate(
    triangles: &mut Vec<Triangle>,
    grid: &[Vec<(Vec3, Vec3)>],
    thick: f32,
    closed: bool,
) {
    let outer = |r: usize, c: usize| grid[r][c].0 + grid[r][c].1 * thick / 2.0;
    let inner = |r: usize, c: usize| grid[r][c].0 - grid[r][c].1 * thick / 2.0;
    let rows = grid.len();
    let columns = grid[0].len();
    let spans = if closed { columns } else { columns - 1 };
    for r in 0..rows - 1 {
        for c in 0..spans {
            let n = (c + 1) % columns;
            let out = grid[r][c].1 + grid[r][n].1 + grid[r + 1][c].1 + grid[r + 1][n].1;
            let [a, b, cc, d] = [outer(r, c), outer(r, n), outer(r + 1, n), outer(r + 1, c)];
            face(triangles, a, b, cc, out);
            face(triangles, a, cc, d, out);
            let [a, b, cc, d] = [inner(r, c), inner(r, n), inner(r + 1, n), inner(r + 1, c)];
            face(triangles, a, b, cc, -out);
            face(triangles, a, cc, d, -out);
        }
    }
    // The edges: top and bottom rows, then the sides if open.
    for (r, next) in [(0, 1), (rows - 1, rows - 2)] {
        for c in 0..spans {
            let n = (c + 1) % columns;
            let away = grid[r][c].0 - grid[next][c].0;
            let [a, b, cc, d] = [outer(r, c), outer(r, n), inner(r, n), inner(r, c)];
            face(triangles, a, b, cc, away);
            face(triangles, a, cc, d, away);
        }
    }
    if !closed {
        for (c, next) in [(0, 1), (columns - 1, columns - 2)] {
            for r in 0..rows - 1 {
                let away = grid[r][c].0 - grid[r][next].0;
                let [a, b, cc, d] = [outer(r, c), outer(r + 1, c), inner(r + 1, c), inner(r, c)];
                face(triangles, a, b, cc, away);
                face(triangles, a, cc, d, away);
            }
        }
    }
}

/// What a non-metal looks like.
pub(in crate::item_art) fn colour_of(material: Material) -> Color {
    match material {
        Material::Leather => Color::srgb(0.42, 0.26, 0.14),
        Material::HardenedLeather => Color::srgb(0.32, 0.18, 0.09),
        Material::BuffLeather => Color::srgb(0.78, 0.64, 0.42),
        Material::Horn => Color::srgb(0.55, 0.45, 0.3),
        Material::Chitin => Color::srgb(0.16, 0.2, 0.14),
        Material::DragonScale => Color::srgb(0.55, 0.12, 0.1),
        Material::Linen => Color::srgb(0.86, 0.82, 0.7),
        Material::Wool => Color::srgb(0.5, 0.42, 0.36),
        Material::Cotton => Color::srgb(0.93, 0.92, 0.88),
        Material::Silk => Color::srgb(0.6, 0.1, 0.15),
        Material::Wood => Color::srgb(0.5, 0.36, 0.22),
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

pub(in crate::item_art) fn matte(colour: Color, roughness: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: colour,
        perceptual_roughness: roughness,
        ..default()
    }
}

pub(in crate::item_art) fn darker(material: StandardMaterial) -> StandardMaterial {
    let colour = material.base_color.to_srgba();
    StandardMaterial {
        base_color: Color::srgb(colour.red * 0.6, colour.green * 0.6, colour.blue * 0.6),
        ..material
    }
}
