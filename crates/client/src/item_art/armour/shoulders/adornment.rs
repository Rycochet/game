//! Adornments on shoulder pieces.

use bevy::prelude::*;
use shared::armour::Construction;
use shared::armour::shoulders::{Adornment, ShouldersForm, ShouldersPiece};
use std::f32::consts::PI;

use super::super::{Model, Stuff};
use super::{Arm, RIGID_CLEAR, disc_place};
use crate::item_art::body::{blob, limb};

/// Spikes, horns, a beast's head or a skull, on the outside of the shoulder.
pub(super) fn add_adornment(model: &mut Model, form: &ShouldersForm, arm: &Arm, height: f32) {
    let across = arm.radius + RIGID_CLEAR + height;
    // On a disc's middle, or over the cop.
    let disc = match form.piece {
        ShouldersPiece::Discs { .. } => Some(disc_place(arm)),
        _ => None,
    };
    let on = |over: f32, angle: f32| {
        disc.unwrap_or_else(|| arm.dome(angle, over, across, across * 0.72))
    };
    let spikes = if form.construction == Construction::Leather {
        Stuff::Rivet
    } else {
        Stuff::Main
    };
    match form.adornment {
        Adornment::None => {}
        Adornment::Spikes(count) => {
            for spike in 0..count {
                let over = if count == 1 {
                    0.6
                } else {
                    0.3 + 0.75 * spike as f32 / (count - 1) as f32
                };
                let (point, out) = on(over, 0.0);
                let long = if count == 1 { 60.0 } else { 42.0 };
                limb(
                    model.of(spikes),
                    &[(point, 10.0), (point + out * long, 0.8)],
                );
            }
        }
        Adornment::Horns => {
            let (point, out) = on(0.55, 0.0);
            let up = -arm.down;
            let path: Vec<_> = (0..6)
                .map(|step| {
                    let t = step as f32 / 5.0;
                    let at = point
                        + out * (70.0 * t)
                        + up * (45.0 * t * t)
                        + arm.forward * (25.0 * t * t);
                    (at, 13.0 * (1.0 - t) + 1.5)
                })
                .collect();
            limb(model.of(Stuff::Bone), &path);
        }
        Adornment::BeastHead => {
            let (point, out) = on(PI / 2.0 - 0.15, 0.0);
            let snout = point + out * 26.0 - arm.down * 6.0;
            blob(model.of(Stuff::Trim), snout, Vec3::new(26.0, 22.0, 30.0));
            blob(
                model.of(Stuff::Trim),
                snout + out * 26.0 - arm.down * 4.0,
                Vec3::new(14.0, 12.0, 18.0),
            );
            for side in [-1.0, 1.0] {
                let eye = snout + out * 18.0 - arm.down * 14.0 + arm.forward * (side * 13.0);
                blob(model.of(Stuff::Etch), eye, Vec3::splat(5.0));
                let ear = snout - arm.down * 22.0 + arm.forward * (side * 16.0);
                limb(
                    model.of(Stuff::Trim),
                    &[(ear, 8.0), (ear - arm.down * 22.0 - out * 6.0, 1.0)],
                );
            }
        }
        Adornment::Skull => {
            let (point, out) = on(1.0, 0.0);
            let skull = point + out * 22.0;
            blob(model.of(Stuff::Bone), skull, Vec3::new(26.0, 28.0, 24.0));
            blob(
                model.of(Stuff::Bone),
                skull + out * 6.0 + arm.down * 26.0,
                Vec3::new(17.0, 10.0, 15.0),
            );
            for side in [-1.0, 1.0] {
                let eye = skull + out * 20.0 + arm.down * 4.0 + arm.forward * (side * 10.0);
                blob(model.of(Stuff::Etch), eye, Vec3::new(6.0, 7.0, 6.0));
            }
        }
    }
}
