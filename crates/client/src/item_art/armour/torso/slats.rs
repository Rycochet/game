//! Upright slats or rods laced side by side round the body (Tlingit slat armour, or
//! metal strips over a backing), and rattan woven over a frame.

use shared::armour::torso::{TorsoBody, TorsoForm};
use shared::body::Frame;
use shared::material::Category;

use super::plates::{stripes, strips};
use super::soft::{round_shell, wrap};
use crate::item_art::armour::surface::{grid, lines};
use crate::item_art::armour::{Model, Stuff, plate};
use crate::item_art::body::limb;

/// How far slats and rattan stand off the skin.
const SLAT_OUT: f32 = 18.0;
/// How many slats or rods go round.
const SLATS: usize = 28;
const RODS: usize = 44;

pub(super) fn add_slats(model: &mut Model, form: &TorsoForm, frame: &Frame) {
    let TorsoBody::Splint { rods } = form.body else {
        return;
    };
    let shell = round_shell(frame, SLAT_OUT, 0.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    let metal = form.material.category() == Category::Metal;
    if metal {
        // Strips riveted over a leather backing.
        wrap(model, Stuff::Backing, &shell, 3.0, &|_, _| 0.0);
    }
    if rods {
        for rod in 0..RODS {
            let u = (rod as f32 + 0.5) / RODS as f32;
            let ((bottom, out), (top, _)) = (shell.at(u, 0.0), shell.at(u, 1.0));
            let path = [(bottom + out * 4.0, 5.0), (top + out * 4.0, 5.0)];
            limb(model.of(Stuff::Main), &path);
        }
    } else {
        // Metal strips close over their backing; slats with gaps between.
        let width = if metal { 0.95 } else { 0.8 } / SLATS as f32;
        for slat in 0..SLATS {
            let u = (slat as f32 + 0.5) / SLATS as f32;
            let lift = if slat % 2 == 0 { 3.0 } else { 4.5 };
            // Rows all the way up, so each slat follows the body's curve.
            let slat_surface = |across: f32, up: f32| {
                let (point, out) = shell.at(u + (across - 0.5) * width, up);
                (point + out * lift, out)
            };
            plate(
                model.of(Stuff::Main),
                &grid(&slat_surface, 12, 1),
                5.0,
                false,
            );
        }
    }
    if !metal {
        // Bands of lacing across, holding the slats together.
        stripes(model, Stuff::Cloth, &surface, 4, 8.0);
    }
}

/// Rattan woven over a frame: a close basket weave over the whole shell.
pub(super) fn add_rattan(model: &mut Model, frame: &Frame) {
    let shell = round_shell(frame, SLAT_OUT, 0.0);
    let surface = |u: f32, v: f32| shell.at(u, v);
    wrap(model, Stuff::Main, &shell, 6.0, &|u: f32, v: f32| {
        // Strands over and under each other.
        let (across, down) = (u * 160.0, v * 24.0);
        1.5 * ((across.sin() * down.cos()).abs())
    });
    lines(model, &surface, 24, 4.0);
    strips(model, Stuff::Shade, &surface, 80, 4.0);
}
