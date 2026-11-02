//! A gem's mesh and how it is drawn. Seven cuts share one builder: an outline, with a
//! flat top above it and a point below. An uncut stone is a lumpy sphere.

use std::f32::consts::TAU;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use shared::item::Quality;
use shared::item_form::{Cut, GemForm};
use shared::material::Material;
use shared::rng::Seed;

/// Points round the outline. Even, for the crown's table to take every other one.
const OUTLINE_POINTS: usize = 16;
/// A multiple of 3 too, so that a trillion has its three corners.
const TRILLION_POINTS: usize = 18;
/// How wide the flat top is, against the outline.
const TABLE: f32 = 0.55;
const CROWN_HEIGHT: f32 = 0.3;
const PAVILION_DEPTH: f32 = 0.7;
/// How much of the back shows through the front of a cut gem.
const SEE_THROUGH: f32 = 0.4;
/// How much darker the back is than the front.
const BACK_SHADE: f32 = 0.6;

/// One gem's parts, in the order they are drawn.
pub(super) fn gem_parts(
    gem: &GemForm,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<(Handle<Mesh>, Handle<StandardMaterial>)> {
    let colour = gem_colour(gem.material);
    if gem.cut == Cut::Uncut {
        let mesh = meshes.add(uncut_mesh(gem.shape_seed));
        let material = materials.add(StandardMaterial {
            base_color: colour.mix(&Color::srgb(0.45, 0.42, 0.4), 0.3),
            perceptual_roughness: 0.9,
            reflectance: 0.15,
            ..default()
        });
        return vec![(mesh, material)];
    }
    let fineness = quality.unwrap_or_default() as usize as f32 / 6.0;
    let mesh = meshes.add(cut_mesh(gem.cut, 0.15 + 0.25 * fineness));
    let roughness = 0.3 - 0.22 * fineness;
    let back = materials.add(StandardMaterial {
        base_color: colour.darker(1.0 - BACK_SHADE),
        perceptual_roughness: roughness,
        reflectance: 0.5,
        // What is drawn is the inside of the far faces, lit as seen from here.
        cull_mode: Some(Face::Front),
        double_sided: true,
        ..default()
    });
    let front = materials.add(StandardMaterial {
        base_color: colour.with_alpha(1.0 - SEE_THROUGH),
        perceptual_roughness: roughness,
        reflectance: 0.9,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    vec![(mesh.clone(), back), (mesh, front)]
}

fn gem_colour(gem: Material) -> Color {
    match gem {
        Material::Amethyst => Color::srgb(0.55, 0.2, 0.8),
        Material::Aquamarine => Color::srgb(0.4, 0.85, 0.9),
        Material::Citrine => Color::srgb(0.98, 0.72, 0.15),
        Material::Diamond => Color::srgb(0.93, 0.96, 1.0),
        Material::Emerald => Color::srgb(0.05, 0.7, 0.3),
        Material::Garnet => Color::srgb(0.5, 0.05, 0.12),
        Material::Peridot => Color::srgb(0.6, 0.8, 0.15),
        Material::Ruby => Color::srgb(0.85, 0.05, 0.18),
        Material::Sapphire => Color::srgb(0.08, 0.2, 0.8),
        Material::Topaz => Color::srgb(0.95, 0.55, 0.3),
        _ => Color::WHITE,
    }
}

/// Where a cut's outline goes, clockwise from the top of the picture, in the flat
/// plane (`x`, `z`), with `-z` being away from the viewer.
fn outline(cut: Cut) -> Vec<Vec2> {
    let round = |n: usize, shape: &dyn Fn(f32) -> f32, stretch: Vec2| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let angle = i as f32 / n as f32 * TAU;
                let along = Vec2::new(angle.sin(), -angle.cos());
                along * shape(angle) * stretch
            })
            .collect()
    };
    let circle = |_: f32| 1.0;
    match cut {
        Cut::Round => round(OUTLINE_POINTS, &circle, Vec2::ONE),
        Cut::Oval => round(OUTLINE_POINTS, &circle, Vec2::new(0.72, 1.0)),
        Cut::Cushion => {
            // A circle pushed out towards a square.
            let squarish = |angle: f32| {
                let (x, y) = (angle.sin().abs(), angle.cos().abs());
                0.92 * (x.powi(4) + y.powi(4)).powf(-0.25)
            };
            round(OUTLINE_POINTS, &squarish, Vec2::ONE)
        }
        Cut::Princess => {
            let square = |angle: f32| 0.8 / angle.sin().abs().max(angle.cos().abs());
            round(OUTLINE_POINTS, &square, Vec2::ONE)
        }
        Cut::Trillion => {
            // Between a triangle and a circle, corners first, mixed so that it can't
            // bend inwards.
            let triangle = |angle: f32| {
                let third = TAU / 3.0;
                let into_side = angle.rem_euclid(third) - third / 2.0;
                let side = (third / 2.0).cos() / into_side.cos();
                1.02 / (0.8 / side + 0.2)
            };
            round(TRILLION_POINTS, &triangle, Vec2::ONE)
        }
        Cut::Pear => pear(),
        Cut::Emerald => emerald_outline(),
        // Has no outline: see `uncut_mesh`.
        Cut::Uncut => Vec::new(),
    }
}

