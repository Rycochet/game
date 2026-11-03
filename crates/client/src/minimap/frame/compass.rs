//! The full map's compass.

use super::super::canvas::{Canvas, polygon_distance};
use super::{line_distance, ridged};

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
