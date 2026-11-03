//! A weapon's model, built from its rolled form. It is first made in millimetres, with
//! the point up and any edge to the right, then fitted to the cells it takes up. Lengths
//! are squeezed to fit but widths much less, or a pike would be a black line.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use shared::blade::{
    Back, BladeForm, BladeShape, BladeWidth, Build, Curve, Decoration, Edges, Guard, HandleShape,
    Point, Pommel, Ricasso, Section, SidePart, Tips, Top,
};
use shared::item::Quality;
use shared::material::Material;

use super::ring::metal_material;
use super::{BladeLook, CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, when drawn for a grid.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round a model, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// How many millimetres across one cell shows at most, whatever its length shows.
const CELL_ACROSS_MM: f32 = 150.0;
/// How far it is turned along its length, so that its faces catch the light differently.
const TURN: f32 = 0.35;
/// Half a handle's width, a little over 3 cm across as its weight assumes.
const HANDLE_RADIUS: f32 = 15.0;
/// Sides of anything round.
const ROUND_SIDES: usize = 8;
/// Steps along a blade: more for one that keeps changing.
const BLADE_STEPS: usize = 28;
const WAVY_STEPS: usize = 56;
/// Steps along a handle.
const HANDLE_STEPS: usize = 12;

/// What each piece is made of, as one mesh is made for each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stuff {
    Blade,
    Handle,
    Fittings,
    /// Fullers and engraving, darker than the blade.
    Dark,
    Inlay,
    Cloth,
}

const STUFFS: [Stuff; 6] = [
    Stuff::Blade,
    Stuff::Handle,
    Stuff::Fittings,
    Stuff::Dark,
    Stuff::Inlay,
    Stuff::Cloth,
];

type Triangle = [Vec3; 3];

/// Triangles for each stuff, each kept facing the way it should.
#[derive(Default)]
struct Model {
    triangles: [Vec<Triangle>; STUFFS.len()],
}

impl Model {
    fn face(&mut self, stuff: Stuff, a: Vec3, b: Vec3, c: Vec3, out: Vec3) {
        let forward = (b - a).cross(c - a).dot(out) >= 0.0;
        let triangle = if forward { [a, b, c] } else { [a, c, b] };
        self.triangles[stuff as usize].push(triangle);
    }

    fn quad(&mut self, stuff: Stuff, corners: [Vec3; 4], out: Vec3) {
        let [a, b, c, d] = corners;
        self.face(stuff, a, b, c, out);
        self.face(stuff, a, c, d, out);
    }

    /// Skin over rings of equally many points, with the ends closed.
    fn loft(&mut self, stuff: Stuff, rings: &[Vec<Vec3>]) {
        let middle = |ring: &[Vec3]| ring.iter().copied().sum::<Vec3>() / ring.len() as f32;
        for pair in rings.windows(2) {
            let centre = (middle(&pair[0]) + middle(&pair[1])) / 2.0;
            let points = pair[0].len();
            for at in 0..points {
                let next = (at + 1) % points;
                let corners = [pair[0][at], pair[0][next], pair[1][next], pair[1][at]];
                let side = corners.iter().copied().sum::<Vec3>() / 4.0;
                self.quad(stuff, corners, side - centre);
            }
        }
        let ends = [
            (rings.first(), rings.get(1)),
            (
                rings.last(),
                rings.len().checked_sub(2).map(|at| &rings[at]),
            ),
        ];
        for (end, inner) in ends {
            let (Some(end), Some(inner)) = (end, inner) else {
                continue;
            };
            let centre = middle(end);
            let out = centre - middle(inner);
            for at in 0..end.len() {
                let next = (at + 1) % end.len();
                self.face(stuff, centre, end[at], end[next], out);
            }
        }
    }

