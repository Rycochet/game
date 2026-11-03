//! Shoulder pieces: pauldrons, spaulders, hanging guards (sode and their eastern
//! cousins), doublings and the linothorax yoke, shoulder discs, ailettes and the Chukchi
//! wing. Each pair is built in millimetres in place on a body frame, lying outside the
//! breastplate and gorget (agreed with the torso's and neck's pieces): rigid pieces
//! 35 mm out from the skin, hanging ones 28 mm out from the arm. Lames end about 130 mm
//! down the arm, where a rerebrace tucks under them. The picture shows the pair as worn,
//! seen as a character is.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::shoulders::{
    Adornment, AiletteShape, CapeCut, Charge, Cut, Finish, GuardShape, HautePiece, Lacing, Length,
    ShouldersForm, ShouldersPiece, Tincture, Wings,
};
use shared::armour::{Construction, Style};
use shared::body::{Frame, Joint, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;
use shared::material::{Category, Material};

use super::surface::{Surface, bands, grid, lines, patch, ring_grid, scales, studs};
use super::{Model, Stuff, colour_of, darker, matte, plate};
use crate::item_art::body::{blob, limb, mesh_of, trunk_depth};
use crate::item_art::ring::metal_material;
use crate::item_art::{CAMERA_AT, CAMERA_UP, OUTLINE_PIXELS, Part, ShouldersLook, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Seen as the bare body is: turned a little to its right and from a little above.
const TURN: f32 = 0.35;
const TILT: f32 = 0.25;
/// How far each piece of a pair is turned to show its outside, and the gap between them.
const SHOW_TURN: f32 = 0.3;
const PICTURE_GAP: f32 = 16.0;
/// How far rigid pieces stand out from the skin, and the wings from the trunk's: past
/// a breastplate and a gorget.
const RIGID_CLEAR: f32 = 35.0;
const WING_CLEAR: f32 = 45.0;
/// How far hanging pieces are out from the arm's skin: past a rerebrace.
const HANGING_CLEAR: f32 = 28.0;
/// How far a cape or yoke lies off the skin, over a gambeson.
const DRAPE_CLEAR: f32 = 22.0;
/// Half the gap at the front of a cape, as an angle round it.
const OPENING: f32 = 0.14;
/// Points round and along each plate.
const ACROSS: usize = 16;
const ALONG: usize = 8;

/// A plate over a whole surface.
fn sheet(model: &mut Model, stuff: Stuff, surface: Surface, thick: f32) {
    plate(model.of(stuff), &grid(surface, ALONG, ACROSS), thick, false);
}

/// An arm's own directions at the shoulder: down the upper arm, out from the body
/// square to it, and forwards.
struct Arm {
    side: Side,
    sign: f32,
    joint: Vec3,
    down: Vec3,
    out: Vec3,
    forward: Vec3,
    /// The upper arm's radius where the bare body is widest at the shoulder.
    radius: f32,
    /// How far down the arm the shoulder pieces reach.
    reach: f32,
}

impl Arm {
    fn of(frame: &Frame, side: Side) -> Self {
        let point = |mm: [i32; 3]| Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32);
        let joint = point(frame.joint(Joint::Shoulder(side)));
        let down = (point(frame.joint(Joint::Elbow(side))) - joint).normalize();
        let sideways = Vec3::X * side.sign() as f32;
        let out = (sideways - down * sideways.dot(down)).normalize();
        Self {
            side,
            sign: side.sign() as f32,
            joint,
            down,
            out,
            forward: down.cross(out).normalize() * down.cross(out).z.signum(),
            radius: frame.upper_arm as f32 * 1.15,
            reach: frame.elbow_length as f32 * 130.0 / 300.0,
        }
    }

    /// Round the arm: `along` down it from the joint, `angle` from straight out towards
    /// the front.
    fn round(&self, along: f32, angle: f32, radius: f32) -> (Vec3, Vec3) {
        let out = self.out * angle.cos() + self.forward * angle.sin();
        (self.joint + self.down * along + out * radius, out)
    }

    /// Over the shoulder as a dome: `over` from its top (0) down to where it meets the
    /// arm (π/2), half widths across and up.
    fn dome(&self, angle: f32, over: f32, across: f32, up: f32) -> (Vec3, Vec3) {
        let out = self.out * angle.cos() + self.forward * angle.sin();
        let point = self.joint - self.down * (up * over.cos()) + out * (across * over.sin());
        let normal = (-self.down * over.cos() / up + out * over.sin() / across).normalize();
        (point, normal)
    }
}

/// How a style shapes plates.
#[derive(Clone, Copy)]
struct Look {
    thick: f32,
    /// Flutes over the cop and lames: how many across a half turn, and how deep.
    flutes: (f32, f32),
    /// How far each lame's edge is cut into points, and how deep.
    cusps: f32,
    /// A leaf point at the bottom of each lame, and a peak at the top.
    leaf: f32,
    /// How flat the cop's top is, out of 1.
    squat: f32,
    rim: f32,
    step: f32,
    /// Heavier rivets, and a band of runes.
    runes: bool,
    etched: bool,
}

fn look_of(style: Style) -> Look {
    let base = Look {
        thick: 2.0,
        flutes: (0.0, 0.0),
        cusps: 0.0,
        leaf: 0.0,
        squat: 1.0,
        rim: 2.2,
        step: 1.6,
        runes: false,
        etched: false,
    };
    match style {
        // Regional styles shape their own constructions, not plate.
        Style::White
        | Style::Japanese
        | Style::Chinese
        | Style::Steppe
        | Style::Classical
        | Style::Byzantine
        | Style::IndoPersian => base,
        Style::Greenwich => Look {
            etched: true,
            ..base
        },
        Style::Milanese => Look { rim: 3.0, ..base },
        Style::Gothic => Look {
            flutes: (7.0, 4.0),
            cusps: 7.0,
            rim: 0.0,
            ..base
        },
        Style::Maximilian => Look {
            flutes: (16.0, 2.6),
            ..base
        },
        Style::Kastenbrust => Look {
            squat: 0.75,
            ..base
        },
        Style::Elven => Look {
            thick: 1.4,
            leaf: 22.0,
            rim: 0.0,
            ..base
        },
        Style::Dwarven => Look {
            thick: 3.2,
            squat: 0.8,
            rim: 3.5,
            step: 4.0,
            runes: true,
            ..base
        },
    }
}

/// How plates are shaped: by style for plate, plainly for anything else.
fn shaped(form: &ShouldersForm) -> Look {
    let look = look_of(form.style);
    match form.construction {
        Construction::Plate | Construction::Mirror => look,
        Construction::Leather => Look {
            thick: 4.0,
            flutes: (0.0, 0.0),
            rim: 0.0,
            etched: false,
            ..look
        },
        _ => Look {
            flutes: (0.0, 0.0),
            cusps: 0.0,
            leaf: 0.0,
            rim: 0.0,
            etched: false,
            runes: false,
            ..look
        },
    }
}

fn flute(look: Look, angle: f32) -> f32 {
    let (count, deep) = look.flutes;
    if count == 0.0 {
        0.0
    } else {
        deep * (angle * count).cos().max(0.0).powi(2)
    }
}

/// How far a lame's bottom edge comes down at this angle, for cusps and leaf points.
fn edge_drop(look: Look, angle: f32) -> f32 {
    look.cusps * (angle * 6.0).sin().abs() + look.leaf * angle.cos().max(0.0).powi(6)
}

/// The cop: a dome over the joint, from `start` to `end` round the arm.
fn cop_surface<'a>(
    arm: &'a Arm,
    look: Look,
    (start, end): (f32, f32),
    (across, up): (f32, f32),
    to: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let angle = start + (end - start) * u;
        let over = 0.12 + (to - 0.12) * v;
        // A leaf peak at the top, straight out.
        let peak = look.leaf * angle.cos().max(0.0).powi(8) * (1.0 - v);
        let grow = flute(look, angle) * over.sin();
        let (point, normal) = arm.dome(angle, over, across + grow, up * look.squat + peak);
        (point, normal)
    }
}

