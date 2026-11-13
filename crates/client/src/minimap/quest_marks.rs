//! Quest markers on the world map's maps: a "!" over each poi with a quest
//! step to do now, or a "?" over a town with a quest to hand in, in its
//! colour and ringed in black like the view's (`quests::QuestPois`). Drawn
//! as strokes, as the maps have no font.

use crate::common::CurrentWorld;
use crate::quests::{MarkLook, QuestPois};

use super::canvas::Canvas;
use super::projection::Projection;

/// How tall a sign is, against a poi's picture.
const OF_ICON: f32 = 0.9;
/// How thick its strokes are, against its height.
const STROKE: f32 = 0.13;
/// How far its black ring reaches, against its height.
const RING: f32 = 0.08;

/// Draws a marker over every marked poi in `area` (world points), whose
/// pictures are `icon` pixels across.
pub(super) fn draw(
    canvas: &mut Canvas,
    projection: &Projection,
    world: &CurrentWorld,
    marks: &QuestPois,
    area: ((i64, i64), (i64, i64)),
    icon: f32,
) {
    if marks.0.is_empty() {
        return;
    }
    for poi in world.pois.in_area(area.0, area.1) {
        let Some(look) = marks.0.get(&poi.chunk) else {
            continue;
        };
        let at = projection.to_picture(poi.x as f64, poi.y as f64);
        let height = icon * OF_ICON;
        // Its foot on the top of the poi's picture.
        let foot = (at.0, at.1 - icon / 2.0);
        sign(canvas, *look, foot, height);
    }
}

/// One "!" or "?" standing on `foot`, `height` pixels tall.
pub(super) fn sign(canvas: &mut Canvas, look: MarkLook, foot: (f32, f32), height: f32) {
    let strokes = strokes(look.ready);
    let place = |(x, y): (f32, f32)| (foot.0 + x * height, foot.1 - (1.0 - y) * height);
    let strokes: Vec<((f32, f32), (f32, f32))> = (strokes.iter())
        .map(|&(a, b)| (place(a), place(b)))
        .collect();
    let (width, ring) = (STROKE * height, RING * height);
    let centre = (foot.0, foot.1 - height / 2.0);
    let reach = height;
    let distance = |x: f32, y: f32| {
        (strokes.iter())
            .map(|&(a, b)| to_segment((x, y), a, b))
            .fold(f32::MAX, f32::min)
            - width
    };
    canvas.shape(centre, reach, [0, 0, 0], |x, y| distance(x, y) - ring);
    canvas.shape(centre, reach, look.rgb, distance);
}

/// A sign's strokes, in its own measure: `x` from its middle, `y` from its
/// top (0) to its foot (1). A dot is a stroke of no length.
fn strokes(ready: bool) -> Vec<((f32, f32), (f32, f32))> {
    let dot = ((0.0, 0.86), (0.0, 0.86));
    if !ready {
        return vec![((0.0, 0.12), (0.0, 0.6)), dot];
    }
    // The hook of the "?": round from its left, over the top, down the right
    // and back to the middle, then the stem.
    let (middle, radius) = ((0.0, 0.3), 0.2);
    let along: Vec<(f32, f32)> = (0..=8)
        .map(|i| {
            let angle = std::f32::consts::PI * (1.0 - 1.4 * i as f32 / 8.0);
            (
                middle.0 + radius * angle.cos(),
                middle.1 - radius * angle.sin(),
            )
        })
        .collect();
    let mut strokes: Vec<_> = along.windows(2).map(|pair| (pair[0], pair[1])).collect();
    let end = along[along.len() - 1];
    strokes.push((end, (0.0, 0.6)));
    strokes.push(dot);
    strokes
}

/// How far a point is from a segment.
fn to_segment(point: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (along_x, along_y) = (b.0 - a.0, b.1 - a.1);
    let (to_x, to_y) = (point.0 - a.0, point.1 - a.1);
    let length = along_x * along_x + along_y * along_y;
    let share = if length > 0.0 {
        ((to_x * along_x + to_y * along_y) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (to_x - along_x * share).hypot(to_y - along_y * share)
}

#[cfg(test)]
mod tests;
