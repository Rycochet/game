//! The band: its cross-section by profile and width, and its mesh.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::item_form::{BandWidth, Pattern, Profile};
use shared::rng::Seed;
use std::f32::consts::{FRAC_PI_2, TAU};

use super::super::RingLook;
use super::{BAND_RADIUS, BAND_STEPS, BAND_THICKNESS, ROUND_STEPS, TWISTS};

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

pub(super) fn band_mesh(ring: &RingLook) -> Mesh {
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
