//! The back of the hand and the lames over it.

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::hands::{Fingers, HandsForm};

use super::super::{Model, Stuff};
use super::{Arc, Axes, LAME_STEP, Shape, add_arc, add_arc_bumped, flute, look_of, styled};

/// The plate over the back of the hand, and the one over the knuckles.
pub(super) fn add_back(
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
pub(super) fn add_lames(
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
pub(super) fn lames_along(
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
