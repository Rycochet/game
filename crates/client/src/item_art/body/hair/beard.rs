//! Beards: a surface over the jaw from the sideburns and under the nose round under
//! the chin, fuller with a moustache, or hanging down the chest, plaited or forked.

use bevy::prelude::*;
use shared::body::Beard;

use super::super::{blob, limb};
use super::scalp::sheet;
use super::{Skull, Triangle};

/// Columns from one ear round the jaw to the other, and rows down each.
const COLUMNS: usize = 15;
const ROWS: usize = 6;
/// How far either side of the front the jaw's hair reaches: past the ears a little.
const REACH: f32 = 1.75;

/// This beard on the head.
pub(super) fn beard(triangles: &mut Vec<Triangle>, skull: &Skull, beard: Beard) {
    let grow = match beard {
        Beard::None => return,
        Beard::Short => 7.0,
        Beard::Full => 12.0,
        Beard::Long | Beard::Braided | Beard::Forked => 9.0,
    };
    jaw(triangles, skull, grow);
    if beard != Beard::Short {
        moustache(triangles, skull);
    }
    let chin = skull.round(0.0, 2.6, grow);
    match beard {
        Beard::Full => blob(
            triangles,
            chin + Vec3::new(0.0, -12.0, -skull.deep * 0.1),
            Vec3::new(skull.across * 0.55, 30.0, skull.deep * 0.35),
        ),
        Beard::Long => hang(triangles, skull, chin, 0.0, &[0.55, 0.5, 0.35, 0.08]),
        Beard::Braided => hang(
            triangles,
            skull,
            chin,
            0.0,
            &[0.45, 0.3, 0.38, 0.26, 0.32, 0.22, 0.26, 0.08],
        ),
        Beard::Forked => {
            for side in [1.0, -1.0] {
                hang(triangles, skull, chin, side, &[0.45, 0.4, 0.22, 0.06]);
            }
        }
        Beard::None | Beard::Short => {}
    }
}

/// Over the jaw, from the sideburns and under the nose down under the chin.
fn jaw(triangles: &mut Vec<Triangle>, skull: &Skull, grow: f32) {
    let columns: Vec<Option<Vec<Vec3>>> = (0..COLUMNS)
        .map(|column| {
            let turn = -REACH + 2.0 * REACH * column as f32 / (COLUMNS - 1) as f32;
            // From under the nose in front, up to the ears at the sides.
            let (front, side) = (turn.cos().max(0.0).powi(2), turn.sin().powi(2));
            let share = -0.3 * front + 0.1 * side;
            let top = share.clamp(-1.0, 1.0).acos();
            let bottom = std::f32::consts::PI - 0.1;
            Some(
                (0..=ROWS)
                    .map(|row| {
                        let down = top + (bottom - top) * row as f32 / ROWS as f32;
                        skull.round(turn, down, grow)
                    })
                    .collect(),
            )
        })
        .collect();
    sheet(triangles, &columns, false);
}

/// Across the upper lip, from cheek to cheek.
fn moustache(triangles: &mut Vec<Triangle>, skull: &Skull) {
    let path: Vec<(Vec3, f32)> = [(-0.55, 2.0, 5.0), (0.0, 1.92, 8.0), (0.55, 2.0, 5.0)]
        .into_iter()
        .map(|(turn, down, thick)| (skull.round(turn, down, thick), thick))
        .collect();
    limb(triangles, &path);
}

/// Hanging from the chin down the chest, by these widths (shares of the head's), kept
/// in front of the chest; `side` parts a forked beard's points to either side.
fn hang(triangles: &mut Vec<Triangle>, skull: &Skull, chin: Vec3, side: f32, widths: &[f32]) {
    let chest = skull.neck_base.z + skull.chest_front;
    let steps = (widths.len() - 1) as f32;
    let path: Vec<(Vec3, f32)> = widths
        .iter()
        .enumerate()
        .map(|(at, width)| {
            let along = at as f32 / steps;
            let thick = skull.across * width;
            let mut point = chin
                + Vec3::new(
                    side * skull.across * 0.35 * along,
                    -240.0 * along,
                    30.0 * along.sqrt(),
                );
            if point.y < skull.neck_base.y + thick {
                point.z = point.z.max(chest + thick + 15.0);
            }
            (point, thick)
        })
        .collect();
    limb(triangles, &path);
}
