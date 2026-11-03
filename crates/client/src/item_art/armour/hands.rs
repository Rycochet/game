//! Hand pieces: gauntlets of every construction, demi-gauntlets, tekkō and padded
//! gloves. Each hand is built in millimetres in place on a body frame, hanging from the
//! wrist with the palm towards the thigh and the thumb forwards, as the bare body draws
//! it; its picture lays both hands down side by side. A cuff sits 28 mm out from the
//! forearm, over the end of any vambrace (agreed with the arms' pieces).

use std::f32::consts::PI;

use bevy::prelude::*;
use shared::armour::hands::{Cuff, Fingers, Hand, HandsForm, Knuckles};
use shared::armour::{Construction, Style};
use shared::body::{Frame, Side};
use shared::creature::CreatureKind;
use shared::item::Quality;
use shared::material::{Category, Material};

use super::{Model, Stuff, colour_of, darker, matte, plate};
use crate::item_art::body::{limb, loft, mesh_of};
use crate::item_art::ring::metal_material;
use crate::item_art::{CAMERA_AT, CAMERA_UP, HandsLook, OUTLINE_PIXELS, Part, VIEW_WIDTH};

/// How many of a picture's pixels are one cell, for the margin's sake.
const PIXELS_PER_CELL: f32 = 64.0;
/// Pixels kept clear round it, for its black edge.
const MARGIN_PIXELS: f32 = OUTLINE_PIXELS + 2.0;
/// Tipped back a little, so the plates' sides show.
const TILT: f32 = 0.45;
/// Between the two hands in a picture.
const PICTURE_GAP: f32 = 12.0;
/// How far a cuff is out from the forearm's skin: past a vambrace's 20 mm.
const CUFF_CLEAR: f32 = 28.0;
/// Points round a cuff, and across a plate.
const ROUND: usize = 20;
const ACROSS: usize = 12;
/// How far each lame stands out over the next, so its edge shows.
const LAME_STEP: f32 = 1.4;

/// A hand's own directions: down to the fingertips, out of the back of the hand, and
/// forwards to the thumb's side.
struct Axes {
    wrist: Vec3,
    down: Vec3,
    out: Vec3,
    forward: Vec3,
}

impl Axes {
    fn of(hand: &Hand) -> Self {
        let point = |mm: [i32; 3]| Vec3::new(mm[0] as f32, mm[1] as f32, mm[2] as f32);
        let wrist = point(hand.wrist);
        let down = (point(hand.tip) - wrist).normalize();
        let side = Vec3::X * hand.side.sign() as f32;
        let out = (side - down * side.dot(down)).normalize();
        let forward = (Vec3::Z - down * Vec3::Z.dot(down) - out * Vec3::Z.dot(out)).normalize();
        Self {
            wrist,
            down,
            out,
            forward,
        }
    }

    /// So far down from the wrist, out from the hand's middle, and forwards.
    fn at(&self, down: f32, out: f32, forward: f32) -> Vec3 {
        self.wrist + self.down * down + self.out * out + self.forward * forward
    }
}

/// A hand's sizes as numbers to build with, and where its fingers are.
struct Shape {
    length: f32,
    palm: f32,
    breadth: f32,
    thick: f32,
    finger: f32,
    /// Each finger's line, knuckle to tip, from the thumb's side.
    fingers: Vec<(Vec3, Vec3)>,
    /// The thumb's line, root to tip.
    thumb_line: (Vec3, Vec3),
}

impl Shape {
    fn of(hand: &Hand, axes: &Axes) -> Self {
        let f = |mm: i32| mm as f32;
        let (length, palm, breadth, thick) = (
            f(hand.length),
            f(hand.palm),
            f(hand.breadth),
            f(hand.thickness),
        );
        let count = hand.fingers.max(1) as usize;
        let finger = breadth / (2.0 * count as f32) * 0.85;
        let reach = length - palm;
        let fingers = (0..count)
            .map(|at| {
                let spread = if count == 1 {
                    0.0
                } else {
                    (breadth / 2.0 - finger * 1.1) * (1.0 - 2.0 * at as f32 / (count - 1) as f32)
                };
                // The middle finger longest, the little one shortest.
                let middle = (count - 1) as f32 * 0.33;
                let span = (count - 1).max(1) as f32;
                let long = 1.0 - 0.32 * (at as f32 - middle).abs() / span;
                (
                    axes.at(palm, 0.0, spread),
                    axes.at(palm + reach * long - finger * 0.5, 0.0, spread * 0.92),
                )
            })
            .collect();
        let thumb_from = f(hand.thumb_from);
        let root = axes.at(thumb_from + 12.0, -thick * 0.15, breadth / 2.0 - 4.0);
        let way = (axes.down * 0.8 + axes.forward * 0.45 - axes.out * 0.25).normalize();
        Self {
            length,
            palm,
            breadth,
            thick,
            finger,
            fingers,
            thumb_line: (root, root + way * f(hand.thumb)),
        }
    }

