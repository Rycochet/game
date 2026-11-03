//! Suns and moons above the minimap, on a circle as big as it, lifted a little:
//! so they rise from behind its edge, and are partly hidden for a while.

use bevy::prelude::*;
use shared::celestial::{BodyKind, BodyNow};

use super::canvas::{Canvas, polygon_distance};
use super::frame::RoundFrame;

/// How far their circle is lifted above the minimap's, in logical pixels at the
/// minimap's base size, and the room that leaves for them.
const LIFT: f32 = SUN_LARGEST * 1.5;
pub const BAND: f32 = LIFT + SUN_LARGEST;

/// The height (the up of a direction) by which one has reached its circle: below
/// it, it is sinking behind the map, wherever it is round the circle.
const RISEN: f32 = 0.25;

/// Radius of the smallest and largest, for their sizes in `shared::celestial`.
const SUN_SMALLEST: f32 = 7.0;
const SUN_LARGEST: f32 = 10.0;
const MOON_SMALLEST: f32 = 3.0;
const MOON_LARGEST: f32 = 5.0;

/// A sun's points, and how far the zig-zag comes in between them, against its
/// radius.
const SUN_POINTS: u32 = 8;
const SUN_INNER: f32 = 0.62;
/// A sun's rays, against its middle (linear).
const RAY_SHADE: f32 = 0.6;

const OUTLINE: f32 = 1.0;
const BLACK: [u8; 3] = [0, 0, 0];
/// The dark part of a moon, against the lit part.
const MOON_SHADOW: f32 = 0.2;
/// Extra pixels round one that count as over it, for fingers.
const HOVER_REACH: f32 = 3.0;

/// One to draw.
#[derive(Clone, Copy, Debug)]
pub struct Icon {
    /// Which of the bodies given.
    pub number: usize,
    pub at: Vec2,
    pub radius: f32,
}

/// How big one is, by its size: suns about `0.5..2`, moons `0.3..1`.
fn radius(now: &BodyNow) -> f32 {
    let along =
        |from: f64, to: f64| (((now.body.size - from) / (to - from)) as f32).clamp(0.0, 1.0);
    match now.body.kind {
        BodyKind::Sun => SUN_SMALLEST + (SUN_LARGEST - SUN_SMALLEST) * along(0.5, 2.0),
        BodyKind::Moon => MOON_SMALLEST + (MOON_LARGEST - MOON_SMALLEST) * along(0.3, 1.0),
    }
}

/// Where one is against the middle of a circle of `circle` pixels: across the
/// top by how far `sideways` it is against how high, and lifted by `lift`; low
/// down, drawn in behind the circle.
fn on_path(sideways: f32, up: f32, circle: f32, radius: f32, lift: f32) -> Vec2 {
    let risen = (up / RISEN).clamp(0.0, 1.0);
    let risen = risen * risen * (3.0 - 2.0 * risen);
    let round = Vec2::new(sideways, -up.max(0.0)).normalize_or(Vec2::NEG_Y);
    round * (circle - (1.0 - risen) * (radius + 1.0)) + Vec2::NEG_Y * lift * risen
}

/// Where those above the horizon go, above a circle of `circle` pixels at
/// `middle`, always left to right as they cross the sky, however the world turns
/// and wherever the camera looks; `trim` is pixels for each size given here.
pub fn icons(bodies: &[BodyNow], middle: Vec2, circle: f32, trim: f32) -> Vec<Icon> {
    let mut icons = Vec::new();
    for (number, now) in bodies.iter().enumerate().filter(|(_, now)| now.is_up()) {
        let radius = radius(now) * trim;
        let at = on_path(
            now.along as f32,
            now.direction[1] as f32,
            circle,
            radius,
            LIFT * trim,
        );
        icons.push(Icon {
            number,
            at: middle + at,
            radius,
        });
    }
    icons
}

fn srgb(colour: [f64; 3], shade: f32) -> [u8; 3] {
    let [red, green, blue] = colour.map(|channel| channel as f32 * shade);
    let [red, green, blue, _] = Color::linear_rgb(red, green, blue).to_srgba().to_u8_array();
    [red, green, blue]
}