/// A band round the arm from `top` to `bottom` down it, between two angles.
fn arm_surface<'a>(
    arm: &'a Arm,
    look: Look,
    (start, end): (f32, f32),
    (top, bottom): (f32, f32),
    radius: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let angle = start + (end - start) * u;
        let along = top + (bottom - top) * v + edge_drop(look, angle) * v.powi(4);
        arm.round(along, angle, radius + flute(look, angle))
    }
}

/// The front or back of the trunk, well clear of it, from `inner` across to `outer`
/// and down from `top` by `drop`, rounded at the bottom.
fn wing_surface<'a>(
    frame: &'a Frame,
    sign: f32,
    front: bool,
    (outer, inner): (f32, f32),
    top: f32,
    drop: f32,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    move |u: f32, v: f32| {
        let x = outer + (inner - outer) * u;
        let y = top - v * drop * (0.55 + 0.45 * (PI * u).sin());
        let deep = trunk_depth(frame, x, y) + WING_CLEAR;
        let z = if front { deep } else { -deep };
        let normal = Vec3::new(sign * x * 0.004, 0.25, z.signum()).normalize();
        (Vec3::new(sign * x, y, z), normal)
    }
}

/// The trunk's top between the neck and the shoulder, as the body is drawn.
fn shoulder_top(frame: &Frame, x: f32) -> f32 {
    let neck = frame.neck as f32 * 1.6;
    let joint = frame.shoulder_half as f32;
    let slope = frame.neck_base as f32
        - (frame.neck_base - frame.shoulder) as f32 * ((x - neck) / (joint - neck)).clamp(0.0, 1.0);
    let ball = frame.upper_arm as f32 * 1.15;
    let over_ball = frame.shoulder as f32 + (ball * ball - (x - joint).powi(2)).max(0.0).sqrt();
    slope.max(over_ball)
}

/// One pauldron or spaulder: cop, lames, and what goes with them.
#[allow(clippy::too_many_arguments)]
fn add_cop_and_lames(
    model: &mut Model,
    form: &ShouldersForm,
    frame: &Frame,
    arm: &Arm,
    look: Look,
    (start, end): (f32, f32),
    lames: u8,
    besagews: bool,
) {
    let across = arm.radius + RIGID_CLEAR;
    // Broader than it is high, as a cop is.
    let up = across * 0.72;
    let stuff = Stuff::Main;
    let laminar = form.construction == Construction::Laminar;
    let cop = cop_surface(arm, look, (start, end), (across, up), PI / 2.0);
    if laminar {
        // Roman: the cop itself in lames, each lower one further out.
        let count = (lames as usize + 2).max(4);
        bands(model, stuff, &cop, count, -look.step, look.thick);
    } else {
        sheet(model, stuff, &cop, look.thick);
    }
    if look.runes {
        patch(
            model,
            Stuff::Etch,
            &cop,
            (0.1, 0.9),
            (0.82, 0.92),
            look.thick / 2.0 + 0.4,
            0.6,
        );
    }
    if form.finish == Finish::Studded {
        studs(model, Stuff::Rivet, &cop, (3, 7), look.thick / 2.0, 3.5);
    }
    if lames == 0 {
        return;
    }
    let lames = lames as usize;
    let length = arm.reach / lames as f32;
    for lame in 0..lames {
        let top = lame as f32 * length - 8.0;
        let bottom = (lame + 1) as f32 * length;
        // Each lame over the one below.
        // Flaring out down the arm.
        let radius = across + lame as f32 * 5.0 + (lames - lame) as f32 * look.step;
        let band = arm_surface(arm, look, (start, end), (top, bottom), radius);
        sheet(model, stuff, &band, look.thick);
        if look.etched {
            patch(
                model,
                Stuff::Etch,
                &band,
                (0.0, 1.0),
                (0.62, 0.86),
                look.thick / 2.0 + 0.3,
                0.6,
            );
        }
        if form.finish == Finish::Studded {
            studs(model, Stuff::Rivet, &band, (1, 9), look.thick / 2.0, 3.0);
        } else if form.construction == Construction::Plate {
            let rivet = if look.runes { 4.5 } else { 2.6 };
            studs(model, Stuff::Rivet, &band, (1, 2), look.thick / 2.0, rivet);
        }
        if lame == lames - 1 && look.rim > 0.0 {
            patch(
                model,
                stuff,
                &band,
                (0.0, 1.0),
                (0.86, 1.0),
                look.rim,
                look.thick,
            );
        }
    }
    if besagews {
        add_besagew(model, frame, arm, look);
    }
}