    /// How long the longest finger is, from the knuckle.
    fn reach(&self) -> f32 {
        self.length - self.palm
    }
}

/// One way across a line: the out and forward directions square to it.
fn across(along: Vec3, axes: &Axes) -> (Vec3, Vec3) {
    let out = (axes.out - along * axes.out.dot(along)).normalize();
    (
        out,
        along.cross(out).normalize() * along.cross(out).dot(axes.forward).signum(),
    )
}

/// Points round a line between two angles from its out side, `radii` out and forwards,
/// with `bump` adding to the radius by angle (flutes).
struct Arc {
    from: Vec3,
    to: Vec3,
    /// Radius out and forwards at each end.
    start: (f32, f32),
    end: (f32, f32),
    angles: (f32, f32),
    rows: usize,
}

fn arc_grid(arc: &Arc, axes: &Axes, bump: &dyn Fn(f32) -> f32) -> Vec<Vec<(Vec3, Vec3)>> {
    let along = (arc.to - arc.from).normalize();
    let (out, forward) = across(along, axes);
    (0..=arc.rows)
        .map(|row| {
            let t = row as f32 / arc.rows as f32;
            let centre = arc.from.lerp(arc.to, t);
            let radius_out = arc.start.0 + (arc.end.0 - arc.start.0) * t;
            let radius_forward = arc.start.1 + (arc.end.1 - arc.start.1) * t;
            (0..=ACROSS)
                .map(|column| {
                    let angle = arc.angles.0
                        + (arc.angles.1 - arc.angles.0) * column as f32 / ACROSS as f32;
                    let grow = bump(angle);
                    let point = centre
                        + out * angle.cos() * (radius_out + grow)
                        + forward * angle.sin() * (radius_forward + grow);
                    let normal = (out * angle.cos() / radius_out.max(1.0)
                        + forward * angle.sin() / radius_forward.max(1.0))
                    .normalize();
                    (point, normal)
                })
                .collect()
        })
        .collect()
}

fn add_arc(model: &mut Model, stuff: Stuff, arc: &Arc, axes: &Axes, thick: f32) {
    add_arc_bumped(model, stuff, arc, axes, thick, &|_| 0.0);
}

fn add_arc_bumped(
    model: &mut Model,
    stuff: Stuff,
    arc: &Arc,
    axes: &Axes,
    thick: f32,
    bump: &dyn Fn(f32) -> f32,
) {
    plate(model.of(stuff), &arc_grid(arc, axes, bump), thick, false);
}

/// A flat ring round the hand's middle line at a distance down from the wrist.
fn ellipse(axes: &Axes, down: f32, half_out: f32, half_forward: f32) -> Vec<Vec3> {
    (0..ROUND)
        .map(|at| {
            let angle = at as f32 / ROUND as f32 * 2.0 * PI;
            axes.at(down, half_out * angle.cos(), half_forward * angle.sin())
        })
        .collect()
}

/// How a style shapes plates.
struct Look {
    thick: f32,
    /// Flutes round a cuff and across the back, and how deep.
    flutes: (f32, f32),
    /// How far the cuff's top is drawn out to a point on the outside.
    point: f32,
    cusps: bool,
    rim: f32,
    square: bool,
    pointed_tips: bool,
}

fn look_of(style: Style, cuff: Cuff) -> Look {
    let base = Look {
        thick: 2.0,
        flutes: (0.0, 0.0),
        point: if cuff == Cuff::Pointed { 40.0 } else { 0.0 },
        cusps: false,
        rim: 2.2,
        square: false,
        pointed_tips: false,
    };
    match style {
        // Regional styles shape their own constructions, not plate.
        Style::White
        | Style::Greenwich
        | Style::Japanese
        | Style::Chinese
        | Style::Steppe
        | Style::Classical
        | Style::Byzantine
        | Style::IndoPersian => base,
        Style::Milanese => Look { rim: 2.8, ..base },
        Style::Gothic => Look {
            flutes: (6.0, 3.0),
            point: base.point.max(25.0),
            cusps: true,
            rim: 0.0,
            pointed_tips: true,
            ..base
        },
        Style::Maximilian => Look {
            flutes: (12.0, 2.5),
            ..base
        },
        Style::Kastenbrust => Look {
            square: true,
            ..base
        },
        Style::Elven => Look {
            thick: 1.4,
            point: base.point.max(45.0),
            rim: 0.0,
            pointed_tips: true,
            ..base
        },
        Style::Dwarven => Look {
            thick: 3.2,
            rim: 3.5,
            ..base
        },
    }
}

