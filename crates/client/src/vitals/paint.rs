//! Painting the circle: Health on the left in red, Mana on the right in blue,
//! each half filling from the bottom over black, with a border all round each,
//! bevelled so each half stands up like a button (owner, 2026-11-06).

/// The fills and what is round them.
const HEALTH: [f32; 3] = [200.0, 30.0, 35.0];
const MANA: [f32; 3] = [40.0, 85.0, 225.0];
const EMPTY: [f32; 3] = [0.0, 0.0, 0.0];
/// The minimap's border colour, so they look alike.
const BORDER: [f32; 3] = [150.0, 115.0, 70.0];

/// How thick the border is, as a share of the circle across.
const BORDER_OF_ACROSS: f32 = 0.05;
/// How much lighter or darker the bevel goes, facing towards or away from the
/// light, which comes from the top left as for the minimap's border.
const BEVEL: f32 = 0.55;
const LIGHT: (f32, f32) = (
    -std::f32::consts::FRAC_1_SQRT_2,
    -std::f32::consts::FRAC_1_SQRT_2,
);

/// The circle `side` pixels square, as sRGB bytes with alpha, filled to these
/// shares (0 to 1) of each half.
pub(super) fn paint(side: u32, health: f32, mana: f32) -> Vec<u8> {
    let size = side as f32;
    let middle = size / 2.0;
    let radius = middle - 0.5;
    let border = (size * BORDER_OF_ACROSS).max(1.5);
    // The inside's height, from its lowest point to its highest.
    let inside = 2.0 * (radius - border);
    let level = |share: f32| middle + radius - border - share.clamp(0.0, 1.0) * inside;
    let (health_level, mana_level) = (level(health), level(mana));
    let mut bytes = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let distance = (px - middle).hypot(py - middle);
            // How much of the pixel is inside the circle, and inside the border.
            let cover = (radius - distance + 0.5).clamp(0.0, 1.0);
            let ring = (distance - (radius - border) + 0.5).clamp(0.0, 1.0);
            let line = (border / 2.0 - (px - middle).abs() + 0.5).clamp(0.0, 1.0);
            let left = px < middle;
            let (fill, level) = if left {
                (HEALTH, health_level)
            } else {
                (MANA, mana_level)
            };
            let filled = (py - level + 0.5).clamp(0.0, 1.0);
            let inner = mix(EMPTY, fill, filled);
            // Which way the border faces, out of its half: round the edge, or
            // across the middle line.
            let (out_x, out_y) = if line > ring {
                (if left { 1.0 } else { -1.0 }, 0.0)
            } else {
                let to = distance.max(0.001);
                ((px - middle) / to, (py - middle) / to)
            };
            let facing = out_x * LIGHT.0 + out_y * LIGHT.1;
            let edge = BORDER.map(|channel| (channel * (1.0 + BEVEL * facing)).clamp(0.0, 255.0));
            let colour = mix(inner, edge, ring.max(line));
            bytes.extend(colour.map(|channel| channel.round() as u8));
            bytes.push((cover * 255.0).round() as u8);
        }
    }
    bytes
}

/// `a` to `b` by `share`.
fn mix(a: [f32; 3], b: [f32; 3], share: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * share)
}

#[cfg(test)]
mod tests;
