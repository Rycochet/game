//! What town shops have in stock: how good it tends to be, how long it takes to
//! make, what sells, and catching up on all that when a shop is next opened. See
//! `docs/context/shared/shop.md`.
//!
//! All the numbers here are first guesses. Only the server calls this: the seed
//! is secret. It makes no items, as ids are the server's: it says what to keep
//! and what to add.

use crate::item::{ItemKind, ODDS_OUT_OF, Quality};
use crate::rng::{Rng, Seed};
use crate::site::BuildingKind;

/// Marks these hashes as being for shops, apart from anything else from the seed.
const PURPOSE: i64 = 0x0073_686f;

const MINUTES_PER_DAY: u64 = 24 * 60;

/// How much time is caught up on. Past this, a shop with one of a kind in town
/// would have replaced everything it had.
pub const REPLACED_AFTER: u64 = 90 * MINUTES_PER_DAY;

/// How long a shop must have been left for it to be topped up on opening, so that
/// buying everything and coming straight back gets nothing.
pub const TOP_UP_AFTER: u64 = MINUTES_PER_DAY;

/// The least and most a shop aims to have, for the worst and best of shops.
const STOCK_AT_WORST: u64 = 5;
const STOCK_AT_BEST: u64 = 30;

/// The share of its count, in 100, below which a shop reopening is topped up.
const MINIMUM_SHARE: u64 = 80;

/// How a shop's make time is scaled, as a percentage, at the worst and best.
const SPEED_AT_WORST: u64 = 150;
const SPEED_AT_BEST: u64 = 50;

/// How near its door someone must be to deal with a shop, in cells.
pub const DOOR_REACH: i64 = 1;

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

/// What a kind of building sells, with how much each weighs: nothing if it isn't
/// a shop. Only placeholders exist to sell for now.
pub fn sells(kind: BuildingKind) -> &'static [(ItemKind, u64)] {
    use ItemKind::{
        Apple, Backpack, Barding, Belt, Boomerang, Boots, Caparison, Cloak, DemiGauntlets,
        FingerlessGloves, FullArmour, Gambeson, Gauntlets, Gloves, Hat, Lantern, Necklace,
        PaddedGloves, Pouch, Ring, Shoes, ShoulderArmour, SmallBox, Socks, Surcoat, Sword, TShirt,
        Tabard, Trousers,
    };
    match kind {
        BuildingKind::Blacksmith => &[(Sword, 2), (Gauntlets, 2), (Lantern, 1)],
        BuildingKind::Herbalist => &[(Apple, 3), (Pouch, 1)],
        BuildingKind::Carpenter => &[(SmallBox, 2), (Boomerang, 1)],
        BuildingKind::Tailor => &[
            (TShirt, 3),
            (Trousers, 3),
            (Socks, 2),
            (Hat, 2),
            (Cloak, 1),
            (Gloves, 1),
            (FingerlessGloves, 1),
            (PaddedGloves, 1),
            (Surcoat, 1),
            (Tabard, 1),
        ],
        BuildingKind::Leatherworker => &[
            (Belt, 2),
            (Boots, 2),
            (Shoes, 2),
            (Pouch, 2),
            (Backpack, 1),
            (Gloves, 1),
        ],
        BuildingKind::Fletcher => &[(Boomerang, 1)],
        BuildingKind::Weaponsmith => &[(Sword, 2), (Boomerang, 1)],
        BuildingKind::Armoursmith => &[
            (FullArmour, 1),
            (Gambeson, 1),
            (ShoulderArmour, 2),
            (Gauntlets, 2),
            (DemiGauntlets, 1),
            (PaddedGloves, 1),
            (Boots, 2),
            (Barding, 1),
            (Caparison, 1),
        ],
        BuildingKind::PotionShop => &[(Apple, 1), (SmallBox, 1)],
        BuildingKind::ScrollShop => &[(Lantern, 1), (SmallBox, 1)],
        BuildingKind::Jeweller => &[(Ring, 2), (Necklace, 1)],
        BuildingKind::GemCutter => &[(Ring, 1), (Necklace, 1)],
        BuildingKind::BlackMarket => &[(Sword, 1), (Ring, 1), (Necklace, 1), (Cloak, 1)],
        _ => &[],
    }
}

/// How many steps better than Common this is, or worse.
const fn steps_from_common(quality: Quality) -> i64 {
    quality as i64 - Quality::Common as i64
}

/// How long something of this quality takes to make, in game minutes, before
/// anything else. Nothing here makes artifacts: they take a questline.
const fn base_minutes(quality: Quality) -> u64 {
    match quality {
        Quality::Poor => 30,
        Quality::Common => 60,
        Quality::Uncommon => 4 * 60,
        Quality::Rare => MINUTES_PER_DAY,
        Quality::Epic => 7 * MINUTES_PER_DAY,
        Quality::Legendary | Quality::Artifact => 30 * MINUTES_PER_DAY,
    }
}

