//! A ring's model: a cross-section swept round a circle, with its settings and gems.
//! It is built standing up, facing the camera, then all tipped back together so that
//! the gems at the top show their faces.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::item::Quality;
use shared::item_form::{BandWidth, Mount, Pattern, Profile};
use shared::material::Material;
use shared::rng::Seed;

use super::gem::{PAVILION_DEPTH, gem_parts};
use super::{CAMERA_AT, CAMERA_UP, Part, RingLook};

/// From the middle of the ring to the middle of its band.
const BAND_RADIUS: f32 = 0.82;
/// How thick the band is, from its middle to its outside.
const BAND_THICKNESS: f32 = 0.05;
/// Steps round the band.
const BAND_STEPS: usize = 72;
/// Steps round a round cross-section.
const ROUND_STEPS: usize = 10;
/// How much it is tipped back, and turned, so it is seen as a ring and not a circle.
const TIP: f32 = 0.75;
const TURN: f32 = 0.35;
/// How many times a twisted band's cross-section turns on its way round.
const TWISTS: f32 = 14.0;
/// How wide a gem is, by how many settings share the band.
const GEM_ALONE: f32 = 0.32;
const GEM_OF_FEW: f32 = 0.21;
const GEM_OF_MANY: f32 = 0.16;
/// How much bigger each size up is drawn.
const SIZE_STEP: f32 = 1.2;
/// How thick a claw is, against its gem.
const CLAW: f32 = 0.09;

/// A ring's parts, all placed.
pub(super) fn ring_parts(
    ring: &RingLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let facing = whole_ring();
    let metal = materials.add(metal_material(ring.metal, quality));
    let mut parts = vec![Part {
        mesh: meshes.add(band_mesh(ring)),
        material: metal.clone(),
        transform: facing,
    }];
    let room = BAND_RADIUS + BAND_THICKNESS;
    let stem = meshes.add(Cylinder::new(1.0, 1.0).mesh());
    for setting in &ring.settings {
        let angle = setting.angle as f32 / 256.0 * TAU;
        let out = Vec3::new(angle.sin(), angle.cos(), 0.0);
        let turned = Quat::from_rotation_arc(Vec3::Y, out);
        let width = setting_width(ring.settings.len()) * size_scale(setting.bigger);
        let sits = room + width * PAVILION_DEPTH * 0.6;
        let centre = out * sits;
        let mut place = |local: Transform, material: Handle<StandardMaterial>| {
            let transform = facing
                * Transform::from_translation(out * room)
                * Transform::from_rotation(turned)
                * local;
            parts.push(Part {
                mesh: stem.clone(),
                material,
                transform,
            });
        };
        let height = sits - room;
        match setting.mount {
            Mount::Bezel => place(
                Transform::from_xyz(0.0, height / 2.0, 0.0).with_scale(Vec3::new(
                    width * 1.05,
                    height + 0.01,
                    width * 1.05,
                )),
                metal.clone(),
            ),
            Mount::Claws => {
                for corner in 0..4 {
                    let around = (corner as f32 + 0.5) / 4.0 * TAU;
                    let at = Vec3::new(around.cos(), 0.0, around.sin());
                    let from = at * width * 0.25;
                    let to = at * width * 0.95 + Vec3::Y * (height + width * 0.12);
                    place(rod(from, to, width * CLAW), metal.clone());
                }
            }
        }
        // Where nothing sits, the dark of the empty seat shows.
        let Some(gem) = &setting.gem else {
            let dark = materials.add(StandardMaterial {
                base_color: Color::srgb(0.05, 0.05, 0.06),
                perceptual_roughness: 1.0,
                ..default()
            });
            place(
                Transform::from_xyz(0.0, height + 0.012, 0.0).with_scale(Vec3::new(
                    width * 0.8,
                    0.01,
                    width * 0.8,
                )),
                dark,
            );
            continue;
        };
        let scale = width * size_scale(gem.bigger);
        let transform = facing
            * Transform::from_translation(centre)
                .with_rotation(turned)
                .with_scale(Vec3::splat(scale));
        for (mesh, material) in gem_parts(&gem.gem, gem.quality, meshes, materials) {
            parts.push(Part {
                mesh,
                material,
                transform,
            });
        }
    }
    parts
}

/// How the whole ring lies, facing the camera then tipped back.
fn whole_ring() -> Transform {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let tipped = Quat::from_rotation_x(TIP) * Quat::from_rotation_y(TURN);
    Transform::from_rotation(camera.rotation * tipped)
}

