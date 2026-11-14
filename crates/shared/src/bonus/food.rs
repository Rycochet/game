//! What eating gives.

use super::effect::{Effect, EndsOn, Source};
use super::stat::{Bonus, Stat};
use crate::energy::ACTION_TICKS_PER_SECOND;
use crate::item::ItemKind;
use crate::vitals::{PER_POINT, Vital};

/// Health an apple gives back a second, in thousandths of a point.
pub const APPLE_RESTORE: u32 = PER_POINT;
/// How long an apple keeps giving it back: 30 seconds of action ticks.
pub const APPLE_TICKS: u64 = 30 * ACTION_TICKS_PER_SECOND as u64;

/// The effect of eating one of these at `now`, if it has one. An apple gives
/// back 1 Health a second for 30 seconds, ended by moving or being hurt (owner,
/// 2026-11-14).
pub fn food_effect(kind: ItemKind, now: u64) -> Option<Effect> {
    match kind {
        ItemKind::Apple => Some(Effect {
            source: Source::Eaten(kind),
            bonuses: vec![Bonus::flat(
                Stat::Restore(Vital::Health),
                APPLE_RESTORE as i32,
            )],
            start: now,
            end: Some(now + APPLE_TICKS),
            ends_on: EndsOn {
                moving: true,
                hurt: true,
            },
        }),
        _ => None,
    }
}
