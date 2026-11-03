//! Flat stairs icons.

use super::super::canvas::{Canvas, polygon_distance};
use super::{STAIRS_INK, STAIRS_STEPS};

/// The corners of a stairs icon `size` across, stepped from its highest step
/// on the left going down, or on the right going up.
pub(super) fn stairs_corners(centre: (f32, f32), size: f32, down: bool) -> Vec<(f32, f32)> {
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