/// Whether plates are drawn with the style's shapes: plate, and the plates of the
/// others that have them.
fn styled(form: &HandsForm) -> bool {
    matches!(
        form.construction,
        Construction::Plate | Construction::PlatesInCloth | Construction::MailAndPlate
    )
}

fn flute(look: &Look) -> impl Fn(f32) -> f32 + '_ {
    move |angle: f32| {
        let (count, deep) = look.flutes;
        if count == 0.0 {
            0.0
        } else {
            deep * (angle * count).cos().max(0.0).powi(2)
        }
    }
}

/// The cuff: round the wrist and up the forearm, flaring by its kind, over a vambrace.
fn add_cuff(
    model: &mut Model,
    form: &HandsForm,
    hand: &Hand,
    frame: &Frame,
    axes: &Axes,
    stuff: Stuff,
) {
    let look = look_of(form.style, form.cuff);
    let plain = !styled(form);
    let reach = form.cuff.reach(frame) as f32;
    let reach = if form.style == Style::Dwarven && !plain {
        reach * 0.8
    } else {
        reach
    };
    let wrist = hand.wrist_radius as f32 + CUFF_CLEAR;
    let flare = form.cuff.flare() as f32 / 100.0;
    let flare = if form.style == Style::Dwarven {
        flare * 1.3
    } else {
        flare
    };
    // How far up, by how far round, from 0 at the bottom to 1 at the top.
    let rows: &[(f32, f32)] = match form.cuff {
        Cuff::Hourglass => &[
            (-0.2, 1.2),
            (0.0, 1.0),
            (0.5, 1.0 + flare * 0.4),
            (1.0, 1.0 + flare),
        ],
        Cuff::Wrist => &[(-0.2, 1.0), (1.0, 1.0)],
        _ => &[
            (-0.15, 1.05),
            (0.0, 1.0),
            (0.4, 1.0 + flare * 0.3),
            (1.0, 1.0 + flare),
        ],
    };
    let (point, cusps, square) = if plain {
        (
            if form.cuff == Cuff::Pointed {
                40.0
            } else {
                0.0
            },
            false,
            false,
        )
    } else {
        (look.point, look.cusps, look.square)
    };
    let top = |angle: f32| {
        let pointed = point * angle.cos().max(0.0).powi(6);
        let cusped = if cusps {
            6.0 * (angle * 3.0).sin().abs()
        } else {
            0.0
        };
        -reach - pointed + cusped
    };
    let bump = flute(&look);
    let grid: Vec<Vec<(Vec3, Vec3)>> = rows
        .iter()
        .map(|&(up, widen)| {
            (0..ROUND)
                .map(|at| {
                    let angle = at as f32 / ROUND as f32 * 2.0 * PI;
                    // Below the wrist into the hand, or up the forearm to the top.
                    let down = if up < 0.0 {
                        -up * reach
                    } else {
                        top(angle) * up
                    };
                    let squared = if square {
                        let (c, s) = (angle.cos().abs(), angle.sin().abs());
                        (c.powi(4) + s.powi(4)).powf(-0.25)
                    } else {
                        1.0
                    };
                    let flutes = if plain { 0.0 } else { bump(angle) };
                    let radius = wrist * widen * squared + flutes;
                    let way = axes.out * angle.cos() + axes.forward * angle.sin();
                    (axes.at(down, 0.0, 0.0) + way * radius, way)
                })
                .collect()
        })
        .collect();
    let thick = if plain { 2.0 } else { look.thick };
    plate(model.of(stuff), &grid, thick, true);
    // A rolled rim round the top, or trim.
    let rim = match form.trim {
        Some(_) => Some((Stuff::Trim, 2.4)),
        None if !plain && look.rim > 0.0 => Some((stuff, look.rim)),
        None => None,
    };
    if let Some((rim_stuff, radius)) = rim {
        let last = grid.last().expect("a cuff has rows");
        let mut path: Vec<(Vec3, f32)> = last
            .iter()
            .map(|&(point, way)| (point + way * thick, radius))
            .collect();
        path.push(path[0]);
        limb(model.of(rim_stuff), &path);
    }
    if form.style == Style::Greenwich && !plain {
        for (from, to) in [(0.3, 0.45), (0.65, 0.8)] {
            let band: Vec<Vec<(Vec3, Vec3)>> = [from, to]
                .iter()
                .map(|&up: &f32| {
                    (0..ROUND)
                        .map(|at| {
                            let angle = at as f32 / ROUND as f32 * 2.0 * PI;
                            let widen = 1.0 + flare * up;
                            let way = axes.out * angle.cos() + axes.forward * angle.sin();
                            let radius = wrist * widen + thick * 0.8;
                            (axes.at(top(angle) * up, 0.0, 0.0) + way * radius, way)
                        })
                        .collect()
                })
                .collect();
            plate(model.of(Stuff::Etch), &band, 0.6, true);
        }
    }
}

