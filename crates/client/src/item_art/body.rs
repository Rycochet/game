//! A plain body to dress, built from its frame in millimetres: everything worn is made in
//! place on the same frame, so it is drawn on this. Smooth shapes, a little turned and
//! seen slightly from above, as a token shows a character.

use std::f32::consts::{PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::body::{Frame, Joint, Parts, Side};

use super::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

mod features;
mod fit;
mod legs;

use fit::fitted;

/// What a body's picture takes up, wide and long: only the proportions matter.
pub(super) const CELLS: (u8, u8) = (1, 2);
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned a little to its right, so its sides show.
const TURN: f32 = 0.35;
/// Seen a little from above, about as a token is.
const TILT: f32 = 0.25;
/// Points round anything round, and rings along a head.
const ROUND: usize = 12;
const HEAD_RINGS: usize = 8;

pub(super) type Triangle = [Vec3; 3];

fn point(mm: [i32; 3]) -> Vec3 {
    Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32)
}

/// Skin over rings of equally many points, with the ends closed.
pub(super) fn loft(triangles: &mut Vec<Triangle>, rings: &[Vec<Vec3>]) {
    let middle = |ring: &[Vec3]| ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
    let mut face = |a: Vec3, b: Vec3, c: Vec3, out: Vec3| {
        let forward = (b - a).cross(c - a).dot(out) >= 0.0;
        triangles.push(if forward { [a, b, c] } else { [a, c, b] });
    };
    for pair in rings.windows(2) {
        let centre = (middle(&pair[0]) + middle(&pair[1])) / 2.0;
        let points = pair[0].len();
        for at in 0..points {
            let next = (at + 1) % points;
            let corners = [pair[0][at], pair[0][next], pair[1][next], pair[1][at]];
            let out = corners.iter().copied().sum::<Vec3>() / 4.0 - centre;
            face(corners[0], corners[1], corners[2], out);
            face(corners[0], corners[2], corners[3], out);
        }
    }
    for (end, inner) in [(0, 1), (rings.len() - 1, rings.len() - 2)] {
        let centre = middle(&rings[end]);
        let out = centre - middle(&rings[inner]);
        for at in 0..rings[end].len() {
            let next = (at + 1) % rings[end].len();
            face(centre, rings[end][at], rings[end][next], out);
        }
    }
}

/// A flat ring round a level, half widths across and front to back.
fn level(y: f32, (across, deep): (f32, f32), forward: f32) -> Vec<Vec3> {
    (0..ROUND)
        .map(|at| {
            let angle = at as f32 / ROUND as f32 * TAU;
            Vec3::new(across * angle.cos(), y, forward + deep * angle.sin())
        })
        .collect()
}

/// A round limb from one point to another, its radius changing along it.
pub(super) fn limb(triangles: &mut Vec<Triangle>, path: &[(Vec3, f32)]) {
    let rings: Vec<Vec<Vec3>> = (0..path.len())
        .map(|at| {
            let (place, radius) = path[at];
            let along = (path[(at + 1).min(path.len() - 1)].0 - path[at.saturating_sub(1)].0)
                .normalize_or(Vec3::Y);
            // Measured the same way at every ring, so the skin doesn't twist.
            let across = along.cross(Vec3::Z).normalize_or(Vec3::X);
            let other = along.cross(across);
            (0..ROUND)
                .map(|point| {
                    let angle = point as f32 / ROUND as f32 * TAU;
                    place + (across * angle.cos() + other * angle.sin()) * radius
                })
                .collect()
        })
        .collect();
    loft(triangles, &rings);
}

/// A ball stretched to these half sizes.
pub(super) fn blob(triangles: &mut Vec<Triangle>, centre: Vec3, half: Vec3) {
    let rings: Vec<Vec<Vec3>> = (0..=HEAD_RINGS)
        .map(|ring| {
            let down = ring as f32 / HEAD_RINGS as f32 * PI;
            let y = centre.y + half.y * down.cos();
            let shrink = down.sin().max(0.02);
            level(y, (half.x * shrink, half.z * shrink), centre.z)
                .into_iter()
                .map(|p| Vec3::new(centre.x + p.x, p.y, p.z))
                .collect()
        })
        .collect();
    loft(triangles, &rings);
}

/// The trunk's levels from the crotch up to the neck: each height, with its half widths
/// across and front to back.
fn trunk_levels(frame: &Frame) -> [(f32, (f32, f32)); 6] {
    let f = |value: i32| value as f32;
    let half = |(across, deep): (i32, i32)| (f(across), f(deep));
    [
        (f(frame.hip) - f(frame.thigh), half(frame.hips_half)),
        (f(frame.hip), half(frame.hips_half)),
        (f(frame.waist), half(frame.waist_half)),
        (f(frame.chest), half(frame.chest_half)),
        (
            f(frame.shoulder),
            (f(frame.shoulder_half), f(frame.chest_half.1) * 0.7),
        ),
        (
            f(frame.neck_base),
            (f(frame.neck) * 1.6, f(frame.neck) * 1.2),
        ),
    ]
}