/// A disc hung in front of the armpit.
fn add_besagew(model: &mut Model, frame: &Frame, arm: &Arm, look: Look) {
    let x = arm.joint.x.abs() - arm.radius * 0.8;
    let y = arm.joint.y - arm.reach * 0.55;
    let z = trunk_depth(frame, x, y) + WING_CLEAR + 8.0;
    let centre = Vec3::new(arm.sign * x, y, z);
    let facing = Vec3::new(arm.sign * 0.3, 0.1, 1.0).normalize();
    add_disc(
        model,
        Stuff::Main,
        centre,
        facing,
        arm.radius * 1.15,
        8.0,
        look.thick,
    );
    blob(
        model.of(Stuff::Main),
        centre + facing * 14.0,
        Vec3::new(5.0, 5.0, 5.0),
    );
}

/// A shallow dome facing `facing`.
fn add_disc(
    model: &mut Model,
    stuff: Stuff,
    centre: Vec3,
    facing: Vec3,
    radius: f32,
    dome: f32,
    thick: f32,
) {
    let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize_or(Vec3::Z);
    let side = facing.cross(up);
    let surface = |u: f32, v: f32| {
        let angle = u * TAU;
        let out = 0.04 + 0.96 * v;
        let point = centre
            + (up * angle.cos() + side * angle.sin()) * radius * out
            + facing * dome * (1.0 - out * out);
        (point, facing)
    };
    plate(
        model.of(stuff),
        &ring_grid(&surface, 4, ACROSS + 4),
        thick,
        true,
    );
}

/// The wings over breast and back, and the upper lames, haute-piece and gardbrace.
#[allow(clippy::too_many_arguments)]
fn add_pauldron_extras(
    model: &mut Model,
    frame: &Frame,
    arm: &Arm,
    look: Look,
    wings: Wings,
    cut: Cut,
    upper_lames: u8,
    haute_piece: HautePiece,
    gardbrace: bool,
) {
    let joint = frame.shoulder_half as f32;
    let great = cut == Cut::GreatLeft && arm.side == Side::Left;
    let lance = cut == Cut::LanceCut && arm.side == Side::Right;
    let small_cut = cut == Cut::GreatLeft && arm.side == Side::Right;
    let inner = match (wings, great, small_cut) {
        (_, true, _) => joint * 0.15,
        (_, _, true) => joint * 0.6,
        (Wings::Large, ..) => joint * 0.32,
        (Wings::Small, ..) => joint * 0.58,
    };
    let deeper = if great { 40.0 } else { 0.0 };
    let top = frame.shoulder as f32 + 12.0;
    let outer = joint - 15.0;
    let front_inner = if lance { joint * 0.8 } else { inner };
    for (front, drop, from) in [
        (true, 170.0 + deeper, front_inner),
        (false, 200.0 + deeper, inner),
    ] {
        let wing = wing_surface(frame, arm.sign, front, (outer, from), top, drop);
        sheet(model, Stuff::Main, &wing, look.thick);
        if look.rim > 0.0 {
            patch(
                model,
                Stuff::Main,
                &wing,
                (0.0, 1.0),
                (0.9, 1.0),
                look.rim,
                look.thick,
            );
        }
        if look.etched {
            patch(
                model,
                Stuff::Etch,
                &wing,
                (0.0, 1.0),
                (0.7, 0.82),
                look.thick / 2.0 + 0.3,
                0.6,
            );
        }
    }
    // Upper lames, over the trunk towards the neck.
    let neck_side = frame.neck as f32 * 2.2;
    for lame in 0..upper_lames {
        let from = joint - 25.0 - lame as f32 * 22.0;
        let to = (from - 28.0).max(neck_side);
        let surface = |u: f32, v: f32| {
            let x = from + (to - from) * v;
            let z = (u * 2.0 - 1.0) * 75.0;
            let y = shoulder_top(frame, x) + RIGID_CLEAR
                - 4.0 * lame as f32
                - (z / 75.0).powi(2) * 22.0;
            (
                Vec3::new(arm.sign * x, y, z),
                Vec3::new(0.0, 1.0, z * 0.006).normalize(),
            )
        };
        sheet(model, Stuff::Main, &surface, look.thick);
    }
    if haute_piece != HautePiece::None {
        let high = haute_piece.height() as f32 * frame.elbow_length as f32 / 300.0;
        let surface = |u: f32, v: f32| {
            let angle = (u - 0.5) * 2.4;
            // Rising from inside the cop's inner edge.
            let x = joint - 52.0 * angle.cos();
            let z = 80.0 * angle.sin();
            let base = shoulder_top(frame, x) + RIGID_CLEAR - 22.0;
            let point = Vec3::new(arm.sign * (x - v * 6.0), base + v * (high + 22.0), z);
            (
                point,
                Vec3::new(-arm.sign * angle.cos(), 0.0, -angle.sin()).normalize(),
            )
        };
        sheet(model, Stuff::Main, &surface, look.thick);
    }
    if gardbrace {
        let across = arm.radius + RIGID_CLEAR + 7.0;
        let surface = cop_surface(arm, look, (0.0, 1.9), (across, across - 12.0), PI / 2.0);
        let part = |u: f32, v: f32| surface(u, 0.3 + 0.7 * v);
        sheet(model, Stuff::Main, &part, look.thick);
    }
}