/// The plate over the back of the hand, and the one over the knuckles.
fn add_back(
    model: &mut Model,
    form: &HandsForm,
    shape: &Shape,
    axes: &Axes,
    stuff: Stuff,
    spread: f32,
) {
    let look = look_of(form.style, form.cuff);
    let thick = if styled(form) { look.thick } else { 2.0 };
    let half_thick = shape.thick / 2.0;
    let bump = flute(&look);
    let no_bump = |_: f32| 0.0;
    let fluted: &dyn Fn(f32) -> f32 = if styled(form) { &bump } else { &no_bump };
    let back = Arc {
        from: axes.at(6.0, 0.0, 0.0),
        to: axes.at(shape.palm - 8.0, 0.0, 0.0),
        start: (half_thick + 4.0, shape.breadth * 0.4 + 4.0),
        end: (half_thick + 4.0, shape.breadth / 2.0 + 4.0),
        angles: (-spread, spread),
        rows: 6,
    };
    add_arc_bumped(model, stuff, &back, axes, thick, fluted);
    let knuckles = Arc {
        from: axes.at(shape.palm - 14.0, 0.0, 0.0),
        to: axes.at(shape.palm + 12.0, 0.0, 0.0),
        start: (half_thick + 7.0, shape.breadth / 2.0 + 6.0),
        end: (half_thick + 7.0, shape.breadth / 2.0 + 6.0),
        angles: (-spread, spread),
        rows: 3,
    };
    add_arc(model, stuff, &knuckles, axes, thick);
    if form.style == Style::Greenwich && styled(form) {
        let band = Arc {
            start: (back.start.0 + thick * 0.8, back.start.1),
            end: (back.end.0 + thick * 0.8, back.end.1),
            angles: (-0.25, 0.25),
            ..back
        };
        add_arc(model, Stuff::Etch, &band, axes, 0.6);
    }
}

/// Plates over each finger, or across all of them for a mitten, and over the thumb.
fn add_lames(
    model: &mut Model,
    form: &HandsForm,
    shape: &Shape,
    axes: &Axes,
    stuff: Stuff,
    gap: f32,
) {
    let look = look_of(form.style, form.cuff);
    let thick = if styled(form) { look.thick } else { 2.0 };
    let tips = styled(form) && look.pointed_tips;
    match form.fingers {
        Fingers::Fingered => {
            for &(knuckle, tip) in &shape.fingers {
                lames_along(
                    model,
                    stuff,
                    axes,
                    (knuckle, tip),
                    shape.finger + 2.0,
                    thick,
                    3,
                    gap,
                    tips,
                    (1.85, 1.85),
                );
            }
        }
        Fingers::Mitten => {
            let end = shape.palm + shape.reach() * 0.85;
            let curl = if form.locking { 2.9 } else { 1.8 };
            let count = 5;
            let span = (end - shape.palm - 8.0) / count as f32;
            for lame in 0..count {
                let from = shape.palm + 8.0 + span * lame as f32 - 2.0;
                let last = lame == count - 1;
                let to = from + span + 4.0;
                // Each lame steps out over the next and flares at its lower edge, so the
                // edges show; the whole narrows a little to the fingertips.
                let narrow = 1.0 - 0.12 * lame as f32 / count as f32;
                let step = lame as f32 * LAME_STEP;
                let radius = (
                    shape.thick * 0.4 + 3.0 + step,
                    (shape.breadth / 2.0 + 3.0) * narrow + step,
                );
                let arc = Arc {
                    from: axes.at(from, 0.0, 0.0),
                    to: axes.at(to, 0.0, 0.0),
                    start: radius,
                    end: if last {
                        (radius.0 * 0.4, radius.1 * 0.7)
                    } else {
                        (radius.0 + LAME_STEP, radius.1 + LAME_STEP)
                    },
                    angles: if last { (-curl, curl) } else { (-1.8, 1.8) },
                    rows: if last { 3 } else { 1 },
                };
                add_arc(model, stuff, &arc, axes, thick);
            }
        }
        Fingers::Open => return,
    }
    lames_along(
        model,
        stuff,
        axes,
        shape.thumb_line,
        shape.finger * 1.15 + 2.0,
        thick,
        2,
        gap,
        tips,
        (1.85, 1.85),
    );
}