    /// A round thing along a line in the picture's plane: places and radii along it.
    fn tube(&mut self, stuff: Stuff, path: &[(Vec2, f32)], depth: f32) {
        let last = path.len().saturating_sub(1);
        let rings: Vec<Vec<Vec3>> = (0..path.len())
            .map(|at| {
                let (place, radius) = path[at];
                let along = (path[(at + 1).min(last)].0 - path[at.saturating_sub(1)].0)
                    .normalize_or(Vec2::Y);
                let across = Vec2::new(-along.y, along.x);
                (0..ROUND_SIDES)
                    .map(|side| {
                        let angle = side as f32 / ROUND_SIDES as f32 * std::f32::consts::TAU;
                        let flat = place + across * angle.cos() * radius;
                        flat.extend(depth + angle.sin() * radius)
                    })
                    .collect()
            })
            .collect();
        self.loft(stuff, &rings);
    }

    /// A straight tapering point.
    fn spike(&mut self, stuff: Stuff, from: Vec2, to: Vec2, radius: f32) {
        self.tube(stuff, &[(from, radius), (to, 0.3)], 0.0);
    }

    /// A round plate facing the camera.
    fn disc(&mut self, stuff: Stuff, centre: Vec2, radius: f32, thick: f32) {
        let ring = |z: f32| -> Vec<Vec3> {
            (0..16)
                .map(|side| {
                    let angle = side as f32 / 16.0 * std::f32::consts::TAU;
                    (centre + Vec2::from_angle(angle) * radius).extend(z)
                })
                .collect()
        };
        self.loft(stuff, &[ring(-thick), ring(thick)]);
    }

    /// A strip lying on both faces at this depth, from `from` to `to`.
    fn ribbon(&mut self, stuff: Stuff, from: Vec2, to: Vec2, half: f32, depth: f32) {
        let across = (to - from).normalize_or(Vec2::Y).perp() * half;
        for side in [depth, -depth] {
            let corners = [from - across, from + across, to + across, to - across];
            self.quad(
                stuff,
                corners.map(|corner| corner.extend(side)),
                Vec3::Z * side,
            );
        }
    }
}

fn half_width(form: &BladeForm) -> f32 {
    match form.width {
        BladeWidth::Narrow => 12.0,
        BladeWidth::Normal => 22.0,
        BladeWidth::Broad => 32.0,
    }
}

/// How thick a blade is at its middle, against its full width.
fn thickness(half: f32) -> f32 {
    (half * 0.22).max(2.5)
}

/// Round a blade cut across: from its back at -1 to its edge at 1, thickness from -1 to 1.
fn section_points(form: &BladeForm) -> Vec<Vec2> {
    let points: &[(f32, f32)] = if form.edges == Edges::Single {
        &[
            (1.0, 0.0),
            (0.0, 0.9),
            (-1.0, 0.7),
            (-1.0, -0.7),
            (0.0, -0.9),
        ]
    } else {
        match form.section {
            Section::Flat => &[
                (1.0, 0.0),
                (0.6, 0.6),
                (-0.6, 0.6),
                (-1.0, 0.0),
                (-0.6, -0.6),
                (0.6, -0.6),
            ],
            Section::Lenticular => &[
                (1.0, 0.0),
                (0.5, 0.75),
                (0.0, 1.0),
                (-0.5, 0.75),
                (-1.0, 0.0),
                (-0.5, -0.75),
                (0.0, -1.0),
                (0.5, -0.75),
            ],
            Section::Diamond => &[(1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0)],
            Section::Midrib => &[
                (1.0, 0.0),
                (0.2, 0.4),
                (0.0, 1.1),
                (-0.2, 0.4),
                (-1.0, 0.0),
                (-0.2, -0.4),
                (0.0, -1.1),
                (0.2, -0.4),
            ],
            Section::Triangular => &[(1.0, -0.7), (0.0, 1.2), (-1.0, -0.7)],
        }
    };
    points.iter().map(|&(x, z)| Vec2::new(x, z)).collect()
}

/// A blade at one place along it: its back and edge from its middle, and that middle.
struct Across {
    back: f32,
    edge: f32,
    middle: f32,
}

