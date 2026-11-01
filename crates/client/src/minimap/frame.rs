//! What is drawn over a map's contents: the round minimap's ridged border and
//! north notch, the full map's compass in the same style, the player's dot,
//! and the arrow pointing to a dot off the map.

use super::canvas::{Canvas, polygon_distance};

const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];

/// The ridged border's middle colour, and how far its sides go lighter and
/// darker from it.
const BORDER: [f32; 3] = [150.0, 115.0, 70.0];
const RIDGE_STRENGTH: f32 = 0.55;

/// The direction light comes from, for the ridge: the top left of the screen,
/// whichever way the map is turned.
const LIGHT: (f32, f32) = (
    -std::f32::consts::FRAC_1_SQRT_2,
    -std::f32::consts::FRAC_1_SQRT_2,
);

/// A stairs icon's steps and colour.
const STAIRS_STEPS: u32 = 4;
const STAIRS_INK: [u8; 3] = [70, 55, 40];

/// The arrow beside the player's mark on another floor, against its height: its
/// shaft's width, its head's width and height, its black edge and its gap from
/// the mark (a little more than the edge, so they nearly touch).
const HINT_SHAFT: f32 = 0.34;
const HINT_HEAD_WIDTH: f32 = 0.95;
const HINT_HEAD_HEIGHT: f32 = 0.5;
const HINT_EDGE: f32 = 0.1;
const HINT_GAP: f32 = 0.13;

/// How wide the notch's base is, against how long it is, and how far along the
/// border it spreads as it grows out of it, against the same.
const NOTCH_BASE: f32 = 0.6;
const NOTCH_SPREAD: f32 = 0.5;

/// The ridge's colour at a point: lit on one side going out from its middle,
/// on the other side coming in, so it stands up from the map. `facing` is how
/// much the way out faces the light, from -1 to 1.
fn ridge_colour(facing: f32, outer: bool) -> [u8; 3] {
    let shade = 1.0 + RIDGE_STRENGTH * if outer { facing } else { -facing };
    BORDER.map(|channel| (channel * shade).clamp(0.0, 255.0) as u8)
}

/// The minimap's outline around `centre`: a circle of `radius`, with a notch from
/// the inside of its border to `notch` beyond it, towards `north` (a unit vector
/// on the picture).
#[derive(Clone, Copy, Debug)]
pub struct RoundFrame {
    pub centre: (f32, f32),
    pub radius: f32,
    pub border: f32,
    pub notch: f32,
    pub north: (f32, f32),
}

impl RoundFrame {
    fn notch_corners(&self) -> [(f32, f32); 3] {
        let (north_x, north_y) = self.north;
        let (across_x, across_y) = (-north_y, north_x);
        let base = self.radius - self.border;
        let half = (self.border + self.notch) * NOTCH_BASE;
        let at = |along: f32, across: f32| {
            (
                self.centre.0 + north_x * along + across_x * across,
                self.centre.1 + north_y * along + across_y * across,
            )
        };
        [
            at(self.radius + self.notch, 0.0),
            at(base, half),
            at(base, -half),
        ]
    }