fn setting_width(settings: usize) -> f32 {
    match settings {
        0 | 1 => GEM_ALONE,
        2 | 3 => GEM_OF_FEW,
        _ => GEM_OF_MANY,
    }
}

fn size_scale(bigger: i8) -> f32 {
    SIZE_STEP.powi(bigger as i32)
}

/// A unit cylinder placed as a rod between two points.
fn rod(from: Vec3, to: Vec3, radius: f32) -> Transform {
    let along = to - from;
    Transform::from_translation((from + to) / 2.0)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, along.normalize()))
        .with_scale(Vec3::new(radius, along.length(), radius))
}

pub(super) fn metal_material(metal: Material, quality: Option<Quality>) -> StandardMaterial {
    let fineness = quality.unwrap_or_default() as usize as f32 / 6.0;
    StandardMaterial {
        base_color: match metal {
            Material::Adamantine => Color::srgb(0.34, 0.31, 0.42),
            Material::Brass => Color::srgb(0.86, 0.7, 0.36),
            Material::Bronze => Color::srgb(0.72, 0.5, 0.26),
            Material::Copper => Color::srgb(0.85, 0.5, 0.3),
            Material::Iron => Color::srgb(0.48, 0.49, 0.52),
            Material::Mithril => Color::srgb(0.82, 0.88, 0.96),
            Material::Silver => Color::srgb(0.88, 0.89, 0.92),
            Material::Steel => Color::srgb(0.64, 0.66, 0.7),
            _ => Color::srgb(1.0, 0.76, 0.3),
        },
        // Fully metal shows only what it reflects, and there is nothing round it.
        metallic: 0.5,
        perceptual_roughness: 0.5 - 0.3 * fineness,
        reflectance: 0.8,
        ..default()
    }
}

/// Round the cross-section, in (outward, across the band), one way round.
fn cross_section(profile: Profile, width: BandWidth) -> Vec<Vec2> {
    let (thick, wide) = (
        BAND_THICKNESS,
        match width {
            BandWidth::Thin => 0.07,
            BandWidth::Medium => 0.11,
            BandWidth::Wide => 0.19,
        },
    );
    let arc = |from: f32, to: f32, steps: usize, radii: Vec2| -> Vec<Vec2> {
        (0..=steps)
            .map(|step| {
                let angle = from + (to - from) * step as f32 / steps as f32;
                Vec2::new(angle.cos(), angle.sin()) * radii
            })
            .collect()
    };
    match profile {
        Profile::Round => {
            let mut round = arc(0.0, TAU, ROUND_STEPS, Vec2::new(thick * 1.3, wide));
            round.pop();
            round
        }
        Profile::Domed => {
            let mut dome = arc(
                -FRAC_PI_2,
                FRAC_PI_2,
                ROUND_STEPS,
                Vec2::new(thick * 2.0, wide),
            );
            for point in &mut dome {
                point.x -= thick;
            }
            dome
        }
        Profile::Flat => vec![
            Vec2::new(thick * 0.8, -wide),
            Vec2::new(thick * 0.8, wide),
            Vec2::new(-thick * 0.8, wide),
            Vec2::new(-thick * 0.8, -wide),
        ],
        Profile::Square => {
            let side = thick * 1.3;
            vec![
                Vec2::new(side, -side),
                Vec2::new(side, side),
                Vec2::new(-side, side),
                Vec2::new(-side, -side),
            ]
        }
        Profile::Knife => vec![
            Vec2::new(thick * 2.0, 0.0),
            Vec2::new(0.0, wide),
            Vec2::new(-thick, wide * 0.7),
            Vec2::new(-thick, -wide * 0.7),
            Vec2::new(0.0, -wide),
        ],
    }
}