/// How far along a blade its plain part below the sharpened part reaches.
fn ricasso_length(form: &BladeForm) -> f32 {
    match form.ricasso {
        Ricasso::None => 0.0,
        Ricasso::Plain | Ricasso::WithHooks => (form.blade as f32 * 0.14).clamp(25.0, 180.0),
    }
}

fn across_at(form: &BladeForm, t: f32) -> Across {
    let length = form.blade as f32;
    let half = half_width(form);
    let bend = match form.curve {
        Curve::Straight => 0.0,
        Curve::Slight => -(length * 0.07).min(60.0) * t * t,
        Curve::Strong => -(length * 0.15).min(110.0) * t * t,
        Curve::Inward => (length * 0.18).min(90.0) * t * t,
        Curve::Wavy => half * 0.4 * (t * (length / 45.0).max(4.0)).sin(),
    };
    let body = match form.shape {
        BladeShape::Parallel => 1.0,
        BladeShape::Tapered => 1.0 - 0.5 * t,
        BladeShape::Leaf => 0.7 + 0.5 * (t * std::f32::consts::PI * 0.85).sin(),
        BladeShape::Widening => 0.7 + 0.55 * t,
    };
    let plain = t * length < ricasso_length(form);
    let wide = half * body * if plain { 0.55 } else { 1.0 };
    let tip = (half * 3.0).min(length * 0.3);
    let near_end = ((t * length - (length - tip)) / tip).clamp(0.0, 1.0);
    let single = form.edges == Edges::Single;
    let (back, edge, hook) = match form.point {
        Point::Sharp if single => (wide, wide * (1.0 - 2.0 * near_end), 0.0),
        Point::Sharp => (wide * (1.0 - near_end), wide * (1.0 - near_end), 0.0),
        Point::Rounded => {
            let left = (1.0 - near_end * near_end).max(0.0).sqrt();
            (wide * left, wide * left, 0.0)
        }
        Point::None | Point::Notched => (wide, wide, 0.0),
        Point::Hooked => {
            let left = 1.0 - near_end;
            (wide * left, wide * left, near_end * near_end * half * 1.4)
        }
        Point::Clip => (
            wide * (1.0 - near_end),
            wide * (1.0 - near_end * near_end),
            0.0,
        ),
        Point::Flared => (
            wide * (1.0 + 0.5 * near_end),
            wide * (1.0 + 0.5 * near_end),
            0.0,
        ),
    };
    Across {
        back,
        edge,
        middle: bend + hook,
    }
}

/// A blade from where it meets the handle, as long as a form says.
fn add_blade(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let section = section_points(form);
    let steps = if form.curve == Curve::Wavy {
        WAVY_STEPS
    } else {
        BLADE_STEPS
    };
    let thick = thickness(half);
    let rings: Vec<Vec<Vec3>> = (0..=steps)
        .map(|step| {
            let t = step as f32 / steps as f32;
            let at = across_at(form, t);
            let thin = ((at.back + at.edge) / (2.0 * half)).clamp(0.15, 1.0);
            section
                .iter()
                .map(|point| {
                    let x = -at.back + (at.back + at.edge) * (point.x + 1.0) / 2.0;
                    Vec3::new(at.middle + x, t * length, point.y * thick * thin)
                })
                .collect()
        })
        .collect();
    model.loft(Stuff::Blade, &rings);
    if form.point == Point::Notched {
        let end = across_at(form, 1.0);
        for side in [-0.55, 0.55] {
            let from = Vec2::new(end.middle + side * half, length);
            let to = from + Vec2::new(side * half * 0.5, half);
            model.spike(Stuff::Blade, from, to, half * 0.4);
        }
    }
    if form.fuller {
        add_along_blade(model, form, Stuff::Dark, (0.03, 0.72), 0.22, 1.05);
    }
    match form.decoration {
        Decoration::Engraved => add_marks(model, form),
        Decoration::Inlaid => add_along_blade(model, form, Stuff::Inlay, (0.05, 0.3), 0.12, 1.1),
        Decoration::Plain | Decoration::Tasselled => {}
    }
    if form.ricasso == Ricasso::WithHooks {
        let at = ricasso_length(form);
        for side in [-1.0, 1.0] {
            let from = Vec2::new(side * half * 0.5, at);
            let to = Vec2::new(side * half * 1.8, at - half * 0.5);
            model.spike(Stuff::Blade, from, to, 4.0);
        }
    }
}