/// How likely something in stock is to be the one sold, as a weight: the better,
/// the longer it stays, but a legendary still goes half as readily as a common.
const fn sell_weight(quality: Quality) -> u64 {
    match quality {
        Quality::Poor => 110,
        Quality::Common => 100,
        Quality::Uncommon => 88,
        Quality::Rare => 75,
        Quality::Epic => 62,
        Quality::Legendary | Quality::Artifact => 50,
    }
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
    fn roll_quality(&self, rng: &mut Rng) -> Quality {
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
    fn average_kind_percent(&self) -> u64 {
        let goods = sells(self.kind);
        let total: u64 = goods.iter().map(|&(_, weight)| weight).sum();
        let sum: u64 = goods
            .iter()
            .map(|&(kind, weight)| kind.def().make_percent * weight)
            .sum();
        sum.checked_div(total).unwrap_or(100).max(1)
    }

    fn roll_kind(&self, rng: &mut Rng) -> Option<ItemKind> {
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

/// Something new in a shop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NewStock {
    pub kind: ItemKind,
    pub quality: Option<Quality>,
    /// The game minute it was made.
    pub minute: u64,
}

/// How a shop stands on opening.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Restocked {
    /// Which of the old stock is left, by index, in order.
    pub keep: Vec<usize>,
    pub new: Vec<NewStock>,
    /// When the last thing was finished: for next time.
    pub last_made: Option<u64>,
}

/// Something in stock while catching up: only its quality is known.
#[derive(Clone, Copy)]
enum Slot {
    Old(usize, Quality),
    New(u64, Quality),
}

impl Slot {
    fn quality(self) -> Quality {
        match self {
            Self::Old(_, quality) | Self::New(_, quality) => quality,
        }
    }
}

/// One sold from `slots`, likelier the less good.
fn sell_one(slots: &mut Vec<Slot>, rng: &mut Rng) {
    let weigh = |slot: &Slot| sell_weight(slot.quality());
    let total: u64 = slots.iter().map(weigh).sum();
    let mut left = rng.range_u64(0, total.max(1));
    let sold = slots.iter().position(|slot| {
        let weight = weigh(slot);
        let here = left < weight;
        left = left.saturating_sub(weight);
        here
    });
    if let Some(sold) = sold {
        slots.remove(sold);
    }
}

/// Catches a shop up to `now` from when it was last dealt with, given the
/// qualities of what it has. Only qualities are rolled until the time is filled:
/// each roll comes with the time to make it, and each finished at its count
/// pushes one out. Kinds are rolled only for what is left. The clock can be moved
/// back (dev tools): that counts as no time having passed. The same for the same
/// answers, and something part made is the same when it is next looked at.
pub fn catch_up(
    seed: Seed,
    shop: &Shop,
    stock: &[Quality],
    last_made: Option<u64>,
    last_visit: Option<u64>,
    now: u64,
) -> Restocked {
    let rolls = |what: i64, minute: u64| Seed::new(seed.at(&[PURPOSE, what, minute as i64])).rng();
    let limit = shop.replaced_after();
    let oldest = now.saturating_sub(limit);
    let (mut cursor, mut slots): (u64, Vec<Slot>) = match last_made {
        Some(made) if made >= oldest => (
            made,
            stock
                .iter()
                .enumerate()
                .map(|(index, &quality)| Slot::Old(index, quality))
                .collect(),
        ),
        _ => (oldest, Vec::new()),
    };
    let start = cursor;
    let target = shop.target() as usize;
    let percent = shop.average_kind_percent();
    loop {
        let mut rng = rolls(0, cursor);
        let quality = shop.roll_quality(&mut rng);
        let finished = cursor.saturating_add(shop.make_minutes(quality, percent));
        if finished > now {
            break;
        }
        cursor = finished;
        slots.push(Slot::New(cursor, quality));
        if slots.len() > target {
            sell_one(&mut slots, &mut rng);
        }
    }
    // Left long enough, it has made up what was missing. Those extras take none of
    // its time, and are spread over the time it was left.
    let left_for = last_visit.map_or(u64::MAX, |visit| now.saturating_sub(visit));
    if left_for >= TOP_UP_AFTER && (slots.len() as u64) < shop.minimum() {
        let mut rng = rolls(1, now);
        let goal = rng.range_u64(shop.minimum(), shop.target() + 1) as usize;
        let extras = goal - slots.len();
        for extra in 0..extras as u64 {
            let quality = shop.roll_quality(&mut rng);
            let minute = start + (now - start.min(now)) * extra / extras as u64;
            slots.push(Slot::New(minute, quality));
        }
    }
    let mut keep = Vec::new();
    let mut new = Vec::new();
    for (place, slot) in slots.into_iter().enumerate() {
        match slot {
            Slot::Old(index, _) => keep.push(index),
            Slot::New(minute, quality) => {
                let mut rng = rolls(2 + place as i64, minute);
                if let Some(kind) = shop.roll_kind(&mut rng) {
                    let quality = kind.def().has_quality.then_some(quality);
                    new.push(NewStock {
                        kind,
                        quality,
                        minute,
                    });
                }
            }
        }
    }
    Restocked {
        keep,
        new,
        last_made: Some(cursor),
    }
}

#[cfg(test)]
mod tests;
