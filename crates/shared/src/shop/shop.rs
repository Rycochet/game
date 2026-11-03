//! A town shop: how much it keeps in stock, and its rolls.

use super::{
    MINIMUM_SHARE, REPLACED_AFTER, SPEED_AT_BEST, SPEED_AT_WORST, STOCK_AT_BEST, STOCK_AT_WORST,
    base_minutes, sells, steps_from_common,
};
use crate::item::{ItemKind, ODDS_OUT_OF, Quality};
use crate::rng::Rng;
use crate::site::BuildingKind;

/// What a shop has been told of itself and its town.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shop {
    pub kind: BuildingKind,
    /// From 0 to 1000.
    pub quality: u16,
    /// Those it was generated with and those made since.
    pub trades: u32,
    /// How many shops of this kind the town has, itself included.
    pub of_kind: u32,
}

impl Shop {
    /// How many things it aims to have.
    pub fn target(&self) -> u64 {
        STOCK_AT_WORST + (STOCK_AT_BEST - STOCK_AT_WORST) * self.quality as u64 / 1000
    }

    /// The fewest it may have on reopening before being topped up.
    pub fn minimum(&self) -> u64 {
        (self.target() * MINIMUM_SHARE).div_ceil(100)
    }

    /// `sqrt` is one of the float functions that is the same everywhere.
    fn rivals_speed(&self) -> f64 {
        (self.of_kind.max(1) as f64).sqrt()
    }

    /// How long until everything it had would have been replaced: competition
    /// makes and sells faster.
    pub fn replaced_after(&self) -> u64 {
        (REPLACED_AFTER as f64 / self.rivals_speed()) as u64
    }

    /// The minutes this shop takes to make something this good, of a kind taking
    /// this percentage of the usual time.
    pub fn make_minutes(&self, quality: Quality, kind_percent: u64) -> u64 {
        let spread = (SPEED_AT_WORST - SPEED_AT_BEST) * self.quality as u64 / 1000;
        let speed = SPEED_AT_WORST - spread;
        let usual = base_minutes(quality) * kind_percent * speed;
        ((usual as f64 / 10_000.0 / self.rivals_speed()) as u64).max(1)
    }

    /// The share of `ODDS_OUT_OF` each quality has here, as weights. Better shops,
    /// and busier ones, make better things. Nothing here is an artifact.
    fn odds(&self) -> [(Quality, u64); 6] {
        let trades = (self.trades as i64 / 20).min(50);
        let power = self.quality as i64 / 10 + trades - 50;
        [
            Quality::Poor,
            Quality::Common,
            Quality::Uncommon,
            Quality::Rare,
            Quality::Epic,
            Quality::Legendary,
        ]
        .map(|quality| {
            let percent = (100 + power * steps_from_common(quality)).max(10) as u64;
            (quality, quality.odds() * percent)
        })
    }

    /// How good the next thing it makes is.
    pub(super) fn roll_quality(&self, rng: &mut Rng) -> Quality {
        let odds = self.odds();
        let total: u64 = odds.iter().map(|&(_, weight)| weight).sum();
        let mut left = rng.range_u64(0, total.max(ODDS_OUT_OF));
        for (quality, weight) in odds {
            if left < weight {
                return quality;
            }
            left -= weight;
        }
        Quality::Common
    }

    /// What share of the usual time its goods take on average, weighted by how
    /// often each is made. Time is filled before the kinds are rolled.
    pub(super) fn average_kind_percent(&self) -> u64 {
        let goods = sells(self.kind);
        let total: u64 = goods.iter().map(|&(_, weight)| weight).sum();
        let sum: u64 = goods
            .iter()
            .map(|&(kind, weight)| kind.def().make_percent * weight)
            .sum();
        sum.checked_div(total).unwrap_or(100).max(1)
    }

    pub(super) fn roll_kind(&self, rng: &mut Rng) -> Option<ItemKind> {
        let goods = sells(self.kind);
        let total: u64 = goods.iter().map(|&(_, weight)| weight).sum();
        let mut left = rng.range_u64(0, total.max(1));
        for &(kind, weight) in goods {
            if left < weight {
                return Some(kind);
            }
            left -= weight;
        }
        goods.first().map(|&(kind, _)| kind)
    }
}