/// A strip following a blade's middle, between two places along it.
fn add_along_blade(
    model: &mut Model,
    form: &BladeForm,
    stuff: Stuff,
    between: (f32, f32),
    wide: f32,
    depth: f32,
) {
    let length = form.blade as f32;
    let half = half_width(form);
    let surface = thickness(half) * depth;
    for step in 0..12 {
        let place = |step: f32| between.0 + (between.1 - between.0) * step / 12.0;
        let (first, next) = (place(step as f32), place(step as f32 + 1.0));
        let centre = |t: f32| {
            let at = across_at(form, t);
            Vec2::new(at.middle + (at.edge - at.back) / 2.0, t * length)
        };
        let room = across_at(form, first);
        let strip = (room.back + room.edge) / 2.0 * wide;
        model.ribbon(stuff, centre(first), centre(next), strip, surface);
    }
}

/// Bands cut across a blade near its base.
fn add_marks(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let surface = thickness(half) * 1.05;
    for mark in 0..5 {
        let t = 0.08 + mark as f32 * 0.05;
        let at = across_at(form, t);
        let centre = at.middle + (at.edge - at.back) / 2.0;
        let reach = (at.back + at.edge) / 2.0 * 0.6;
        let y = t * length;
        let from = Vec2::new(centre - reach, y);
        let to = Vec2::new(centre + reach, y + reach * 0.5);
        model.ribbon(Stuff::Dark, from, to, 1.2, surface);
    }
}

/// Where a handle's line is, from 0 where it meets the blade to 1 at its end.
fn handle_at(form: &BladeForm, along: f32) -> Vec2 {
    let length = form.handle as f32;
    let reach = length.min(500.0);
    let x = match form.handle_shape {
        HandleShape::Straight => 0.0,
        HandleShape::Curved => -reach * 0.14 * along * along,
        HandleShape::SCurved => reach * 0.07 * (along * std::f32::consts::TAU).sin(),
        HandleShape::Angled => -reach * 0.35 * (along - 0.3).max(0.0),
    };
    Vec2::new(x, -along * length)
}

