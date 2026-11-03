//! The cuff over the wrist.

use bevy::prelude::*;
use shared::armour::Style;
use shared::armour::hands::{Cuff, Hand, HandsForm};
use shared::body::Frame;
use std::f32::consts::PI;

use super::super::{Model, Stuff, plate};
use super::{Axes, CUFF_CLEAR, ROUND, flute, look_of, styled};
use crate::item_art::body::limb;

/// The cuff: round the wrist and up the forearm, flaring by its kind, over a vambrace.
pub(super) fn add_cuff(
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
