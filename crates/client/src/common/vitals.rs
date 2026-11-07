//! The player's Health and Mana as the server last told them, and the room the
//! interface keeps at the bottom of the window.

use bevy::prelude::*;
use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::vitals::{REFILL_TICKS, VitalNow};

/// The player's own Health and Mana (`ServerMessage::Vitals` with no `who`),
/// and when they were told, by `Time` (which stops while single player is
/// paused, as the server's clock does). `vitals` writes it; `None` until told.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct OwnVitals(pub Option<ToldVitals>);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToldVitals {
    pub health: VitalNow,
    pub mana: VitalNow,
    /// `Time::elapsed_secs_f64` when they came.
    pub at: f64,
}

impl ToldVitals {
    /// Health and Mana `now` (by `Time`), refilled as the server would, with
    /// the part of a point refilled so far, for drawing smoothly.
    pub fn now(&self, now: f64) -> (f32, f32) {
        let ticks = ((now - self.at).max(0.0) * ACTION_TICKS_PER_SECOND) as f32;
        (refilled(self.health, ticks), refilled(self.mana, ticks))
    }
}

/// `VitalNow::after` with fractions.
fn refilled(told: VitalNow, ticks: f32) -> f32 {
    let most = told.most as f32;
    let refilling = (ticks - told.refills_in as f32).max(0.0);
    (told.value.as_f32().min(most) + refilling * most / REFILL_TICKS as f32).min(most)
}

/// How much of the window's bottom, in interface pixels, the interface keeps
/// across its middle (the Health and Mana circle): boxes naming what is
/// hovered stand above it.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct BottomHud(pub f32);

/// Where the Health and Mana circle is drawn, in interface pixels, at the bottom
/// middle of the window: how far across (0 while hidden) and how far its bottom
/// stands above the window's. The action bar stands either side of it.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct VitalsCircle {
    pub across: f32,
    /// Its bottom edge, above the window's.
    pub bottom: f32,
}