/// Pauldrons of scales or plated mail hang in one piece from a small cop.
fn add_hanging_pauldron(model: &mut Model, form: &ShouldersForm, arm: &Arm, look: Look) {
    let across = arm.radius + RIGID_CLEAR - 8.0;
    let cop = cop_surface(arm, look, (-2.2, 2.2), (across, across - 10.0), PI / 2.0);
    let band = arm_surface(arm, look, (-2.2, 2.2), (-6.0, arm.reach * 1.1), across);
    match form.construction {
        Construction::Scale => {
            sheet(model, Stuff::Backing, &cop, 1.5);
            sheet(model, Stuff::Backing, &band, 1.5);
            scales(model, Stuff::Main, &cop, 4, 14);
            scales(model, Stuff::Main, &band, 6, 16);
        }
        _ => {
            let small = cop_surface(arm, look, (-1.8, 1.8), (across + 4.0, across - 6.0), 1.1);
            sheet(model, Stuff::Main, &cop, 1.5);
            lines(model, &cop, 8, 0.8);
            sheet(model, Stuff::Main, &small, 2.0);
            sheet(model, Stuff::Main, &band, 1.5);
            lines(model, &band, 10, 0.8);
            // Small plates set in the mail, in rows.
            for row in 0..3 {
                let v = 0.12 + row as f32 * 0.3;
                for column in 0..6 {
                    let u = 0.08 + column as f32 * 0.17;
                    patch(
                        model,
                        Stuff::Main,
                        &band,
                        (u, u + 0.12),
                        (v, v + 0.2),
                        1.6,
                        1.6,
                    );
                }
            }
        }
    }
}

/// Spaulders' pteruges: strips hanging below the last lame, in one or two rows.
fn add_pteruges(model: &mut Model, arm: &Arm, rows: u8, radius: f32) {
    for row in 0..rows {
        let long = 85.0 + row as f32 * 30.0;
        let radius = radius - 4.0 - row as f32 * 4.0;
        let count = 9;
        for strip in 0..count {
            let middle = -1.7 + 3.4 * (strip as f32 + 0.5 * row as f32) / count as f32;
            let half = 13.0 / radius;
            let surface = |u: f32, v: f32| {
                arm.round(
                    arm.reach - 10.0 + v * long,
                    middle + (u * 2.0 - 1.0) * half,
                    radius,
                )
            };
            sheet(model, Stuff::Backing, &surface, 2.0);
        }
    }
}

/// A panel hanging over the upper arm, its middle `HANGING_CLEAR` out from it.
fn guard_surface<'a>(
    arm: &'a Arm,
    frame: &'a Frame,
    (wide, high): (f32, f32),
    shape: GuardShape,
) -> impl Fn(f32, f32) -> (Vec3, Vec3) + 'a {
    // Flatter the wider it is.
    let near = arm.radius + HANGING_CLEAR;
    let bend = (wide / 1.6).max(near);
    let scale = frame.elbow_length as f32 / 300.0;
    move |u: f32, v: f32| {
        let width = wide
            * match shape {
                GuardShape::Straight => 1.0,
                GuardShape::Tapered => 1.0 - 0.3 * v,
                GuardShape::Flared => 1.0 + 0.2 * v,
            };
        let angle = (u - 0.5) * width / bend;
        let along = -45.0 * scale + v * high;
        let axis = arm.joint + arm.down * along - arm.out * (bend - near);
        let out = arm.out * angle.cos() + arm.forward * angle.sin();
        (axis + out * bend, out)
    }
}

fn add_guards(model: &mut Model, form: &ShouldersForm, frame: &Frame, arm: &Arm) {
    let ShouldersPiece::Guards {
        size,
        shape,
        rows,
        lacing,
    } = form.piece
    else {
        return;
    };
    let japanese = form.japanese();
    let scale = frame.elbow_length as f32 / 300.0;
    let (wide, high) = size.guard();
    let size = (wide as f32 * scale, high as f32 * scale);
    let surface = guard_surface(arm, frame, size, shape);
    let rows = rows as usize;
    // A Japanese guard's solid top plate, a little flared.
    let body = |u: f32, v: f32| surface(u, if japanese { 0.1 + 0.9 * v } else { v });
    if japanese {
        let top = |u: f32, v: f32| {
            let (point, out) = surface(u, v * 0.1);
            (point + out * (6.0 - v * 4.0), out)
        };
        sheet(model, Stuff::Main, &top, 3.0);
    }
    match form.construction {
        Construction::Lamellar | Construction::Laminar => {
            bands(model, Stuff::Main, &body, rows, 1.8, 2.0);
            let (count, width) = match (lacing, form.construction) {
                (Lacing::Dense, Construction::Lamellar) => (14, 0.035),
                (Lacing::Dense, _) => (9, 0.04),
                (Lacing::Spaced, _) => (5, 0.025),
            };
            for column in 0..count {
                let u = (column as f32 + 0.5) / count as f32;
                let pair = if lacing == Lacing::Spaced {
                    [-0.02, 0.02]
                } else {
                    [0.0, 0.0]
                };
                for shift in pair
                    .into_iter()
                    .take(if lacing == Lacing::Spaced { 2 } else { 1 })
                {
                    let u = u + shift;
                    patch(
                        model,
                        Stuff::Cloth,
                        &body,
                        (u - width / 2.0, u + width / 2.0),
                        (0.0, 1.0),
                        rows as f32 * 1.8 + 1.6,
                        0.8,
                    );
                }
            }
            if japanese {
                // The bottom row's cross-knots in a band of their own.
                patch(
                    model,
                    Stuff::Cloth,
                    &body,
                    (0.0, 1.0),
                    (0.93, 1.0),
                    2.6,
                    1.0,
                );
            }
        }
        Construction::Scale => {
            sheet(model, Stuff::Backing, &body, 1.5);
            scales(model, Stuff::Main, &body, rows * 2, 12);
        }
        Construction::PlatesInCloth => {
            sheet(model, Stuff::Cloth, &body, 3.0);
            studs(model, Stuff::Rivet, &body, (rows, 8), 1.5, 3.0);
        }
        Construction::Mail => {
            sheet(model, Stuff::Cloth, &body, 1.5);
            patch(
                model,
                Stuff::Main,
                &body,
                (0.08, 0.92),
                (0.05, 0.95),
                1.5,
                1.2,
            );
            lines(
                model,
                &|u, v| body(0.08 + 0.84 * u, 0.05 + 0.9 * v),
                rows * 4,
                2.4,
            );
        }
        _ => {
            bands(model, Stuff::Main, &body, rows, 2.0, 3.5);
            match form.finish {
                Finish::Studded => studs(
                    model,
                    Stuff::Rivet,
                    &body,
                    (rows, 7),
                    rows as f32 * 2.0 + 1.8,
                    3.0,
                ),
                Finish::RingMail => lines(model, &body, rows * 3, rows as f32 * 2.0 + 2.0),
                Finish::Tooled => lines(model, &body, rows * 2, rows as f32 * 2.0 + 1.9),
                Finish::Plain => {}
            }
        }
    }
}

