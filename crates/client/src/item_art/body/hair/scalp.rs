//! The hair over the scalp: a surface a little off the head, from the crown (or a
//! helmet's rim) down to the hairline, or to the shoulders for long hair.

use std::f32::consts::{FRAC_PI_4, TAU};

use bevy::prelude::*;
use shared::body::HairStyle;

use super::{Cover, Skull, Triangle};

/// Columns round the head, and rows down each.
const COLUMNS: usize = 24;
const ROWS: usize = 8;
/// How far hair stands off the scalp: less than a helmet's own gap, so it stays inside.
pub(super) const GROW: f32 = 6.0;
/// Long hair, a little fuller.
const LONG_GROW: f32 = 9.0;

/// How high the hairline is at this angle, as a share of half the head's height above
/// its middle: the forehead in front, above the ears at the sides, the nape behind.
pub(super) fn hairline(skull: &Skull, turn: f32) -> f32 {
    let (front, back) = (turn.cos().max(0.0), (-turn.cos()).max(0.0));
    let side = turn.sin().powi(2);
    let share = 0.45 * front * front - 0.15 * side - 0.6 * back * back;
    skull.middle.y + skull.up * share
}

/// The hair on the head, as this style wears it: short, long, or pulled back close to
/// the head for a tail, braids or a knot.
pub(super) fn cap(triangles: &mut Vec<Triangle>, skull: &Skull, cover: &Cover, style: HairStyle) {
    let long = style == HairStyle::Long;
    let grow = if long { LONG_GROW } else { GROW };
    // Shoulder length: a little above where the neck meets them.
    let shoulders = skull.neck_base.y + 30.0;
    let columns: Vec<Option<Vec<Vec3>>> = (0..COLUMNS)
        .map(|column| {
            let turn = column as f32 / COLUMNS as f32 * TAU;
            let mut bottom = skull.down_to(hairline(skull, turn), grow);
            if long {
                // Framing the face, then down to the shoulders at the sides and back.
                let off_face = ((turn.sin().abs().asin() - FRAC_PI_4) / 0.6).clamp(0.0, 1.0);
                let off_face = if turn.cos() < 0.0 { 1.0 } else { off_face };
                let hanging = skull.down_to(shoulders, grow);
                bottom += (hanging - bottom) * off_face;
            }
            let top = match cover.rim {
                Some(_) => skull.down_to(cover.rim_at(turn), grow),
                None => 0.0,
            };
            (top + 0.02 < bottom).then(|| {
                (0..=ROWS)
                    .map(|row| {
                        let down = top + (bottom - top) * row as f32 / ROWS as f32;
                        skull.at(turn, down, grow)
                    })
                    .collect()
            })
        })
        .collect();
    sheet(triangles, &columns, true);
}

/// Both faces of a surface through columns of points, each as long as the others,
/// skipping any pair with a column missing; `round` joins the last to the first.
pub(super) fn sheet(triangles: &mut Vec<Triangle>, columns: &[Option<Vec<Vec3>>], round: bool) {
    let pairs = if round {
        columns.len()
    } else {
        columns.len() - 1
    };
    for at in 0..pairs {
        let (Some(a), Some(b)) = (&columns[at], &columns[(at + 1) % columns.len()]) else {
            continue;
        };
        for row in 0..a.len().min(b.len()) - 1 {
            for [p, q, r] in [
                [a[row], b[row], b[row + 1]],
                [a[row], b[row + 1], a[row + 1]],
            ] {
                // Nothing where the rows meet at the crown.
                if (q - p).cross(r - p).length() < 1.0 {
                    continue;
                }
                triangles.push([p, q, r]);
                triangles.push([p, r, q]);
            }
        }
    }
}