    /// Distance from the outline, negative inside: the circle and the notch run
    /// smoothly into each other, so the notch grows out of the border.
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let circle = (x - self.centre.0).hypot(y - self.centre.1) - self.radius;
        let notch = polygon_distance(&self.notch_corners(), x, y);
        let spread = (self.border + self.notch).max(0.01) * NOTCH_SPREAD;
        let near = (spread - (circle - notch).abs()).max(0.0) / spread;
        circle.min(notch) - near * near * spread / 4.0
    }

    /// Cuts the picture to the outline and draws the border on it.
    pub fn draw(&self, canvas: &mut Canvas) {
        let inner = self.radius - self.border;
        for y in 0..canvas.height {
            for x in 0..canvas.width {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                let from_centre = (px - self.centre.0).hypot(py - self.centre.1);
                // Well inside the border: nothing to do, the usual case.
                if from_centre < inner - 1.0 {
                    continue;
                }
                let distance = self.distance(px, py);
                let at = ((y * canvas.width + x) * 4) as usize;
                if distance >= 0.5 {
                    canvas.pixels[at..at + 4].fill(0);
                    continue;
                }
                // The way out, which round the notch isn't straight from the middle.
                let (out_x, out_y) = (
                    self.distance(px + 0.5, py) - self.distance(px - 0.5, py),
                    self.distance(px, py + 0.5) - self.distance(px, py - 0.5),
                );
                let facing = (out_x * LIGHT.0 + out_y * LIGHT.1) / out_x.hypot(out_y).max(0.01);
                let colour = if distance < -self.border {
                    // Deep in the notch: as deep in the compass.
                    BORDER.map(|channel| channel as u8)
                } else {
                    ridge_colour(facing, distance > -self.border / 2.0)
                };
                canvas.blend(x, y, colour, (from_centre - inner + 0.5).clamp(0.0, 1.0));
                // The outer edge fades out over the picture's transparency.
                if from_centre >= inner {
                    canvas.pixels[at + 3] = ((0.5 - distance).clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
    }
}

/// Distance from a point to a line with rounded ends, `half` to each side of
/// it, negative inside.
pub fn line_distance(from: (f32, f32), to: (f32, f32), half: f32, x: f32, y: f32) -> f32 {
    let (along_x, along_y) = (to.0 - from.0, to.1 - from.1);
    let (at_x, at_y) = (x - from.0, y - from.1);
    let length = along_x * along_x + along_y * along_y;
    let along = ((at_x * along_x + at_y * along_y) / length.max(f32::MIN_POSITIVE)).clamp(0.0, 1.0);
    (at_x - along_x * along).hypot(at_y - along_y * along) - half
}

/// Fills a shape in the ridge's style: `band` wide from its edge, its plain
/// colour deeper in. Which way is out comes from the distance function.
fn ridged(
    canvas: &mut Canvas,
    centre: (f32, f32),
    reach: f32,
    band: f32,
    distance: impl Fn(f32, f32) -> f32,
) {
    let low = |at: f32| (at - reach - 1.0).floor().max(0.0) as u32;
    let high = |at: f32, end: u32| ((at + reach + 1.0).ceil().max(0.0) as u32).min(end);
    for y in low(centre.1)..high(centre.1, canvas.height) {
        for x in low(centre.0)..high(centre.0, canvas.width) {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let here = distance(px, py);
            if here >= 0.5 {
                continue;
            }
            let (out_x, out_y) = (
                distance(px + 0.5, py) - distance(px - 0.5, py),
                distance(px, py + 0.5) - distance(px, py - 0.5),
            );
            let facing = (out_x * LIGHT.0 + out_y * LIGHT.1) / out_x.hypot(out_y).max(0.01);
            let colour = if here < -band {
                BORDER.map(|channel| channel as u8)
            } else {
                ridge_colour(facing, here > -band / 2.0)
            };
            canvas.blend(x, y, colour, (0.5 - here).clamp(0.0, 1.0));
        }
    }
}

/// The full map's compass: a stylised N, under it an arrow pointing north,
/// and under that an inverted cross of `length`, ending at `foot`. `line` is
/// how wide the cross's lines are.
pub fn compass(canvas: &mut Canvas, foot: (f32, f32), length: f32, line: f32) {
    let x = foot.0;
    let half = line / 2.0;
    let top = foot.1 - length;
    // South, east and west are all the same length.
    let arm = length * 0.32;
    let arm_y = foot.1 - arm;
    let head_height = line * 2.6;
    let head_half = line * 1.7;
    let tip = top - head_height;
    let head = [(x, tip), (x + head_half, top), (x - head_half, top)];
    let middle = (x, (tip + foot.1) / 2.0);
    let reach = (foot.1 - tip) / 2.0 + arm.max(head_half);
    ridged(canvas, middle, reach, half, |px, py| {
        let stem = line_distance((x, top), (x, foot.1 - half), half, px, py);
        let cross = line_distance((x - arm, arm_y), (x + arm, arm_y), half, px, py);
        stem.min(cross).min(polygon_distance(&head, px, py))
    });

    // The N, with a little bar at the head of each upright.
    let stroke = line * 0.3;
    let (n_half, n_height) = (line * 1.2, line * 3.2);
    let n_bottom = tip - line * 0.8;
    let n_top = n_bottom - n_height;
    let (left, right) = (x - n_half, x + n_half);
    let bar = stroke * 2.5;
    let n_middle = (x, (n_top + n_bottom) / 2.0);
    ridged(canvas, n_middle, n_height, stroke, |px, py| {
        [
            ((left, n_top), (left, n_bottom)),
            ((right, n_top), (right, n_bottom)),
            ((left, n_top), (right, n_bottom)),
            ((left - bar, n_top), (left, n_top)),
            ((left - bar, n_bottom), (left + bar, n_bottom)),
            ((right - bar, n_top), (right + bar, n_top)),
        ]
        .iter()
        .map(|&(from, to)| line_distance(from, to, stroke, px, py))
        .fold(f32::MAX, f32::min)
    });
}

/// The corners of a stairs icon `size` across, stepped from its highest step
/// on the left going down, or on the right going up.
fn stairs_corners(centre: (f32, f32), size: f32, down: bool) -> Vec<(f32, f32)> {
    let step = size / STAIRS_STEPS as f32;
    let half = size / 2.0;
    // Worked out for going down, then mirrored for going up.
    let mut corners = vec![(-half, half), (-half, -half)];
    for done in 1..=STAIRS_STEPS {
        let x = -half + step * done as f32;
        let y = -half + step * (done - 1) as f32;
        corners.push((x, y));
        if done < STAIRS_STEPS {
            corners.push((x, y + step));
        }
    }
    corners.push((half, half));
    corners
        .into_iter()
        .map(|(x, y)| (centre.0 + if down { x } else { -x }, centre.1 + y))
        .collect()
}

/// A flat stairs icon, drawn upright whichever way the map is turned, since
/// which side its top is on says which way it goes.
pub fn stairs_icon(canvas: &mut Canvas, centre: (f32, f32), size: f32, down: bool) {
    let corners = stairs_corners(centre, size, down);
    canvas.shape(centre, size, STAIRS_INK, |x, y| {
        polygon_distance(&corners, x, y)
    });
}

/// The player: a white circle with a thin black outline, `size` across in all.
pub fn player_dot(canvas: &mut Canvas, centre: (f32, f32), size: f32, outline: f32) {
    canvas.disc(centre, size / 2.0, BLACK);
    canvas.disc(centre, size / 2.0 - outline, WHITE);
}

/// A little arrow, head and shaft, `size` tall beside a mark `reach` across
/// each way, up or down: which way the player's floor is. On its right, or its
/// left if that is off the picture.
pub fn floor_hint(canvas: &mut Canvas, mark: (f32, f32), reach: f32, size: f32, up: bool) {
    let half_head = size * HINT_HEAD_WIDTH / 2.0;
    let side = reach + size * HINT_GAP + half_head;
    let goes_off = mark.0 + side + half_head > canvas.width as f32;
    let x = mark.0 + if goes_off { -side } else { side };
    let half_shaft = size * HINT_SHAFT / 2.0;
    // Worked out pointing up, then turned over for down.
    let at = |across: f32, from_tip: f32| {
        let down = from_tip - size / 2.0;
        (x + across, mark.1 + if up { down } else { -down })
    };
    let neck = size * HINT_HEAD_HEIGHT;
    let corners = [
        at(0.0, 0.0),
        at(half_head, neck),
        at(half_shaft, neck),
        at(half_shaft, size),
        at(-half_shaft, size),
        at(-half_shaft, neck),
        at(-half_head, neck),
    ];
    let centre = (x, mark.1);
    let edge = size * HINT_EDGE;
    canvas.shape(centre, size + edge, BLACK, |x, y| {
        polygon_distance(&corners, x, y) - edge
    });
    canvas.shape(centre, size, WHITE, |x, y| polygon_distance(&corners, x, y));
}

/// Where to put the arrow to a player off the map: on the edge of the box
/// `inset` inside the picture, towards `target` from the picture's middle.
/// `None` if the target is inside that box.
pub fn arrow_place(
    (width, height): (f32, f32),
    target: (f32, f32),
    inset: f32,
) -> Option<((f32, f32), (f32, f32))> {
    let (middle_x, middle_y) = (width / 2.0, height / 2.0);
    let (dx, dy) = (target.0 - middle_x, target.1 - middle_y);
    let (reach_x, reach_y) = (middle_x - inset, middle_y - inset);
    if dx.abs() <= reach_x && dy.abs() <= reach_y {
        return None;
    }
    let scale =
        (reach_x / dx.abs().max(f32::MIN_POSITIVE)).min(reach_y / dy.abs().max(f32::MIN_POSITIVE));
    let length = dx.hypot(dy);
    Some((
        (middle_x + dx * scale, middle_y + dy * scale),
        (dx / length, dy / length),
    ))
}

/// The corners of an arrowhead pointing along `direction` (a unit vector).
fn arrow_corners(centre: (f32, f32), direction: (f32, f32), size: f32) -> [(f32, f32); 4] {
    let (along_x, along_y) = direction;
    let (across_x, across_y) = (-along_y, along_x);
    let at = |along: f32, across: f32| {
        (
            centre.0 + along_x * along + across_x * across,
            centre.1 + along_y * along + across_y * across,
        )
    };
    // A notched back makes it an arrowhead rather than a triangle.
    [
        at(size * 0.6, 0.0),
        at(-size * 0.5, size * 0.5),
        at(-size * 0.2, 0.0),
        at(-size * 0.5, -size * 0.5),
    ]
}

/// A white arrowhead with a black outline, in the dot's style, pointing along
/// `direction` (a unit vector).
pub fn arrow(
    canvas: &mut Canvas,
    centre: (f32, f32),
    direction: (f32, f32),
    size: f32,
    outline: f32,
) {
    let corners = arrow_corners(centre, direction, size);
    canvas.shape(centre, size + outline, BLACK, |x, y| {
        polygon_distance(&corners, x, y) - outline
    });
    canvas.shape(centre, size, WHITE, |x, y| polygon_distance(&corners, x, y));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notch_runs_from_inside_the_border_to_just_past_it_towards_north() {
        let frame = RoundFrame {
            centre: (50.0, 50.0),
            radius: 30.0,
            border: 5.0,
            notch: 10.0,
            north: (1.0, 0.0),
        };
        let corners = frame.notch_corners();
        assert_eq!(corners[0], (90.0, 50.0));
        // Its base is on the inside line of the border.
        assert_eq!(corners[1].0, 75.0);
        assert_eq!(corners[2].0, 75.0);
        assert!(frame.distance(50.0, 50.0) < 0.0);
        // Out past the circle on that side, not on the other, and its foot spreads
        // along the border.
        assert!(frame.distance(85.0, 50.0) < 0.0);
        assert!(frame.distance(15.0, 50.0) > 0.0);
        assert!(frame.distance(80.0, 54.0) < 0.0);
    }

    #[test]
    fn the_frame_cuts_the_picture_round_and_borders_it() {
        let mut pixels = vec![9; 4 * 40 * 40];
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width: 40,
            height: 40,
        };
        RoundFrame {
            centre: (20.0, 20.0),
            radius: 10.0,
            border: 3.0,
            notch: 5.0,
            north: (0.0, -1.0),
        }
        .draw(&mut canvas);
        let alpha = |x: usize, y: usize| pixels[(y * 40 + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0);
        // Over the map, and on the border (solid).
        assert_eq!(pixels[(20 * 40 + 20) * 4], 9);
        assert_eq!(alpha(20, 11), 255);
        assert_ne!(pixels[(11 * 40 + 20) * 4], 9);
    }

    #[test]
    fn the_compass_stands_on_its_foot_with_its_arrow_above() {
        let mut pixels = vec![0; 4 * 100 * 100];
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width: 100,
            height: 100,
        };
        compass(&mut canvas, (50.0, 90.0), 30.0, 5.0);
        let solid = |x: usize, y: usize| pixels[(y * 100 + x) * 4 + 3] == 255;
        // The stem, the crossbar, and the arrow above them.
        assert!(solid(50, 80));
        assert!(solid(42, 80));
        assert!(!solid(40, 85));
        assert!(solid(50, 55));
    }

    #[test]
    fn stairs_are_highest_on_the_left_going_down_and_the_right_going_up() {
        let height_of = |down: bool, x: f32| {
            let corners = stairs_corners((0.0, 0.0), 8.0, down);
            (-40..40)
                .filter(|y| polygon_distance(&corners, x, *y as f32 / 10.0 + 0.05) < 0.0)
                .count()
        };
        // Four steps, 2 wide and 2 high, across 8.
        let down: Vec<_> = [-3.0, -1.0, 1.0, 3.0]
            .iter()
            .map(|&x| height_of(true, x))
            .collect();
        assert_eq!(down, [80, 60, 40, 20]);
        let up: Vec<_> = [-3.0, -1.0, 1.0, 3.0]
            .iter()
            .map(|&x| height_of(false, x))
            .collect();
        assert_eq!(up, [20, 40, 60, 80]);
    }

    #[test]
    fn the_arrow_beside_the_mark_is_white_edged_in_black_and_points_its_way() {
        let mut pixels = vec![100; 4 * 40 * 40];
        let mut canvas = Canvas {
            pixels: &mut pixels,
            width: 40,
            height: 40,
        };
        floor_hint(&mut canvas, (10.0, 20.0), 5.0, 16.0, true);
        let colour = |x: usize, y: usize| pixels[(y * 40 + x) * 4];
        let white = |y: usize| (0..40).filter(|&x| colour(x, y) == 255).count();
        // Up: the head, at the top, is wider than the shaft.
        assert!(white(19) > white(25), "{} {}", white(19), white(25));
        assert!(white(25) > 0);
        assert!((0..40).any(|x| colour(x, 25) == 0));
    }

    #[test]
    fn the_arrow_sits_on_the_edge_towards_the_player() {
        let size = (100.0, 100.0);
        assert_eq!(arrow_place(size, (60.0, 40.0), 10.0), None);
        let (place, direction) = arrow_place(size, (500.0, 50.0), 10.0).unwrap();
        assert_eq!(place, (90.0, 50.0));
        assert_eq!(direction, (1.0, 0.0));
        let (place, _) = arrow_place(size, (-150.0, -150.0), 10.0).unwrap();
        assert_eq!(place, (10.0, 10.0));
    }
}
