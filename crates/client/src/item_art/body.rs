//! A plain body to dress, built from its frame in millimetres: everything worn is made in
//! place on the same frame, so it is drawn on this. Smooth shapes, a little turned and
//! seen slightly from above, as a token shows a character.

use std::f32::consts::{PI, TAU};

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::body::{Hair, Layout, Role};

use super::colours::{Colours, material};
use super::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

mod arms;
mod coat_hair;
mod extras;
mod features;
mod fit;
mod hair;
mod head;
mod legs;
mod morph;
mod trunk;

pub(super) use coat_hair::coat_hair;
pub(super) use features::wings;
use fit::fitted;
pub(super) use fit::turn_of;
pub(super) use hair::{Cover, hair};
pub(super) use head::{HeadAxes, crown};
#[cfg(test)]
pub(super) use morph::placed as morph_placed;
pub(super) use morph::{morph_parts, pieces as bare_pieces};
#[cfg(test)]
pub(super) use trunk::Girth;
pub(super) use trunk::{neck_root, trunk_depth, trunk_half};

/// What an upright body's picture takes up, wide and long: only the proportions
/// matter. An animal's is square, as its token is.
pub(super) const CELLS: (u8, u8) = (1, 2);
const BEAST_CELLS: (u8, u8) = (2, 2);

/// What a body's picture takes up.
pub(super) fn cells(layout: &Layout) -> (u8, u8) {
    if layout.frame().is_some() {
        CELLS
    } else {
        BEAST_CELLS
    }
}
/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Turned a little to its right, so its sides show.
const TURN: f32 = 0.35;
/// Seen a little from above, about as a token is.
const TILT: f32 = 0.25;
/// Points round anything round, and rings along a head.
pub(super) const ROUND: usize = 12;
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

/// The body, in millimetres, standing as its layout says, with whatever it has beyond
/// a person's.
pub(super) fn build(layout: &Layout) -> Vec<Triangle> {
    let mut triangles = Vec::new();
    trunk::build(&mut triangles, layout);
    neck_and_head(&mut triangles, layout);
    limbs(&mut triangles, layout);
    features::rear(&mut triangles, layout);
    triangles
}

/// Each limb, by its role.
fn limbs(triangles: &mut Vec<Triangle>, layout: &Layout) {
    for limb in layout.limbs() {
        match limb.role {
            Role::Arm | Role::WingArm => arms::build(triangles, layout, limb),
            Role::Leg | Role::Foreleg | Role::Hindleg | Role::JointedLeg => {
                legs::build(triangles, layout, limb, layout.parts.feet)
            }
        }
    }
}

/// The neck from the top of the spine, and the head on it with its snout, ears, horns
/// and tusks.
fn neck_and_head(triangles: &mut Vec<Triangle>, layout: &Layout) {
    // A measured head: an animal's on a neck carrying on from its trunk, a beast
    // people's on a person's neck, which neck armour is made to fit.
    if let Some(shape) = layout.head.shape {
        if layout.frame().is_some() {
            neck(triangles, layout);
        } else {
            head::neck(triangles, layout);
        }
        head::build(triangles, layout, &shape);
        extras::build(triangles, layout);
        return;
    }
    neck(triangles, layout);
    let head = layout.head;
    blob(
        triangles,
        features::skull(layout),
        Vec3::new(
            head.half.0 as f32,
            head.length as f32 / 2.0,
            head.half.1 as f32,
        ),
    );
    features::head(triangles, layout, &layout.parts);
}

/// A person's neck: round, from a little into the trunk to a little into the head.
fn neck(triangles: &mut Vec<Triangle>, layout: &Layout) {
    let axes = trunk::Axes::of(layout);
    let neck_base = trunk::neck_root(layout);
    let chin = point(layout.chin());
    let up_neck = (chin - neck_base).normalize_or(Vec3::Y);
    let neck = layout.neck.radius as f32;
    limb(
        triangles,
        &[
            (neck_base - axes.along * 20.0, neck),
            (chin + up_neck * 20.0, neck),
        ],
    );
}

/// The body's skin, and apart from it the coat of a taur's lower body, which may be
/// another colour.
pub(super) fn build_coated(layout: &Layout) -> (Vec<Triangle>, Vec<Triangle>) {
    let Some(upper) = layout.upper_body() else {
        return (build(layout), Vec::new());
    };
    // The lower body: trunk, legs and tail, its neck and head left to the upper body,
    // unless it still has one of its own, as when shapeshifting into or out of a taur.
    let mut coat = Vec::new();
    trunk::build(&mut coat, layout);
    if layout.head.length > 0 {
        neck_and_head(&mut coat, layout);
    }
    limbs(&mut coat, layout);
    features::rear(&mut coat, layout);
    (build(&upper), coat)
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

/// A bare body standing as this layout says, as tall as `height` beside a person, in
/// these colours, with this hair.
pub(super) fn body_parts(
    layout: &Layout,
    height: i32,
    colours: Colours,
    hair: Hair,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    // The picture's plane faces the camera, as weapons' do.
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let [skin, coat, feathers, hair, mane] = fitted(layout, height, hair);
    [
        (skin, colours.skin),
        (coat, colours.coat),
        (feathers, colours.feathers),
        (hair, colours.hair),
        (mane, colours.mane),
    ]
    .into_iter()
    .filter(|(triangles, _)| !triangles.is_empty())
    .map(|(triangles, colour)| Part {
        mesh: meshes.add(mesh_of(&triangles)),
        material: materials.add(material(colour)),
        transform: Transform::from_rotation(camera.rotation),
    })
    .collect()
}

#[cfg(test)]
mod tests;