/// Overlapping plates down a finger, the last rounded (or pointed) over the tip.
#[allow(clippy::too_many_arguments)]
fn lames_along(
    model: &mut Model,
    stuff: Stuff,
    axes: &Axes,
    (root, tip): (Vec3, Vec3),
    radius: f32,
    thick: f32,
    count: usize,
    gap: f32,
    pointed: bool,
    angles: (f32, f32),
) {
    let along = tip - root;
    let step = 1.0 / count as f32;
    for lame in 0..count {
        let last = lame == count - 1;
        let from = root + along * (step * lame as f32 + if lame == 0 { 0.08 } else { 0.0 });
        let to = root + along * (step * (lame + 1) as f32) + along.normalize() * (gap + 3.0);
        let r = radius + lame as f32 * LAME_STEP;
        let arc = Arc {
            from,
            to: if last && pointed {
                to + along.normalize() * 10.0
            } else {
                to
            },
            start: (r, r),
            end: if last {
                (r * if pointed { 0.15 } else { 0.5 }, r * 0.6)
            } else {
                (r + LAME_STEP, r + LAME_STEP)
            },
            angles: (-angles.0, angles.1),
            rows: if last { 3 } else { 1 },
        };
        add_arc(model, stuff, &arc, axes, thick);
    }
}

/// A glove under everything, or the whole piece for mail, leather and padding:
/// palm, fingers (or one bag for them) and thumb. `bands` shades every other strip of
/// that many millimetres, as rows of mail or quilting.
fn add_glove(
    model: &mut Model,
    shape: &Shape,
    axes: &Axes,
    mitten: bool,
    swell: f32,
    stuff: Stuff,
    bands: Option<f32>,
) {
    let mut rings = vec![
        (-6.0, shape.thick * 0.5, shape.breadth * 0.38),
        (shape.palm * 0.5, shape.thick / 2.0, shape.breadth * 0.47),
        (shape.palm + 4.0, shape.thick * 0.45, shape.breadth / 2.0),
    ];
    if mitten {
        let end = shape.palm + shape.reach() * 0.85;
        rings.extend([
            (end - 20.0, shape.thick * 0.38, shape.breadth / 2.0),
            (end - 6.0, shape.thick * 0.3, shape.breadth * 0.42),
            (end, shape.thick * 0.1, shape.breadth * 0.2),
        ]);
    }
    let rings: Vec<(f32, f32, f32)> = rings
        .into_iter()
        .map(|(down, out, forward)| (down, out + swell, forward + swell))
        .collect();
    let lines = if mitten {
        vec![shape.thumb_line]
    } else {
        let mut lines = shape.fingers.clone();
        lines.push(shape.thumb_line);
        lines
    };
    let band = bands.unwrap_or(f32::MAX);
    // The palm, in strips.
    let mut strip = 0;
    for pair in rings.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let steps = ((to.0 - from.0) / band).ceil().max(1.0) as usize;
        for step in 0..steps {
            let lerp = |t: f32| {
                let at = |a: f32, b: f32| a + (b - a) * t;
                ellipse(axes, at(from.0, to.0), at(from.1, to.1), at(from.2, to.2))
            };
            let (a, b) = (step as f32 / steps as f32, (step + 1) as f32 / steps as f32);
            let stuff = if strip % 2 == 1 { Stuff::Shade } else { stuff };
            loft(model.of(stuff), &[lerp(a), lerp(b)]);
            strip += 1;
        }
    }
    // Fingers and thumb, likewise.
    for (at, &(root, tip)) in lines.iter().enumerate() {
        let thumb = at == lines.len() - 1;
        let radius = shape.finger * if thumb { 1.15 } else { 1.0 } + swell;
        let length = root.distance(tip);
        let steps = (length / band).ceil().max(1.0) as usize;
        for step in 0..steps {
            let (a, b) = (step as f32 / steps as f32, (step + 1) as f32 / steps as f32);
            let narrow = |t: f32| radius * (1.0 - 0.15 * t);
            let stuff = if step % 2 == 1 { Stuff::Shade } else { stuff };
            limb(
                model.of(stuff),
                &[
                    (root.lerp(tip, a), narrow(a)),
                    (root.lerp(tip, b), narrow(b)),
                ],
            );
        }
    }
}