fn band_mesh(ring: &RingLook) -> Mesh {
    let section = cross_section(ring.profile, ring.width);
    let middle = section.iter().copied().sum::<Vec2>() / section.len() as f32;
    let twist = |step: usize| match ring.pattern {
        Pattern::Twisted => Mat2::from_angle(step as f32 / BAND_STEPS as f32 * TAU * TWISTS),
        _ => Mat2::IDENTITY,
    };
    let round = |step: usize| {
        let angle = step as f32 / BAND_STEPS as f32 * TAU;
        Vec3::new(angle.sin(), angle.cos(), 0.0)
    };
    let dents = Seed::new(ring.metal as u64).derive("hammered");
    let (mut positions, mut normals, mut colours) = (Vec::new(), Vec::new(), Vec::new());
    for step in 0..BAND_STEPS {
        for edge in 0..section.len() {
            let (from, to) = (section[edge], section[(edge + 1) % section.len()]);
            let corner = |step: usize, point: Vec2| {
                let point = twist(step) * (point - middle) + middle;
                round(step) * (BAND_RADIUS + point.x) + Vec3::Z * point.y
            };
            let quad = [
                corner(step, from),
                corner(step + 1, from),
                corner(step + 1, to),
                corner(step, to),
            ];
            let side = to - from;
            let mut out = Vec2::new(side.y, -side.x).normalize_or_zero();
            if out.dot((from + to) / 2.0 - middle) < 0.0 {
                out = -out;
            }
            let normal_at = |step: usize, dent: Vec3| {
                let out = twist(step) * out;
                (round(step) * out.x + Vec3::Z * out.y + dent).normalize_or_zero()
            };
            let hash =
                |which: i64| dents.at(&[step as i64, edge as i64, which]) as f32 / u64::MAX as f32;
            let dent = match ring.pattern {
                Pattern::Hammered => Vec3::new(hash(0), hash(1), hash(2)) * 0.7 - Vec3::splat(0.35),
                _ => Vec3::ZERO,
            };
            let shade = match ring.pattern {
                Pattern::Engraved if out.x > 0.3 && (step % 6 == 0 || step % 6 == 1) => 0.3,
                _ => 1.0,
            };
            let (first, next) = (normal_at(step, dent), normal_at(step + 1, dent));
            let face = (quad[1] - quad[0]).cross(quad[2] - quad[0]);
            let order = if face.dot(first) >= 0.0 {
                [0, 1, 2, 0, 2, 3]
            } else {
                [0, 2, 1, 0, 3, 2]
            };
            for index in order {
                positions.push(quad[index].to_array());
                let normal = if index == 0 || index == 3 {
                    first
                } else {
                    next
                };
                normals.push(normal.to_array());
                colours.push([shade, shade, shade, 1.0]);
            }
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item_art::VIEW_WIDTH;

    fn ring(profile: Profile, width: BandWidth, pattern: Pattern) -> RingLook {
        RingLook {
            metal: Material::Gold,
            profile,
            width,
            pattern,
            settings: Vec::new(),
        }
    }

    const PROFILES: [Profile; 5] = [
        Profile::Domed,
        Profile::Flat,
        Profile::Knife,
        Profile::Round,
        Profile::Square,
    ];
    const PATTERNS: [Pattern; 4] = [
        Pattern::Engraved,
        Pattern::Hammered,
        Pattern::Plain,
        Pattern::Twisted,
    ];

    #[test]
    fn every_band_stays_in_the_picture_and_has_faces() {
        for profile in PROFILES {
            for pattern in PATTERNS {
                let mesh = band_mesh(&ring(profile, BandWidth::Wide, pattern));
                let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
                let points = points.as_float3().unwrap();
                assert!(points.len() >= BAND_STEPS * 3);
                for &point in points {
                    let reach = Vec3::from(point).length();
                    assert!(reach < VIEW_WIDTH / 2.0, "{profile:?} {pattern:?} {reach}");
                }
            }
        }
    }

    #[test]
    fn a_face_of_the_band_looks_the_way_its_normal_says() {
        let mesh = band_mesh(&ring(Profile::Square, BandWidth::Medium, Pattern::Plain));
        let points = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let points = points.as_float3().unwrap();
        let normals = mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap();
        let normals = normals.as_float3().unwrap();
        for (corners, normals) in points.chunks(3).zip(normals.chunks(3)) {
            let (a, b, c) = (
                Vec3::from(corners[0]),
                Vec3::from(corners[1]),
                Vec3::from(corners[2]),
            );
            assert!((b - a).cross(c - a).dot(Vec3::from(normals[0])) > 0.0);
        }
    }

    #[test]
    fn a_rod_goes_between_its_two_points() {
        let (from, to) = (Vec3::new(0.1, 0.0, 0.2), Vec3::new(0.4, 0.5, 0.0));
        let placed = rod(from, to, 0.02);
        let top = placed.transform_point(Vec3::Y * 0.5);
        let bottom = placed.transform_point(Vec3::NEG_Y * 0.5);
        assert!(top.distance(to) < 1e-4 && bottom.distance(from) < 1e-4);
    }

    #[test]
    fn larger_sizes_draw_larger() {
        assert!(size_scale(1) > size_scale(0) && size_scale(0) > size_scale(-1));
    }
}
