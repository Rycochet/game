//! Rolling everything about a blade but its size, from its row: the blade
//! itself, then the hilt, then materials, in that order, as each takes rolls in
//! turn from the same generator.

mod blade;
mod hilt;
mod materials;

use super::table::Row;
use super::{BladeForm, Build, GRIP, SHAFT_FROM, within};
use crate::item::Quality;
use crate::rng::Rng;
use blade::roll_blade_itself;
use hilt::roll_hilt;
use materials::roll_materials;

/// How a blade is mounted and held, which every roll goes by.
#[derive(Clone, Copy)]
struct Holding {
    /// A head at the side of a pole, as an axe's.
    is_side: bool,
    /// Held in the hand by its handle, rather than a shaft.
    held: bool,
    /// On a grip, short enough for one hand.
    on_a_grip: bool,
}

/// First guesses at how often each is seen. Properties are rolled in this order.
pub(super) fn roll_properties(
    rng: &mut Rng,
    row: &Row,
    quality: Quality,
    blade: u16,
    handle: u16,
) -> BladeForm {
    let build = row.build;
    let is_side = build == Build::Side;
    let held = !is_side && handle < SHAFT_FROM;
    let on_a_grip = !is_side && within(GRIP, handle);
    let holding = Holding {
        is_side,
        held,
        on_a_grip,
    };

    let parts = roll_blade_itself(rng, row, holding, blade);
    let hilt = roll_hilt(rng, row, holding, handle);
    let made = roll_materials(rng, quality, holding);
    BladeForm {
        family: row.family,
        build,
        blade,
        handle,
        width: parts.width,
        edges: parts.edges,
        curve: parts.curve,
        point: parts.point,
        shape: parts.shape,
        section: parts.section,
        fuller: parts.fuller,
        hands: hilt.hands,
        guard: hilt.guard,
        pommel: hilt.pommel,
        ricasso: hilt.ricasso,
        langets: hilt.langets,
        handle_shape: hilt.handle_shape,
        sides: hilt.sides,
        top: hilt.top,
        back: hilt.back,
        tips: hilt.tips,
        blade_metal: made.blade_metal,
        handle_material: made.handle_material,
        fittings: made.fittings,
        decoration: made.decoration,
        generic_only: false,
    }
}