/// Small round heads in rows over the back of the hand.
fn add_rivets(model: &mut Model, shape: &Shape, axes: &Axes, out: f32, stuff: Stuff) {
    for row in 0..3 {
        let down = 14.0 + row as f32 * (shape.palm - 24.0) / 2.0;
        for column in -1..=1 {
            let angle = column as f32 * 0.7;
            let at = axes.at(
                down,
                (shape.thick / 2.0 + out) * angle.cos(),
                shape.breadth * 0.45 * angle.sin(),
            );
            let way = (axes.out * angle.cos() + axes.forward * angle.sin()).normalize();
            limb(
                model.of(stuff),
                &[(at - way * 1.0, 3.0), (at + way * 1.5, 2.0)],
            );
        }
    }
}

/// Spikes or studs on the knuckles, a bar, a disc, claws.
fn add_extras(model: &mut Model, form: &HandsForm, shape: &Shape, axes: &Axes) {
    let out = shape.thick / 2.0 + 9.0;
    let knuckle_lines: Vec<Vec3> = shape.fingers.iter().map(|&(knuckle, _)| knuckle).collect();
    match form.knuckles {
        Knuckles::Plain => {}
        Knuckles::Spiked if form.construction == Construction::Leather => {
            for &knuckle in &knuckle_lines {
                let at = knuckle + axes.out * (shape.thick / 2.0 + 2.0);
                limb(
                    model.of(Stuff::Rivet),
                    &[(at, 4.0), (at + axes.out * 3.0, 2.5)],
                );
            }
        }
        Knuckles::Spiked => {
            for &knuckle in &knuckle_lines {
                let at = knuckle + axes.out * out;
                limb(
                    model.of(Stuff::Main),
                    &[(at, 4.5), (at + axes.out * 18.0, 0.6)],
                );
            }
        }
        Knuckles::Ridged => {
            let bar = if form.style == Style::Dwarven {
                6.0
            } else {
                3.5
            };
            let half = shape.breadth / 2.0;
            limb(
                model.of(Stuff::Main),
                &[
                    (axes.at(shape.palm - 2.0, out + 1.0, half), bar),
                    (axes.at(shape.palm - 2.0, out + 2.0, 0.0), bar),
                    (axes.at(shape.palm - 2.0, out + 1.0, -half), bar),
                ],
            );
        }
        Knuckles::Rondel => {
            let centre = axes.at(shape.palm * 0.5, shape.thick / 2.0 + 8.0, 0.0);
            limb(
                model.of(Stuff::Main),
                &[(centre, 24.0), (centre + axes.out * 3.0, 24.0)],
            );
        }
    }
    if form.claws {
        for &(_, tip) in &shape.fingers {
            let along = (tip - shape.fingers[0].0).normalize();
            let r = shape.finger * 0.55;
            limb(
                model.of(Stuff::Main),
                &[
                    (tip, r),
                    (tip + along * 20.0 - axes.out * 6.0, r * 0.6),
                    (tip + along * 32.0 - axes.out * 16.0, 0.5),
                ],
            );
        }
    }
}

/// A tekkō: a plate over the back of the hand on cloth, tied on by a loop round the
/// middle finger and a band round the wrist.
fn add_tekko(model: &mut Model, shape: &Shape, axes: &Axes) {
    let base = Arc {
        from: axes.at(-8.0, 0.0, 0.0),
        to: axes.at(shape.palm + 8.0, 0.0, 0.0),
        start: (shape.thick / 2.0 + 2.0, shape.breadth * 0.38),
        end: (shape.thick / 2.0 + 2.0, shape.breadth / 2.0),
        angles: (-1.4, 1.4),
        rows: 4,
    };
    add_arc(model, Stuff::Cloth, &base, axes, 2.0);
    let top = Arc {
        from: axes.at(6.0, 0.0, 0.0),
        to: axes.at(shape.palm - 4.0, 0.0, 0.0),
        start: (shape.thick / 2.0 + 5.0, shape.breadth * 0.3),
        end: (shape.thick / 2.0 + 5.0, shape.breadth * 0.42),
        angles: (-1.0, 1.0),
        rows: 4,
    };
    add_arc(model, Stuff::Main, &top, axes, 2.0);
    // The loop, round the middle finger.
    let middle = shape.fingers[shape.fingers.len() / 3];
    let along = (middle.1 - middle.0).normalize();
    let (out, forward) = across(along, axes);
    let centre = middle.0 + along * 18.0;
    let mut path: Vec<(Vec3, f32)> = (0..12)
        .map(|at| {
            let angle = at as f32 / 12.0 * 2.0 * PI;
            (
                centre + (out * angle.cos() + forward * angle.sin()) * (shape.finger + 3.0),
                2.5,
            )
        })
        .collect();
    path.push(path[0]);
    limb(model.of(Stuff::Cloth), &path);
}