/// The handle, with its fittings at each end.
fn add_handle(model: &mut Model, form: &BladeForm) {
    let path: Vec<(Vec2, f32)> = (0..=HANDLE_STEPS)
        .map(|step| {
            let along = step as f32 / HANDLE_STEPS as f32;
            (handle_at(form, along), HANDLE_RADIUS)
        })
        .collect();
    model.tube(Stuff::Handle, &path, 0.0);
    let end = handle_at(form, 1.0);
    if form.pommel == Pommel::None {
        let cap = (end, HANDLE_RADIUS + 1.5);
        let inner = (
            handle_at(form, 1.0 - 18.0 / form.handle as f32),
            HANDLE_RADIUS + 1.5,
        );
        model.tube(Stuff::Fittings, &[inner, cap], 0.0);
    }
    if form.handle < 400 {
        // Bands of wrapping round a grip.
        for band in 1..form.handle / 30 {
            let along = band as f32 * 30.0 / form.handle as f32;
            let width = 5.0 / form.handle as f32;
            let path = [
                (handle_at(form, along - width), HANDLE_RADIUS + 0.8),
                (handle_at(form, along + width), HANDLE_RADIUS + 0.8),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

fn add_guard(model: &mut Model, form: &BladeForm) {
    let half = half_width(form);
    let reach = (form.blade as f32 * 0.16).clamp(half + 12.0, 80.0);
    let grip = form.handle as f32;
    match form.guard {
        Guard::None => {}
        Guard::Cross => add_cross(model, reach),
        Guard::Disc => model.tube(
            Stuff::Fittings,
            &[
                (Vec2::new(0.0, 3.0), reach * 0.85),
                (Vec2::new(0.0, -3.0), reach * 0.85),
            ],
            0.0,
        ),
        Guard::Cup => add_cup(model, reach),
        Guard::KnuckleBow => {
            add_cross(model, reach);
            add_bow(model, form, reach, 0.0);
        }
        Guard::Basket => {
            add_cup(model, reach);
            for depth in [-12.0, 0.0, 12.0] {
                add_bow(model, form, reach, depth);
            }
            let back = [
                (Vec2::new(-reach * 0.7, 0.0), 2.5),
                (Vec2::new(-reach * 0.75, -grip * 0.4), 2.5),
                (Vec2::new(-5.0, -grip * 0.9), 2.5),
            ];
            model.tube(Stuff::Fittings, &back, 0.0);
        }
    }
}

fn add_cross(model: &mut Model, reach: f32) {
    let path = [
        (Vec2::new(-reach, 0.0), 6.0),
        (Vec2::new(0.0, 0.0), 7.0),
        (Vec2::new(reach, 0.0), 6.0),
    ];
    model.tube(Stuff::Fittings, &path, 0.0);
}

fn add_cup(model: &mut Model, reach: f32) {
    let path = [
        (Vec2::new(0.0, 16.0), reach * 0.9),
        (Vec2::new(0.0, 6.0), reach * 0.75),
        (Vec2::new(0.0, -4.0), HANDLE_RADIUS + 2.0),
    ];
    model.tube(Stuff::Fittings, &path, 0.0);
}

/// A curved bar from the guard round the fingers to the pommel.
fn add_bow(model: &mut Model, form: &BladeForm, reach: f32, depth: f32) {
    let grip = form.handle as f32;
    let end = handle_at(form, 0.95);
    let path = [
        (Vec2::new(reach * 0.8, 0.0), 3.0),
        (Vec2::new(reach * 1.05, -grip * 0.3), 3.0),
        (Vec2::new(reach * 0.9, -grip * 0.7), 3.0),
        (Vec2::new(end.x + 10.0, end.y), 3.0),
    ];
    model.tube(Stuff::Fittings, &path, depth);
}

fn add_pommel(model: &mut Model, form: &BladeForm) {
    let end = handle_at(form, 1.0);
    match form.pommel {
        Pommel::None => {}
        Pommel::Plain => {
            let path = [
                (end + Vec2::Y * 2.0, 8.0),
                (end - Vec2::Y * 8.0, 19.0),
                (end - Vec2::Y * 20.0, 16.0),
                (end - Vec2::Y * 28.0, 6.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
        Pommel::Ring => {
            let centre = end - Vec2::Y * 20.0;
            let circle: Vec<(Vec2, f32)> = (0..=16)
                .map(|step| {
                    let angle = step as f32 / 16.0 * std::f32::consts::TAU;
                    (centre + Vec2::from_angle(angle) * 19.0, 5.0)
                })
                .collect();
            model.tube(Stuff::Fittings, &circle, 0.0);
        }
        Pommel::Wheel => model.disc(Stuff::Fittings, end - Vec2::Y * 14.0, 24.0, 9.0),
        Pommel::Disc => model.disc(Stuff::Fittings, end - Vec2::Y * 16.0, 28.0, 4.0),
        Pommel::Hooked => {
            let path = [
                (end + Vec2::Y * 4.0, 11.0),
                (end + Vec2::new(-12.0, -10.0), 10.0),
                (end + Vec2::new(-30.0, -12.0), 8.0),
                (end + Vec2::new(-42.0, -2.0), 5.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

/// The socket a head on a shaft sits on, and any strips holding it on.
fn add_shaft_top(model: &mut Model, form: &BladeForm) {
    let handle = form.handle as f32;
    let socket = (handle * 0.06).clamp(35.0, 120.0);
    let path = [
        (Vec2::new(0.0, 6.0), HANDLE_RADIUS + 1.0),
        (Vec2::new(0.0, -socket), HANDLE_RADIUS + 3.0),
    ];
    model.tube(Stuff::Blade, &path, 0.0);
    if form.langets {
        let reach = (handle * 0.14).clamp(80.0, 350.0);
        for side in [-1.0, 1.0] {
            let x = side * (HANDLE_RADIUS + 1.0);
            let path = [
                (Vec2::new(x, -socket), 4.0),
                (Vec2::new(x, -socket - reach), 3.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

/// How far a part on the side of a shaft reaches.
fn side_reach(form: &BladeForm) -> f32 {
    (form.blade as f32 / 2.0).clamp(40.0, 150.0)
}

/// How long a side head's edge is, drawn on the side `side` (1 for right, -1 for left).
fn add_side_head(model: &mut Model, form: &BladeForm, side: f32, at: f32) {
    let edge = form.blade as f32;
    let deep = (edge / 2.0).min(150.0);
    let rings: Vec<Vec<Vec3>> = (0..=16)
        .map(|step| {
            let v = step as f32 / 16.0 * 2.0 - 1.0;
            let bare = v.abs();
            let y = at + v * edge / 2.0;
            let back = HANDLE_RADIUS * 0.6 + deep * 0.72 * bare * bare;
            let front = HANDLE_RADIUS + deep * (0.8 + 0.2 * (1.0 - bare * bare));
            let thick = 8.0 * (1.0 - 0.6 * bare);
            let middle = back + (front - back) * 0.6;
            [
                Vec3::new(back * side, y, thick),
                Vec3::new(middle * side, y, thick * 0.7),
                Vec3::new(front * side, y, 0.2),
                Vec3::new(middle * side, y, -thick * 0.7),
                Vec3::new(back * side, y, -thick),
            ]
            .to_vec()
        })
        .collect();
    model.loft(Stuff::Blade, &rings);
    match form.decoration {
        Decoration::Engraved | Decoration::Inlaid => {
            let stuff = if form.decoration == Decoration::Engraved {
                Stuff::Dark
            } else {
                Stuff::Inlay
            };
            let centre = Vec2::new(side * (HANDLE_RADIUS + deep * 0.4), at);
            let from = centre - Vec2::Y * edge * 0.18;
            let to = centre + Vec2::Y * edge * 0.18;
            model.ribbon(stuff, from, to, 2.5, 8.5);
        }
        Decoration::Plain | Decoration::Tasselled => {}
    }
}

/// Something other than an axe's head sticking out of a shaft.
fn add_side_part(model: &mut Model, form: &BladeForm, part: SidePart, side: f32, at: f32) {
    let reach = side_reach(form);
    let root = Vec2::new(side * HANDLE_RADIUS * 0.6, at);
    let out = |x: f32, y: f32| root + Vec2::new(side * x, y);
    match part {
        SidePart::None => {}
        SidePart::AxeBlade => add_side_head(model, form, side, at),
        SidePart::Spike => model.spike(Stuff::Blade, root, out(reach, 0.0), 9.0),
        SidePart::Hook => {
            let path = [
                (root, 9.0),
                (out(reach * 0.55, -reach * 0.1), 7.0),
                (out(reach, reach * 0.2), 4.0),
                (out(reach * 0.9, reach * 0.65), 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        SidePart::Beak => {
            let path = [
                (root, 12.0),
                (out(reach * 0.5, reach * 0.05), 8.0),
                (out(reach * 0.9, -reach * 0.25), 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        SidePart::Lug => {
            let path = [(root, 7.0), (out(reach * 0.5, reach * 0.25), 0.8)];
            model.tube(Stuff::Blade, &path, 0.0);
        }
    }
}

fn add_back(model: &mut Model, form: &BladeForm, at: f32) {
    let root = Vec2::new(-HANDLE_RADIUS * 0.6, at);
    match form.back {
        Back::None => {}
        Back::Spike => {
            let to = root - Vec2::X * side_reach(form).max(60.0);
            model.spike(Stuff::Blade, root, to, 10.0);
        }
        Back::Hammer => {
            let path = [
                (root, 11.0),
                (root - Vec2::X * 28.0, 12.0),
                (root - Vec2::X * 34.0, 17.0),
                (root - Vec2::X * 46.0, 17.0),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
        Back::Pipe => {
            let path = [
                (root, 9.0),
                (root - Vec2::X * 34.0, 7.0),
                (root + Vec2::new(-38.0, 26.0), 9.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
    }
}

/// A head on the side of a shaft, whose top is at 0.
fn add_side_build(model: &mut Model, form: &BladeForm) {
    let edge = form.blade as f32;
    let at = -edge * 0.45;
    add_side_part(model, form, form.sides[0], 1.0, at);
    add_side_part(model, form, form.sides[1], -1.0, at);
    if form.sides[1] == SidePart::None {
        add_back(model, form, at);
    }
    let top_length = (edge * 0.45).max(80.0);
    match form.top {
        Top::None => {
            let path = [
                (Vec2::Y * 8.0, HANDLE_RADIUS * 0.8),
                (Vec2::ZERO, HANDLE_RADIUS + 1.0),
            ];
            model.tube(Stuff::Fittings, &path, 0.0);
        }
        Top::Spike => model.spike(Stuff::Blade, Vec2::ZERO, Vec2::Y * top_length, 11.0),
        Top::Blade => {
            let path = [
                (Vec2::ZERO, 9.0),
                (Vec2::Y * top_length * 0.3, 16.0),
                (Vec2::Y * top_length, 0.5),
            ];
            model.tube(Stuff::Blade, &path, 0.0);
        }
    }
}

/// A spear's divided point: a stem, a bar, and a tine for each.
fn add_tines(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    let places: &[f32] = if form.tips == Tips::TwoTines {
        &[-1.0, 1.0]
    } else {
        &[-1.0, 0.0, 1.0]
    };
    let fork = length * 0.3;
    let reach = half * 1.5;
    let stem = [(Vec2::ZERO, 11.0), (Vec2::Y * fork, 8.0)];
    model.tube(Stuff::Blade, &stem, 0.0);
    let bar = [
        (Vec2::new(-reach, fork), 6.0),
        (Vec2::new(reach, fork), 6.0),
    ];
    model.tube(Stuff::Blade, &bar, 0.0);
    for &place in places {
        let from = Vec2::new(place * reach, fork);
        let to = Vec2::new(place * reach, length);
        model.spike(Stuff::Blade, from, to, 6.0);
    }
}

fn add_barbs(model: &mut Model, form: &BladeForm) {
    let length = form.blade as f32;
    let half = half_width(form);
    for side in [-1.0, 1.0] {
        let from = Vec2::new(side * half * 0.6, length * 0.2);
        let to = Vec2::new(side * half * 1.6, -length * 0.02);
        model.spike(Stuff::Blade, from, to, 6.0);
    }
}

/// Cords hanging from where a grip ends, or below a head on a shaft.
fn add_tassel(model: &mut Model, from: Vec2) {
    for strand in [-1.0, 0.0, 1.0] {
        let path = [
            (from, 3.0),
            (from + Vec2::new(strand * 10.0, -30.0), 3.5),
            (from + Vec2::new(strand * 22.0, -75.0), 1.5),
        ];
        model.tube(Stuff::Cloth, &path, HANDLE_RADIUS * 0.3);
    }
}

/// The whole weapon, in millimetres.
fn build(form: &BladeForm) -> Model {
    let mut model = Model::default();
    add_handle(&mut model, form);
    let on_shaft = form.handle >= 400;
    match form.build {
        Build::Along => {
            match form.tips {
                Tips::TwoTines | Tips::ThreeTines => add_tines(&mut model, form),
                Tips::Single | Tips::Barbed => add_blade(&mut model, form),
            }
            if form.tips == Tips::Barbed {
                add_barbs(&mut model, form);
            }
            if on_shaft {
                add_shaft_top(&mut model, form);
            }
            add_side_part(&mut model, form, form.sides[0], 1.0, 8.0);
            add_side_part(&mut model, form, form.sides[1], -1.0, 8.0);
            add_guard(&mut model, form);
        }
        Build::Side => add_side_build(&mut model, form),
    }
    add_pommel(&mut model, form);
    if form.decoration == Decoration::Tasselled {
        let from = match (on_shaft, form.build) {
            (false, _) => handle_at(form, 1.0),
            (true, Build::Along) => {
                Vec2::new(HANDLE_RADIUS, -(form.handle as f32 * 0.06).max(35.0))
            }
            (true, Build::Side) => Vec2::new(HANDLE_RADIUS, -(form.blade as f32)),
        };
        add_tassel(&mut model, from);
    }
    model
}

/// The model placed to fill these cells, with room for its black edge: its triangles by
/// what they are made of.
fn fitted(look: &BladeLook) -> Vec<(Stuff, Vec<Triangle>)> {
    let model = build(&look.form);
    let turn = Quat::from_rotation_y(TURN);
    let turned = |corner: Vec3| turn * corner;
    let (mut low, mut high) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
    for triangle in model.triangles.iter().flatten() {
        for &corner in triangle {
            let corner = turned(corner);
            low = low.min(corner);
            high = high.max(corner);
        }
    }
    let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = high - low;
    let along = room(long) / size.y.max(1.0);
    let cell = PIXELS_PER_CELL * world_per_pixel;
    let across = (room(wide) / size.x.max(1.0)).min(cell / CELL_ACROSS_MM);
    let middle = (low + high) / 2.0;
    let scale = Vec3::new(across, along, across);
    let place = |corner: Vec3| (turned(corner) - middle) * scale;
    STUFFS
        .iter()
        .zip(&model.triangles)
        .filter(|(_, triangles)| !triangles.is_empty())
        .map(|(&stuff, triangles)| {
            let triangles = triangles.iter().map(|triangle| triangle.map(place));
            (stuff, triangles.collect())
        })
        .collect()
}

/// A flat-faced mesh of these triangles: they are small, and it shows off the bevels.
fn mesh_of(triangles: &[Triangle]) -> Mesh {
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

fn material_of(stuff: Stuff, form: &BladeForm, quality: Option<Quality>) -> StandardMaterial {
    match stuff {
        Stuff::Blade => metal_material(form.blade_metal, quality),
        Stuff::Fittings => metal_material(form.fittings, quality),
        Stuff::Handle => match form.handle_material {
            Material::Wood => StandardMaterial {
                base_color: Color::srgb(0.46, 0.29, 0.15),
                perceptual_roughness: 0.75,
                ..default()
            },
            Material::Bone => StandardMaterial {
                base_color: Color::srgb(0.86, 0.81, 0.67),
                perceptual_roughness: 0.6,
                ..default()
            },
            metal => metal_material(metal, quality),
        },
        Stuff::Dark => {
            let mut dark = metal_material(form.blade_metal, quality);
            dark.base_color = dark.base_color.darker(0.28);
            dark
        }
        Stuff::Inlay => metal_material(
            if form.blade_metal == Material::Gold {
                Material::Silver
            } else {
                Material::Gold
            },
            quality,
        ),
        Stuff::Cloth => StandardMaterial {
            base_color: Color::srgb(0.62, 0.12, 0.12),
            perceptual_roughness: 0.9,
            ..default()
        },
    }
}

/// A weapon's parts, all placed.
pub(super) fn blade_parts(
    look: &BladeLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    let facing = Transform::from_rotation(camera.rotation);
    fitted(look)
        .into_iter()
        .map(|(stuff, triangles)| Part {
            mesh: meshes.add(mesh_of(&triangles)),
            material: materials.add(material_of(stuff, &look.form, quality)),
            transform: facing,
        })
        .collect()
}

#[cfg(test)]
mod tests;
