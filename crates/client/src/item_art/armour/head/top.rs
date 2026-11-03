//! What goes on top (knobs, spikes, a cabasset's stalk, combs, horsehair crests,
//! plumes) and ornaments: horns, antlers, wings, a Japanese crest at the brow, a crown,
//! a dragon.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use shared::armour::head::{HeadForm, Ornament, Top};

use super::shell::Shell;
use crate::item_art::armour::surface::{grid, ring_grid};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::{blob, limb};

/// A point along the line over the top from the front of the edge to the back, `s`
/// from 0 to 1, and the way out from it.
fn over(shell: &Shell, s: f32) -> (Vec3, Vec3) {
    let apex = shell.apex();
    let (angle, from, t) = if s < 0.5 {
        (0.0, shell.edge_at(0.0) + 25.0, s * 2.0)
    } else {
        (PI, shell.edge_at(PI) + 25.0, (1.0 - s) * 2.0)
    };
    shell.point(angle, from + (apex - from) * (t * PI / 2.0).sin())
}

/// A fin along the line over the top, between `from` and `to`, its height at each
/// point given by `tall`, standing `lift` off the bowl.
#[allow(clippy::too_many_arguments)]
fn fin(
    model: &mut Model,
    stuff: Stuff,
    shell: &Shell,
    (from, to): (f32, f32),
    lift: f32,
    tall: &dyn Fn(f32) -> f32,
    thick: f32,
) {
    let surface = |u: f32, v: f32| {
        let s = from + (to - from) * u;
        let (point, out) = over(shell, s);
        (point + out * (lift + tall(s) * v), Vec3::X)
    };
    plate(model.of(stuff), &grid(&surface, 1, 24), thick, false);
}

pub(super) fn add_top(model: &mut Model, form: &HeadForm, shell: &Shell) {
    let (apex, _) = over(shell, 0.5);
    match form.top {
        Top::None => {}
        Top::Knob => blob(
            model.of(Stuff::Main),
            apex + Vec3::Y * 8.0,
            Vec3::splat(13.0),
        ),
        Top::Spike => limb(
            model.of(Stuff::Main),
            &[
                (apex, 10.0),
                (apex + Vec3::Y * 60.0, 5.0),
                (apex + Vec3::Y * 125.0, 1.0),
            ],
        ),
        Top::Stalk => limb(
            model.of(Stuff::Main),
            &[
                (apex, 6.0),
                (apex + Vec3::new(0.0, 15.0, -5.0), 4.0),
                (apex + Vec3::new(0.0, 20.0, -18.0), 1.5),
            ],
        ),
        Top::Comb => fin(
            model,
            Stuff::Main,
            shell,
            (0.1, 0.9),
            0.0,
            &|s| 15.0 * (s * PI).sin().sqrt(),
            3.0,
        ),
        Top::HighComb => fin(
            model,
            Stuff::Main,
            shell,
            (0.12, 0.88),
            0.0,
            &|s| 75.0 * (s * PI).sin().powf(0.8),
            3.0,
        ),
        Top::Crest => {
            fin(model, Stuff::Main, shell, (0.3, 0.7), 0.0, &|_| 22.0, 8.0);
            // Horsehair, low at the front and hanging behind.
            fin(
                model,
                Stuff::Field,
                shell,
                (0.15, 0.95),
                20.0,
                &|s| 30.0 + 55.0 * (s * PI * 0.8).sin(),
                18.0,
            );
        }
        Top::Plume => {
            limb(
                model.of(Stuff::Trim),
                &[(apex, 5.0), (apex + Vec3::new(0.0, 35.0, -10.0), 4.0)],
            );
            let base = apex + Vec3::new(0.0, 35.0, -10.0);
            limb(
                model.of(Stuff::Field),
                &[
                    (base, 4.0),
                    (base + Vec3::new(0.0, 80.0, -30.0), 16.0),
                    (base + Vec3::new(0.0, 160.0, -90.0), 12.0),
                    (base + Vec3::new(0.0, 190.0, -150.0), 2.0),
                ],
            );
        }
    }
}