/// A cape over both shoulders: rings from the neck out over the shoulders and down.
fn add_cape(model: &mut Model, form: &ShouldersForm, frame: &Frame, length: Length) {
    let f = |value: i32| value as f32;
    let spread = f(frame.arm_spread) / 1000.0;
    let down = match length {
        Length::Short => 60.0,
        Length::Long => 150.0,
    } * f(frame.elbow_length)
        / 300.0;
    let bottom = f(frame.shoulder) - down;
    // Over the shoulders' tops, rising a little round the neck as a collar.
    let over = f(frame.shoulder) + f(frame.upper_arm) * 1.15 + DRAPE_CLEAR;
    let rings = [
        (over + 22.0, (f(frame.neck) * 1.45, f(frame.neck) * 1.3)),
        (
            over,
            (
                f(frame.shoulder_half) + 15.0,
                f(frame.chest_half.1) * 0.75 + DRAPE_CLEAR,
            ),
        ),
        (
            bottom,
            (
                f(frame.shoulder_half) + down * spread + f(frame.upper_arm) + DRAPE_CLEAR + 10.0,
                f(frame.chest_half.1) + DRAPE_CLEAR + 15.0,
            ),
        ),
    ];
    let surface = |u: f32, v: f32| {
        // Open at the front, where it hooks together.
        let angle = PI / 2.0 + OPENING + u * (TAU - 2.0 * OPENING);
        let (from, to, t) = if v < 0.3 {
            (rings[0], rings[1], v / 0.3)
        } else {
            (rings[1], rings[2], (v - 0.3) / 0.7)
        };
        // Longest over the arms, shorter in front and behind.
        let y = from.0
            + (to.0 - from.0) * t
            + if v > 0.3 {
                35.0 * angle.sin().abs() * t
            } else {
                0.0
            };
        let across = from.1.0 + (to.1.0 - from.1.0) * t;
        let deep = from.1.1 + (to.1.1 - from.1.1) * t;
        let point = Vec3::new(across * angle.cos(), y, deep * angle.sin());
        let normal = Vec3::new(
            angle.cos() / across * 100.0,
            (1.0 - v) * 0.9,
            angle.sin() / deep * 100.0,
        )
        .normalize();
        (point, normal)
    };
    let mesh = grid(&surface, 10, 40);
    match form.construction {
        Construction::Scale => {
            plate(model.of(Stuff::Backing), &mesh, 1.5, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, 0.12 + v * 0.88);
                scales(model, Stuff::Main, &quarter, 7, 9);
            }
        }
        Construction::Leather => {
            plate(model.of(Stuff::Main), &mesh, 3.0, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, v);
                lines(model, &quarter, 14, 2.0);
            }
        }
        _ => {
            plate(model.of(Stuff::Main), &mesh, 2.0, false);
            for part in 0..4 {
                let u0 = part as f32 / 4.0;
                let quarter = |u: f32, v: f32| surface(u0 + u / 4.0, v);
                lines(model, &quarter, 16, 1.2);
            }
        }
    }
    if form.trim.is_some() {
        let edge = |u: f32, v: f32| surface(u, 0.94 + v * 0.06);
        plate(model.of(Stuff::Trim), &grid(&edge, 2, 40), 3.0, false);
    }
}

/// The top of a linothorax: a band from the back over each shoulder, tied down in front.
fn add_yoke(model: &mut Model, form: &ShouldersForm, frame: &Frame, length: Length) {
    let f = |value: i32| value as f32;
    let joint = f(frame.shoulder_half);
    let low = f(frame.shoulder)
        - match length {
            Length::Short => 80.0,
            Length::Long => 110.0,
        };
    for side in Side::BOTH {
        let sign = side.sign() as f32;
        let surface = |u: f32, v: f32| {
            let x = joint * (0.42 + 0.46 * u);
            let angle = (v - 0.5) * PI;
            let top = shoulder_top(frame, x) + DRAPE_CLEAR;
            let y = low + (top - low) * angle.cos();
            let z = angle.sin() * (trunk_depth(frame, x, y) + DRAPE_CLEAR);
            (
                Vec3::new(sign * x, y, z),
                Vec3::new(0.0, angle.cos(), angle.sin()).normalize(),
            )
        };
        let thick = if form.construction == Construction::Padded {
            5.0
        } else {
            3.5
        };
        plate(model.of(Stuff::Main), &grid(&surface, 16, 6), thick, false);
        if form.finish == Finish::Studded {
            studs(model, Stuff::Rivet, &surface, (8, 2), thick / 2.0, 3.0);
        } else if form.finish == Finish::Tooled || form.construction == Construction::Padded {
            lines(model, &|u, v| surface(v, u), 4, thick / 2.0);
        }
    }
}

/// Where a shoulder disc's middle is, and which way it faces: out and a little up.
fn disc_place(arm: &Arm) -> (Vec3, Vec3) {
    let centre = arm.joint + arm.out * (arm.radius + RIGID_CLEAR - 6.0) - arm.down * 8.0;
    (centre, (arm.out - arm.down * 0.35).normalize())
}