/// Draws them in order, so moons go over suns, and only outside the map's outline
/// (its border and notch too): inside it they are behind the map.
pub fn draw(canvas: &mut Canvas, icons: &[Icon], bodies: &[BodyNow], map: &RoundFrame, trim: f32) {
    let behind_map = |x: f32, y: f32| -map.distance(x, y);
    for icon in icons {
        let now = &bodies[icon.number];
        match now.body.kind {
            BodyKind::Sun => draw_sun(canvas, *icon, now, trim, behind_map),
            BodyKind::Moon => draw_moon(canvas, *icon, now, trim, behind_map),
        }
    }
}

/// A star with a zig-zag edge, its rays darker than its middle.
fn draw_sun(
    canvas: &mut Canvas,
    icon: Icon,
    now: &BodyNow,
    trim: f32,
    behind_map: impl Fn(f32, f32) -> f32,
) {
    let (centre, reach) = (icon.at, icon.radius);
    let corners: Vec<(f32, f32)> = (0..SUN_POINTS * 2)
        .map(|corner| {
            let along = if corner % 2 == 0 { 1.0 } else { SUN_INNER };
            let angle = corner as f32 * std::f32::consts::PI / SUN_POINTS as f32;
            (centre + Vec2::from_angle(angle) * reach * along).into()
        })
        .collect();
    let distance = |x: f32, y: f32| polygon_distance(&corners, x, y).max(behind_map(x, y));
    let outline = OUTLINE * trim;
    canvas.shape(centre.into(), reach + outline, BLACK, |x, y| {
        (polygon_distance(&corners, x, y) - outline).max(behind_map(x, y))
    });
    canvas.shape(
        centre.into(),
        reach,
        srgb(now.body.colour, RAY_SHADE),
        distance,
    );
    canvas.shape(centre.into(), reach, srgb(now.body.colour, 1.0), |x, y| {
        ((Vec2::new(x, y) - centre).length() - reach * SUN_INNER).max(behind_map(x, y))
    });
}

/// A disc lit as far as its phase, from the right while waxing, from the left
/// while waning, whichever way the map is turned.
fn draw_moon(
    canvas: &mut Canvas,
    icon: Icon,
    now: &BodyNow,
    trim: f32,
    behind_map: impl Fn(f32, f32) -> f32,
) {
    let (centre, reach) = (icon.at, icon.radius);
    let disc = |x: f32, y: f32| (Vec2::new(x, y) - centre).length() - reach;
    let outline = OUTLINE * trim;
    canvas.shape(centre.into(), reach + outline, BLACK, |x, y| {
        (disc(x, y) - outline).max(behind_map(x, y))
    });
    canvas.shape(
        centre.into(),
        reach,
        srgb(now.body.colour, MOON_SHADOW),
        |x, y| disc(x, y).max(behind_map(x, y)),
    );
    let side = if now.waxing { 1.0 } else { -1.0 };
    // Where the edge of the lit part crosses each row of the disc.
    let edge = 1.0 - 2.0 * now.lit as f32;
    let lit = |x: f32, y: f32| {
        let at = Vec2::new(x, y) - centre;
        let half = (reach * reach - at.y * at.y).max(0.0).sqrt();
        disc(x, y)
            .max(edge * half - at.x * side)
            .max(behind_map(x, y))
    };
    canvas.shape(centre.into(), reach, srgb(now.body.colour, 1.0), lit);
}

/// The one under a point on the picture, if any: the last drawn first. Not one
/// behind the map.
pub fn under(icons: &[Icon], middle: Vec2, circle: f32, at: Vec2) -> Option<usize> {
    if at.distance(middle) <= circle {
        return None;
    }
    icons
        .iter()
        .rev()
        .find(|icon| icon.at.distance(at) <= icon.radius + HOVER_REACH)
        .map(|icon| icon.number)
}

/// What one is called: "Blue moon", or "Moon" if it is too near white for a name.
pub fn name(now: &BodyNow) -> String {
    let kind = match now.body.kind {
        BodyKind::Sun => "sun",
        BodyKind::Moon => "moon",
    };
    match now.body.colour_name {
        Some(colour) => format!("{colour:?} {kind}"),
        None => {
            let mut letters = kind.chars();
            let first = letters.next().map(|c| c.to_ascii_uppercase());
            first.into_iter().chain(letters).collect()
        }
    }
}

#[cfg(test)]
mod tests;