/// Rows of small plates over the back of the hand and down each finger.
fn add_scales(model: &mut Model, shape: &Shape, axes: &Axes) {
    let rows = ((shape.palm + 8.0) / 11.0) as usize;
    for row in 0..rows {
        let down = 2.0 + row as f32 * 11.0;
        let lift = (rows - row) as f32 * 0.5;
        for column in -4..4 {
            let angle = column as f32 * 0.32 + if row % 2 == 1 { 0.16 } else { 0.0 };
            let arc = Arc {
                from: axes.at(down, 0.0, 0.0),
                to: axes.at(down + 15.0, 0.0, 0.0),
                start: (
                    shape.thick / 2.0 + 3.0 + lift,
                    shape.breadth / 2.0 + 3.0 + lift,
                ),
                end: (
                    shape.thick / 2.0 + 3.0 + lift,
                    shape.breadth / 2.0 + 3.0 + lift,
                ),
                angles: (angle - 0.2, angle + 0.2),
                rows: 1,
            };
            add_arc(model, Stuff::Main, &arc, axes, 1.5);
        }
    }
    let mut lines = shape.fingers.clone();
    lines.push(shape.thumb_line);
    for (root, tip) in lines {
        let length = root.distance(tip);
        let count = (length / 10.0) as usize;
        for at in 0..count {
            let from = root.lerp(tip, at as f32 / count as f32);
            let to = from + (tip - root).normalize() * 14.0;
            let r = shape.finger + 2.5 + (count - at) as f32 * 0.3;
            let arc = Arc {
                from,
                to,
                start: (r, r),
                end: (r, r),
                angles: (-1.4, 1.4),
                rows: 1,
            };
            add_arc(model, Stuff::Main, &arc, axes, 1.5);
        }
    }
}

/// One hand of a pair, in place on its frame.
pub(in crate::item_art) fn build_hand(
    form: &HandsForm,
    frame: &Frame,
    side: Side,
    digits: u8,
) -> Model {
    let hand = Hand::of(frame, side, digits);
    let axes = Axes::of(&hand);
    let shape = Shape::of(&hand, &axes);
    let mut model = Model::default();
    let mitten = form.fingers == Fingers::Mitten;
    match form.construction {
        Construction::Padded => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                false,
                4.0,
                Stuff::Main,
                Some(14.0),
            );
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::Mail => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                mitten,
                2.5,
                Stuff::Main,
                Some(6.0),
            );
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::Leather => {
            add_glove(&mut model, &shape, &axes, mitten, 2.0, Stuff::Main, None);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
            if form.knuckles == Knuckles::Spiked {
                add_rivets(&mut model, &shape, &axes, 2.5, Stuff::Rivet);
            }
        }
        Construction::Scale => {
            add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Backing);
            add_scales(&mut model, &shape, &axes);
        }
        Construction::MailAndPlate if form.fingers == Fingers::Open => {
            add_tekko(&mut model, &shape, &axes);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Cloth);
        }
        Construction::MailAndPlate => {
            add_glove(
                &mut model,
                &shape,
                &axes,
                false,
                2.5,
                Stuff::Main,
                Some(6.0),
            );
            add_back(&mut model, form, &shape, &axes, Stuff::Main, 1.2);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, -4.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
        Construction::PlatesInCloth => {
            add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            add_back(&mut model, form, &shape, &axes, Stuff::Cloth, 1.7);
            add_rivets(&mut model, &shape, &axes, 7.0, Stuff::Rivet);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, 0.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Cloth);
        }
        _ => {
            if form.fingers != Fingers::Open {
                add_glove(&mut model, &shape, &axes, mitten, 1.5, Stuff::Backing, None);
            }
            add_back(&mut model, form, &shape, &axes, Stuff::Main, 1.75);
            add_lames(&mut model, form, &shape, &axes, Stuff::Main, 0.0);
            add_cuff(&mut model, form, &hand, frame, &axes, Stuff::Main);
        }
    }
    add_extras(&mut model, form, &shape, &axes);
    model
}

/// Both hands of a pair, in place on this frame: for sets and a dressed character.
// Waiting for those: only the tests use it so far.
#[allow(dead_code)]
pub(in crate::item_art) fn build_pair(form: &HandsForm, frame: &Frame, digits: u8) -> Model {
    let mut model = Model::default();
    for side in Side::BOTH {
        model.join(build_hand(form, frame, side, digits));
    }
    model
}