fn add_shoulder_disc(
    model: &mut Model,
    form: &ShouldersForm,
    arm: &Arm,
    diameter: f32,
    boss: bool,
) {
    let (centre, facing) = disc_place(arm);
    let radius = diameter / 2.0;
    if form.construction == Construction::MailAndPlate {
        let mail = arm_surface(
            arm,
            look_of(Style::White),
            (-2.0, 2.0),
            (-20.0, arm.reach),
            arm.radius + 20.0,
        );
        sheet(model, Stuff::Shade, &mail, 1.5);
        lines(model, &mail, 10, 0.8);
        let cop = cop_surface(
            arm,
            look_of(Style::White),
            (-2.0, 2.0),
            (arm.radius + 20.0, arm.radius + 10.0),
            PI / 2.0,
        );
        sheet(model, Stuff::Shade, &cop, 1.5);
        lines(model, &cop, 6, 0.8);
    }
    add_disc(
        model,
        Stuff::Main,
        centre,
        facing,
        radius,
        radius * 0.15,
        2.2,
    );
    let rim = if form.trim.is_some() {
        Stuff::Trim
    } else {
        Stuff::Main
    };
    let surface = |u: f32, v: f32| {
        let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize();
        let side = facing.cross(up);
        let angle = u * TAU;
        let out = radius * (0.88 + 0.12 * v);
        (
            centre + (up * angle.cos() + side * angle.sin()) * out + facing * 2.0,
            facing,
        )
    };
    plate(model.of(rim), &ring_grid(&surface, 1, 28), 3.0, true);
    if form.construction == Construction::Mirror {
        let inner = |u: f32, v: f32| {
            let up = (Vec3::Y - facing * Vec3::Y.dot(facing)).normalize();
            let side = facing.cross(up);
            let angle = u * TAU;
            let out = radius * (0.5 + 0.06 * v);
            (
                centre
                    + (up * angle.cos() + side * angle.sin()) * out
                    + facing * (radius * 0.15 * 0.75 + 1.5),
                facing,
            )
        };
        plate(model.of(Stuff::Shade), &ring_grid(&inner, 1, 24), 1.2, true);
    }
    if boss {
        blob(
            model.of(Stuff::Main),
            centre + facing * (radius * 0.15 + 4.0),
            Vec3::splat(radius * 0.13),
        );
    }
}

/// A board standing on the shoulder, facing out and a little forwards, tipped back.
fn add_ailette(
    model: &mut Model,
    form: &ShouldersForm,
    frame: &Frame,
    arm: &Arm,
    shape: AiletteShape,
    (wide, high): (f32, f32),
    charge: Charge,
) {
    let joint = frame.shoulder_half as f32;
    let base = Vec3::new(
        arm.sign * (joint - 12.0),
        shoulder_top(frame, joint - 12.0) + 18.0,
        -10.0,
    );
    let facing = Vec3::new(arm.sign * 0.8, 0.0, 0.6).normalize();
    let across = Vec3::Y.cross(facing).normalize();
    let tip = 15f32.to_radians();
    let up = (Vec3::Y * tip.cos() - facing * tip.sin()).normalize();
    let facing = across.cross(up).normalize() * across.cross(up).dot(facing).signum();
    let width = move |v: f32| match shape {
        AiletteShape::Rectangle => 1.0,
        AiletteShape::Rounded => (1.0 - ((v - 0.7).max(0.0) / 0.3).powi(2))
            .max(0.0)
            .sqrt()
            .max(0.3),
        AiletteShape::Shield => (v / 0.45).min(1.0).sqrt().max(0.06),
        AiletteShape::Lozenge => (1.0 - (2.0 * v - 1.0).abs()).max(0.06),
    };
    let at = move |u: f32, v: f32| base + across * (u * wide / 2.0 * width(v)) + up * (v * high);
    let board = |u: f32, v: f32| (at(u * 2.0 - 1.0, v), facing);
    let thick = match (form.construction, form.material) {
        (Construction::Leather, _) => 6.0,
        (_, Material::Wood) => 8.0,
        _ => 2.0,
    };
    plate(model.of(Stuff::Field), &grid(&board, 10, 8), thick, false);
    // The charge as bars on the front, kept inside the outline.
    let lift = thick / 2.0 + 0.5;
    let bar = |model: &mut Model, from: Vec2, to: Vec2, half: f32| {
        let along = to - from;
        let square = Vec2::new(-along.y, along.x).normalize() * half;
        let surface = |u: f32, v: f32| {
            let point = from + along * v + square * (u * 2.0 - 1.0);
            let v = point.y.clamp(0.0, 1.0);
            let limit = width(v) * 0.9;
            (at(point.x.clamp(-limit, limit), v) + facing * lift, facing)
        };
        plate(model.of(Stuff::Charge), &grid(&surface, 8, 2), 0.8, false);
    };
    let x = |sign: f32| sign * arm.sign;
    match charge {
        Charge::Plain => {}
        Charge::Cross => {
            bar(model, Vec2::new(0.0, 0.05), Vec2::new(0.0, 0.95), 0.12);
            bar(model, Vec2::new(-0.95, 0.6), Vec2::new(0.95, 0.6), 0.07);
        }
        Charge::Saltire => {
            bar(model, Vec2::new(-0.9, 0.95), Vec2::new(0.9, 0.05), 0.1);
            bar(model, Vec2::new(0.9, 0.95), Vec2::new(-0.9, 0.05), 0.1);
        }
        Charge::Pale => bar(model, Vec2::new(0.0, 0.0), Vec2::new(0.0, 1.0), 0.3),
        Charge::Fess => bar(model, Vec2::new(-1.0, 0.55), Vec2::new(1.0, 0.55), 0.1),
        Charge::Bend => bar(
            model,
            Vec2::new(x(1.0), 0.95),
            Vec2::new(x(-1.0), 0.05),
            0.12,
        ),
        Charge::Chevron => {
            bar(model, Vec2::new(-0.95, 0.15), Vec2::new(0.0, 0.7), 0.1);
            bar(model, Vec2::new(0.95, 0.15), Vec2::new(0.0, 0.7), 0.1);
        }
    }
}

/// One huge board of planks over the left shoulder, from the ear to the waist, wider at
/// the top, laced on two straps.
fn add_wing(model: &mut Model, frame: &Frame) {
    let f = |value: i32| value as f32;
    let side = 1.0;
    let joint = f(frame.shoulder_half);
    let bend = joint * 1.2;
    let centre = joint - 60.0;
    let (bottom, top) = (f(frame.waist), f(frame.chin) + 30.0);
    let surface = |u: f32, v: f32| {
        let reach = 0.62 * (0.65 + 0.35 * (1.0 - v));
        let angle = (u * 2.0 - 1.0) * reach;
        let y = top - v * (top - bottom);
        let out = Vec3::new(side * angle.cos(), 0.0, angle.sin());
        (Vec3::new(side * centre, y, 0.0) + out * bend, out)
    };
    bands(model, Stuff::Main, &surface, 9, 2.5, 8.0);
    for strap in [0.3, 0.7] {
        patch(
            model,
            Stuff::Backing,
            &surface,
            (strap - 0.03, strap + 0.03),
            (0.0, 1.0),
            9.0 * 2.5 + 5.0,
            2.0,
        );
    }
}