/// A point away from the viewer, its sides straight down to a round end.
fn pear() -> Vec<Vec2> {
    const RADIUS: f32 = 0.68;
    let centre = Vec2::new(0.0, 0.32);
    let tip = Vec2::new(0.0, -1.0);
    // How far round the circle from straight at the tip the straight sides touch it.
    let touches = (RADIUS / (centre - tip).length()).acos();
    let mut points = vec![tip];
    let arc = OUTLINE_POINTS - 1;
    for i in 0..arc {
        let along = i as f32 / (arc - 1) as f32;
        let angle = touches + along * (TAU - 2.0 * touches);
        points.push(centre + RADIUS * Vec2::new(angle.sin(), -angle.cos()));
    }
    points
}

/// A long rectangle with its corners cut off.
fn emerald_outline() -> Vec<Vec2> {
    const WIDE: f32 = 0.68;
    const LONG: f32 = 1.0;
    const CORNER: f32 = 0.2;
    vec![
        Vec2::new(-WIDE + CORNER, -LONG),
        Vec2::new(WIDE - CORNER, -LONG),
        Vec2::new(WIDE, -LONG + CORNER),
        Vec2::new(WIDE, LONG - CORNER),
        Vec2::new(WIDE - CORNER, LONG),
        Vec2::new(-WIDE + CORNER, LONG),
        Vec2::new(-WIDE, LONG - CORNER),
        Vec2::new(-WIDE, -LONG + CORNER),
    ]
}

/// Triangles with their own flat faces, each a little lighter or darker.
struct Facets {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colours: Vec<[f32; 4]>,
    /// How far from white each face's shade may go.
    variation: f32,
    shades: shared::rng::Rng,
}

impl Facets {
    fn new(seed: u64, variation: f32) -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            colours: Vec::new(),
            variation,
            shades: Seed::new(seed).rng(),
        }
    }

    /// A face, turned to look away from the middle, as the whole is convex.
    fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3) {
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

    fn into_mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colours)
    }
}

fn at(point: Vec2, scale: f32, height: f32) -> Vec3 {
    Vec3::new(point.x * scale, height, point.y * scale)
}

fn cut_mesh(cut: Cut, variation: f32) -> Mesh {
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
fn brilliant(facets: &mut Facets, outline: &[Vec2]) {
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
fn stepped(facets: &mut Facets, outline: &[Vec2]) {
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

/// A sphere pushed in and out a different way for each seed. Its faces shade only a
/// little: the lumps do the work.
fn uncut_mesh(shape_seed: u16) -> Mesh {
    let seed = Seed::new(shape_seed as u64).derive("uncut gem");
    let sphere = Sphere::new(1.0)
        .mesh()
        .ico(2)
        .expect("2 is few enough subdivisions");
    let points = sphere
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(|positions| positions.as_float3())
        .expect("a sphere has positions");
    let number = |purpose: i64, from: f32, to: f32| {
        let bits = seed.at(&[purpose]) >> 40;
        from + (to - from) * bits as f32 / (1u64 << 24) as f32
    };
    // Squashed along each axis a different amount, then lumps at each point.
    let squash = Vec3::new(
        number(-1, 0.8, 1.0),
        number(-2, 0.65, 0.9),
        number(-3, 0.8, 1.0),
    );
    let points: Vec<Vec3> = points
        .iter()
        .enumerate()
        .map(|(i, &point)| Vec3::from(point) * number(i as i64, 0.8, 1.08) * squash)
        .collect();
    let mut facets = Facets::new(shape_seed as u64 + 1, 0.05);
    let corners: Vec<usize> = sphere
        .indices()
        .map_or(Vec::new(), |indices| indices.iter().collect());
    for &[a, b, c] in corners.as_chunks::<3>().0 {
        facets.triangle(points[a], points[b], points[c]);
    }
    facets.into_mesh()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUTS: [Cut; 7] = [
        Cut::Cushion,
        Cut::Emerald,
        Cut::Oval,
        Cut::Pear,
        Cut::Princess,
        Cut::Round,
        Cut::Trillion,
    ];

    fn turn(a: Vec2, b: Vec2, c: Vec2) -> f32 {
        (b - a).perp_dot(c - b)
    }

    #[test]
    fn every_outline_goes_one_way_round_and_never_turns_back() {
        for cut in CUTS {
            let points = outline(cut);
            for i in 0..points.len() {
                let (a, b) = (points[i], points[(i + 1) % points.len()]);
                let c = points[(i + 2) % points.len()];
                assert!(turn(a, b, c) > -1e-4, "{cut:?} at {i}");
            }
        }
    }

    #[test]
    fn every_outline_fits_the_picture() {
        for cut in CUTS {
            for point in outline(cut) {
                assert!(point.length() <= 1.2, "{cut:?} {point}");
            }
        }
    }

    #[test]
    fn a_trillion_has_a_corner_at_the_top_and_a_pear_its_point() {
        assert!(outline(Cut::Trillion)[0].y < -0.95);
        assert_eq!(outline(Cut::Pear)[0], Vec2::new(0.0, -1.0));
        assert_eq!(TRILLION_POINTS % 3, 0);
    }

    #[test]
    fn stones_of_different_seeds_have_different_lumps() {
        let positions = |seed| {
            let mesh = uncut_mesh(seed);
            let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
            points.as_float3().unwrap().to_vec()
        };
        assert_eq!(positions(1), positions(1));
        assert_ne!(positions(1), positions(2));
    }
}