/// The trunk's half widths across and front to back at a height, as it is drawn: what
/// armour on it is offset from.
pub(super) fn trunk_half(frame: &Frame, y: f32) -> (f32, f32) {
    let levels = trunk_levels(frame);
    let (first, last) = (levels[0], levels[levels.len() - 1]);
    if y <= first.0 {
        return first.1;
    }
    if y >= last.0 {
        return last.1;
    }
    let at = levels
        .windows(2)
        .find(|pair| y <= pair[1].0)
        .expect("between the ends");
    let t = (y - at[0].0) / (at[1].0 - at[0].0);
    (
        at[0].1.0 + (at[1].1.0 - at[0].1.0) * t,
        at[0].1.1 + (at[1].1.1 - at[0].1.1) * t,
    )
}

/// How far in front of the middle (or behind it) the trunk's skin is at this height and
/// this far across: 0 past its side.
pub(super) fn trunk_depth(frame: &Frame, x: f32, y: f32) -> f32 {
    let (across, deep) = trunk_half(frame, y);
    deep * (1.0 - (x / across).powi(2)).max(0.0).sqrt()
}

/// The body, in millimetres, standing as its frame says, with whatever it has beyond a
/// person's.
pub(super) fn build(frame: &Frame, parts: &Parts) -> Vec<Triangle> {
    let f = |value: i32| value as f32;
    let mut triangles = Vec::new();
    // The trunk, from the crotch up to where the arms join.
    let trunk: Vec<_> = trunk_levels(frame)
        .into_iter()
        .map(|(y, halves)| level(y, halves, 0.0))
        .collect();
    loft(&mut triangles, &trunk);
    let neck = f(frame.neck);
    limb(
        &mut triangles,
        &[
            (Vec3::new(0.0, f(frame.neck_base) - 20.0, 0.0), neck),
            (Vec3::new(0.0, f(frame.chin) + 20.0, 0.0), neck),
        ],
    );
    let skull = point(frame.joint(Joint::Skull));
    let head_half = Vec3::new(
        f(frame.head.0),
        f(frame.height - frame.chin) / 2.0,
        f(frame.head.1),
    );
    blob(&mut triangles, skull, head_half);
    for side in Side::BOTH {
        let joint = |joint: Joint| point(frame.joint(joint));
        let (upper, fore) = (f(frame.upper_arm), f(frame.forearm));
        limb(
            &mut triangles,
            &[
                (joint(Joint::Shoulder(side)), upper),
                (joint(Joint::Elbow(side)), fore * 1.05),
                (joint(Joint::Wrist(side)), fore * 0.7),
            ],
        );
        blob(
            &mut triangles,
            joint(Joint::Wrist(side)).lerp(joint(Joint::Fingertips(side)), 0.5),
            Vec3::new(fore * 0.45, f(frame.hand_length) / 2.0, fore * 1.1),
        );
        blob(
            &mut triangles,
            joint(Joint::Shoulder(side)),
            Vec3::splat(upper * 1.15),
        );
        legs::build(&mut triangles, frame, parts.feet, side);
    }
    features::build(&mut triangles, frame, parts);
    triangles
}

/// A flat-faced mesh of these triangles, as weapons' are.
pub(super) fn mesh_of(triangles: &[Triangle]) -> Mesh {
    let mut positions = Vec::with_capacity(triangles.len() * 3);
    let mut normals = Vec::with_capacity(triangles.len() * 3);
    for [a, b, c] in triangles {
        let normal = (*b - *a).cross(*c - *a).normalize_or(Vec3::Z).to_array();
        for corner in [a, b, c] {
            positions.push(corner.to_array());
            normals.push(normal);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
}

/// A bare body standing on this frame, with these parts, as tall as `height` beside a
/// person.
pub(super) fn body_parts(
    frame: &Frame,
    parts: &Parts,
    height: i32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    // The picture's plane faces the camera, as weapons' do.
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let (skin, feathers) = fitted(frame, parts, height);
    // Plain, as a tailor's dummy: skin comes with looks. Feathers white, so wings show
    // apart from the arms.
    [
        (skin, Color::srgb(0.78, 0.72, 0.64)),
        (feathers, Color::srgb(0.95, 0.95, 0.93)),
    ]
    .into_iter()
    .filter(|(triangles, _)| !triangles.is_empty())
    .map(|(triangles, colour)| Part {
        mesh: meshes.add(mesh_of(&triangles)),
        material: materials.add(StandardMaterial {
            base_color: colour,
            perceptual_roughness: 0.8,
            ..default()
        }),
        transform: Transform::from_rotation(camera.rotation),
    })
    .collect()
}

#[cfg(test)]
mod tests;