/// Spikes, horns, a beast's head or a skull, on the outside of the shoulder.
fn add_adornment(model: &mut Model, form: &ShouldersForm, arm: &Arm, height: f32) {
    let across = arm.radius + RIGID_CLEAR + height;
    // On a disc's middle, or over the cop.
    let disc = match form.piece {
        ShouldersPiece::Discs { .. } => Some(disc_place(arm)),
        _ => None,
    };
    let on = |over: f32, angle: f32| {
        disc.unwrap_or_else(|| arm.dome(angle, over, across, across * 0.72))
    };
    let spikes = if form.construction == Construction::Leather {
        Stuff::Rivet
    } else {
        Stuff::Main
    };
    match form.adornment {
        Adornment::None => {}
        Adornment::Spikes(count) => {
            for spike in 0..count {
                let over = if count == 1 {
                    0.6
                } else {
                    0.3 + 0.75 * spike as f32 / (count - 1) as f32
                };
                let (point, out) = on(over, 0.0);
                let long = if count == 1 { 60.0 } else { 42.0 };
                limb(
                    model.of(spikes),
                    &[(point, 10.0), (point + out * long, 0.8)],
                );
            }
        }
        Adornment::Horns => {
            let (point, out) = on(0.55, 0.0);
            let up = -arm.down;
            let path: Vec<_> = (0..6)
                .map(|step| {
                    let t = step as f32 / 5.0;
                    let at = point
                        + out * (70.0 * t)
                        + up * (45.0 * t * t)
                        + arm.forward * (25.0 * t * t);
                    (at, 13.0 * (1.0 - t) + 1.5)
                })
                .collect();
            limb(model.of(Stuff::Bone), &path);
        }
        Adornment::BeastHead => {
            let (point, out) = on(PI / 2.0 - 0.15, 0.0);
            let snout = point + out * 26.0 - arm.down * 6.0;
            blob(model.of(Stuff::Trim), snout, Vec3::new(26.0, 22.0, 30.0));
            blob(
                model.of(Stuff::Trim),
                snout + out * 26.0 - arm.down * 4.0,
                Vec3::new(14.0, 12.0, 18.0),
            );
            for side in [-1.0, 1.0] {
                let eye = snout + out * 18.0 - arm.down * 14.0 + arm.forward * (side * 13.0);
                blob(model.of(Stuff::Etch), eye, Vec3::splat(5.0));
                let ear = snout - arm.down * 22.0 + arm.forward * (side * 16.0);
                limb(
                    model.of(Stuff::Trim),
                    &[(ear, 8.0), (ear - arm.down * 22.0 - out * 6.0, 1.0)],
                );
            }
        }
        Adornment::Skull => {
            let (point, out) = on(1.0, 0.0);
            let skull = point + out * 22.0;
            blob(model.of(Stuff::Bone), skull, Vec3::new(26.0, 28.0, 24.0));
            blob(
                model.of(Stuff::Bone),
                skull + out * 6.0 + arm.down * 26.0,
                Vec3::new(17.0, 10.0, 15.0),
            );
            for side in [-1.0, 1.0] {
                let eye = skull + out * 20.0 + arm.down * 4.0 + arm.forward * (side * 10.0);
                blob(model.of(Stuff::Etch), eye, Vec3::new(6.0, 7.0, 6.0));
            }
        }
    }
}

/// One shoulder's piece, in place on this frame.
fn build_side(form: &ShouldersForm, frame: &Frame, side: Side) -> Model {
    let mut model = Model::default();
    let arm = Arm::of(frame, side);
    let look = shaped(form);
    let scale = frame.elbow_length as f32 / 300.0;
    let mut height = 0.0;
    match form.piece {
        ShouldersPiece::Pauldrons {
            cut,
            wings,
            lames,
            upper_lames,
            haute_piece,
            gardbrace,
            besagews,
        } => {
            if lames == 0 {
                add_hanging_pauldron(&mut model, form, &arm, look);
            } else {
                // The right of a lance cut is cut away in front, under the arm.
                let range = if cut == Cut::LanceCut && side == Side::Right {
                    (-2.2, 0.7)
                } else {
                    (-2.2, 2.2)
                };
                add_cop_and_lames(&mut model, form, frame, &arm, look, range, lames, besagews);
                add_pauldron_extras(
                    &mut model,
                    frame,
                    &arm,
                    look,
                    wings,
                    cut,
                    upper_lames,
                    haute_piece,
                    gardbrace,
                );
                if gardbrace {
                    height = 9.0;
                }
            }
        }
        ShouldersPiece::Spaulders {
            lames,
            besagews,
            pteruges,
        } => {
            if form.dendra() {
                // One big plate over the shoulder, and one over the upper arm.
                let across = arm.radius + RIGID_CLEAR + 8.0;
                let cop = cop_surface(&arm, look, (-2.1, 2.1), (across, across - 6.0), PI / 2.0);
                sheet(&mut model, Stuff::Main, &cop, 2.4);
                let band = arm_surface(
                    &arm,
                    look,
                    (-2.0, 2.0),
                    (-10.0, arm.reach * 0.9),
                    across - 3.0,
                );
                sheet(&mut model, Stuff::Main, &band, 2.4);
            } else {
                add_cop_and_lames(
                    &mut model,
                    form,
                    frame,
                    &arm,
                    look,
                    (-1.85, 1.85),
                    lames,
                    besagews,
                );
            }
            if pteruges > 0 {
                add_pteruges(&mut model, &arm, pteruges, arm.radius + RIGID_CLEAR);
            }
        }
        ShouldersPiece::Guards { .. } => {
            add_guards(&mut model, form, frame, &arm);
            height = HANGING_CLEAR - RIGID_CLEAR;
        }
        ShouldersPiece::Discs { diameter, boss } => {
            add_shoulder_disc(&mut model, form, &arm, diameter as f32 * scale, boss);
            height = -6.0;
        }
        ShouldersPiece::Ailettes {
            shape,
            size,
            heraldry,
        } => {
            let (wide, high) = size.ailette();
            let size = (wide as f32 * scale, high as f32 * scale);
            add_ailette(&mut model, form, frame, &arm, shape, size, heraldry.charge);
        }
        // Both shoulders at once.
        ShouldersPiece::Cape { .. } | ShouldersPiece::Wing => {}
    }
    add_adornment(&mut model, form, &arm, height);
    model
}