/// Both hands built on an ordinary body, laid down for their picture: fingers to the
/// right, backs up, the right hand above the left, then fitted to the cells.
fn fitted(look: &HandsLook) -> Model {
    let frame = Frame::of(CreatureKind::Human).expect("people have a frame");
    let mut picture = Model::default();
    for side in Side::BOTH {
        let mut hand = build_hand(&look.form, &frame, side, look.digits);
        let axes = Axes::of(&Hand::of(&frame, side, look.digits));
        // Down goes right, out comes towards the camera.
        let turn = Mat3::from_cols(axes.down, axes.out.cross(axes.down), axes.out).transpose();
        hand.map(|point| turn * (point - axes.wrist));
        let (low, high) = hand.bounds();
        // Thumbs point away from each other: the left one turns down.
        let shift = match side {
            Side::Right => PICTURE_GAP / 2.0 - low.y,
            Side::Left => -PICTURE_GAP / 2.0 - high.y,
        };
        hand.map(|point| point + Vec3::Y * shift);
        picture.join(hand);
    }
    let tilt = Quat::from_rotation_x(-TILT);
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

/// How each part looks, by what it is made of.
pub(in crate::item_art) fn material_of(
    stuff: Stuff,
    form: &HandsForm,
    quality: Option<Quality>,
) -> StandardMaterial {
    let main = || match form.material.category() {
        Category::Metal => metal_material(form.material, quality),
        Category::Leather => matte(colour_of(form.material), 0.75),
        Category::Textile => matte(colour_of(form.material), 0.95),
        _ => matte(colour_of(form.material), 0.4),
    };
    match stuff {
        Stuff::Main => main(),
        Stuff::Shade => darker(main()),
        Stuff::Backing => matte(colour_of(Material::Leather), 0.8),
        Stuff::Cloth => matte(colour_of(form.cloth.unwrap_or(Material::Linen)), 0.95),
        Stuff::Trim => metal_material(form.trim.unwrap_or(Material::Brass), quality),
        Stuff::Rivet => metal_material(
            match form.construction {
                Construction::Leather => Material::Brass,
                _ => Material::Iron,
            },
            quality,
        ),
        Stuff::Field | Stuff::Charge | Stuff::Bone => main(),
        Stuff::Etch => StandardMaterial {
            base_color: Color::srgb(0.14, 0.16, 0.24),
            metallic: 0.5,
            perceptual_roughness: 0.4,
            ..default()
        },
    }
}

/// A pair of hand pieces, laid down to fill their picture.
pub(in crate::item_art) fn hands_parts(
    look: &HandsLook,
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
mod tests {
    use super::*;
    use shared::armour::hands::{HandsPiece, roll};
    use shared::rng::Seed;

    fn looks() -> impl Iterator<Item = HandsLook> {
        let pieces = [HandsPiece::Gauntlets, HandsPiece::Demi, HandsPiece::Padded];
        pieces.into_iter().flat_map(|piece| {
            (0..300u64).map(move |seed| HandsLook {
                form: roll(&mut Seed::new(seed).rng(), piece, None),
                digits: [5, 4][seed as usize % 2],
                cells: (2, 1),
            })
        })
    }

    #[test]
    fn every_pair_stays_inside_its_picture_with_room_for_its_edge() {
        for look in looks() {
            let (wide, long) = (look.cells.0 as f32, look.cells.1 as f32);
            let per_cell = VIEW_WIDTH / wide.max(long);
            let edge = OUTLINE_PIXELS / PIXELS_PER_CELL * per_cell;
            let model = fitted(&look);
            assert!(!model.parts.is_empty());
            for corner in model.parts.iter().flat_map(|(_, t)| t.iter().flatten()) {
                assert!(corner.is_finite(), "{look:?}");
                assert!(
                    corner.x.abs() + edge <= wide * per_cell / 2.0 + 0.001,
                    "{look:?}"
                );
                assert!(
                    corner.y.abs() + edge <= long * per_cell / 2.0 + 0.001,
                    "{look:?}"
                );
            }
        }
    }

    #[test]
    fn worn_pieces_are_at_the_hands() {
        let frame = Frame::of(CreatureKind::Human).unwrap();
        for look in looks().take(300) {
            let model = build_pair(&look.form, &frame, 5);
            for side in Side::BOTH {
                let hand = Hand::of(&frame, side, 5);
                let wrist = Vec3::new(
                    hand.wrist[0] as f32,
                    hand.wrist[1] as f32,
                    hand.wrist[2] as f32,
                );
                // Everything on this side is within reach of its wrist.
                let near = model
                    .parts
                    .iter()
                    .flat_map(|(_, t)| t.iter().flatten())
                    .filter(|corner| corner.x.signum() == side.sign() as f32);
                for corner in near {
                    assert!(corner.distance(wrist) < 320.0, "{look:?}");
                }
            }
        }
    }
}