pub(super) fn add_ornament(model: &mut Model, form: &HeadForm, shell: &Shell) {
    let temple = |side: f32| shell.point(side * 1.5, shell.centre() + 35.0);
    match form.ornament {
        Ornament::None => {}
        Ornament::Horns => {
            for side in [-1.0f32, 1.0] {
                let (root, out) = temple(side);
                let across = Vec3::X * side;
                limb(
                    model.of(Stuff::Bone),
                    &[
                        (root - out * 5.0, 24.0),
                        (root + across * 60.0 + Vec3::Y * 15.0, 18.0),
                        (root + across * 120.0 + Vec3::Y * 70.0, 12.0),
                        (root + across * 135.0 + Vec3::new(0.0, 150.0, 15.0), 6.0),
                        (root + across * 115.0 + Vec3::new(0.0, 200.0, 30.0), 1.5),
                    ],
                );
            }
        }
        Ornament::Antlers => {
            for side in [-1.0f32, 1.0] {
                let (root, _) = temple(side);
                let at = |x: f32, y: f32, z: f32| root + Vec3::new(x * side, y, z);
                limb(
                    model.of(Stuff::Bone),
                    &[
                        (at(0.0, 0.0, 0.0), 12.0),
                        (at(50.0, 90.0, -10.0), 9.0),
                        (at(90.0, 190.0, -30.0), 5.0),
                        (at(110.0, 250.0, -40.0), 1.5),
                    ],
                );
                for (from, tip) in [
                    ((50.0, 90.0, -10.0), (40.0, 150.0, 40.0)),
                    ((90.0, 190.0, -30.0), (150.0, 230.0, 0.0)),
                ] {
                    limb(
                        model.of(Stuff::Bone),
                        &[
                            (at(from.0, from.1, from.2), 6.0),
                            (at(tip.0, tip.1, tip.2), 1.5),
                        ],
                    );
                }
            }
        }
        Ornament::Wings | Ornament::Dragon => {
            let span = if form.ornament == Ornament::Wings {
                180.0
            } else {
                120.0
            };
            for side in [-1.0f32, 1.0] {
                let (root, _) = temple(side);
                let surface = |u: f32, v: f32| {
                    let feather = (v - 0.5) * 1.2;
                    let reach = span * u;
                    let point = root
                        + Vec3::new(
                            side * (10.0 + reach * 0.35),
                            reach * (0.6 + feather),
                            -reach * (0.75 - feather * 0.5),
                        );
                    (point, Vec3::X * side)
                };
                plate(model.of(Stuff::Main), &grid(&surface, 4, 6), 3.0, false);
            }
            if form.ornament == Ornament::Dragon {
                fin(
                    model,
                    Stuff::Main,
                    shell,
                    (0.05, 0.95),
                    0.0,
                    &|s| 40.0 + 25.0 * (s * 9.0 * PI).sin().abs(),
                    4.0,
                );
                let (front, out) = over(shell, 0.12);
                blob(
                    model.of(Stuff::Main),
                    front + out * 50.0 + Vec3::Z * 30.0,
                    Vec3::new(22.0, 20.0, 45.0),
                );
            }
        }
        Ornament::FrontCrest => {
            // Kuwagata: two flat prongs rising from the brow.
            let (brow, _) = shell.point(0.0, shell.level(0.68));
            for side in [-1.0f32, 1.0] {
                let surface = |u: f32, v: f32| {
                    let point = brow
                        + Vec3::new(
                            side * (12.0 + 70.0 * u.powf(1.5) + 8.0 * v),
                            160.0 * u,
                            12.0 + 10.0 * u,
                        );
                    (point, Vec3::Z)
                };
                plate(model.of(Stuff::Charge), &grid(&surface, 1, 8), 3.0, false);
            }
        }
        Ornament::Crown => {
            let base = shell.level(0.72);
            let surface = |u: f32, v: f32| {
                let angle = u * TAU;
                let tooth = ((angle * 8.0).cos().max(0.0)).powi(3);
                let (point, out) = shell.point(angle, base);
                (point + out * 5.0 + Vec3::Y * (18.0 + 22.0 * tooth) * v, out)
            };
            plate(
                model.of(Stuff::Charge),
                &ring_grid(&surface, 2, 64),
                3.0,
                true,
            );
        }
    }
}