/// Both shoulders' pieces, in place on this frame: for sets and a dressed character.
pub(in crate::item_art) fn build_pair(form: &ShouldersForm, frame: &Frame) -> Model {
    let mut model = Model::default();
    match form.piece {
        ShouldersPiece::Cape { cut, length } => match cut {
            CapeCut::Round => add_cape(&mut model, form, frame, length),
            CapeCut::Yoke => add_yoke(&mut model, form, frame, length),
        },
        ShouldersPiece::Wing => add_wing(&mut model, frame),
        _ => {
            for side in Side::BOTH {
                model.join(build_side(form, frame, side));
            }
        }
    }
    model
}

/// The pair built on an ordinary body for its picture: each piece turned to show its
/// outside and set beside the other, the right on the left as worn, tipped back a
/// little, then fitted to its cells. Capes and the wing are seen as worn.
fn fitted(look: &ShouldersLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = match look.form.piece {
        ShouldersPiece::Cape { .. } | ShouldersPiece::Wing => {
            let mut model = build_pair(&look.form, &frame);
            // The wing faces out to the left: turned to face the camera.
            let turn = Quat::from_rotation_y(if look.form.piece == ShouldersPiece::Wing {
                -1.25
            } else {
                TURN
            });
            model.map(|point| turn * point);
            model
        }
        _ => {
            let mut picture = Model::default();
            for side in Side::BOTH {
                let mut piece = build_side(&look.form, &frame, side);
                let sign = side.sign() as f32;
                let joint = Arm::of(&frame, side).joint;
                let turn = Quat::from_rotation_y(-sign * SHOW_TURN);
                piece.map(|point| turn * (point - joint));
                let (low, high) = piece.bounds();
                // Edge to edge, a little apart.
                let shift = if sign > 0.0 {
                    PICTURE_GAP / 2.0 - low.x
                } else {
                    -PICTURE_GAP / 2.0 - high.x
                };
                piece.map(|point| point + Vec3::X * shift);
                picture.join(piece);
            }
            picture
        }
    };
    let tilt = Quat::from_rotation_x(TILT);
    picture.map(|point| tilt * point);
    let (low, high) = picture.bounds();
    let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
    let world_per_pixel = VIEW_WIDTH / (wide.max(long) * PIXELS_PER_CELL);
    let room = |cells: f32| (cells * PIXELS_PER_CELL - 2.0 * MARGIN_PIXELS) * world_per_pixel;
    let size = high - low;
    let scale = (room(long) / size.y).min(room(wide) / size.x);
    let middle = (low + high) / 2.0;
    picture.map(|point| (point - middle) * scale);
    picture
}

/// What a tincture looks like.
fn tincture(tincture: Tincture, quality: Option<Quality>) -> StandardMaterial {
    match tincture {
        Tincture::Or => metal_material(Material::Gold, quality),
        Tincture::Argent => metal_material(Material::Silver, quality),
        Tincture::Gules => matte(Color::srgb(0.68, 0.1, 0.1), 0.7),
        Tincture::Azure => matte(Color::srgb(0.12, 0.24, 0.62), 0.7),
        Tincture::Vert => matte(Color::srgb(0.1, 0.42, 0.2), 0.7),
        Tincture::Sable => matte(Color::srgb(0.07, 0.07, 0.08), 0.7),
        Tincture::Purpure => matte(Color::srgb(0.42, 0.14, 0.46), 0.7),
    }
}

/// How each part looks, by what it is made of.
fn material_of(stuff: Stuff, form: &ShouldersForm, quality: Option<Quality>) -> StandardMaterial {
    let japanese = form.japanese();
    let main = || match form.material.category() {
        // Japanese lamellae are lacquered black.
        _ if japanese && form.construction != Construction::Mail => {
            matte(Color::srgb(0.1, 0.08, 0.07), 0.3)
        }
        Category::Metal => metal_material(form.material, quality),
        Category::Leather => matte(colour_of(form.material), 0.75),
        Category::Textile => matte(colour_of(form.material), 0.95),
        Category::Wood => matte(colour_of(form.material), 0.8),
        _ => matte(colour_of(form.material), 0.4),
    };
    let heraldry = match form.piece {
        ShouldersPiece::Ailettes { heraldry, .. } => Some(heraldry),
        _ => None,
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => matte(colour_of(form.cloth.unwrap_or(Material::Linen)), 0.9),
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather | Construction::PlatesInCloth => Material::Brass,
                _ => Material::Iron,
            },
            quality,
        ),
        Stuff::Etch => StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.24),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        },
        Stuff::Bone => matte(Color::srgb(0.86, 0.82, 0.7), 0.6),
        Stuff::Field => heraldry.map_or_else(main, |h| tincture(h.field, quality)),
        Stuff::Charge => heraldry.map_or_else(main, |h| tincture(h.colour, quality)),
    }
}

/// A pair of shoulder pieces, as worn, filling their picture.
pub(in crate::item_art) fn shoulders_parts(
    look: &ShouldersLook,
    quality: Option<Quality>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<Part> {
    let camera = Transform::from_translation(CAMERA_AT).looking_at(Vec3::ZERO, CAMERA_UP);
    fitted(look)
        .parts
        .into_iter()
        .filter(|(_, triangles)| !triangles.is_empty())
        .map(|(stuff, triangles)| Part {
            mesh: meshes.add(mesh_of(&triangles)),
            material: materials.add(material_of(stuff, &look.form, quality)),
            transform: Transform::from_rotation(camera.rotation),
        })
        .collect()
}

#[cfg(test)]
mod tests;
