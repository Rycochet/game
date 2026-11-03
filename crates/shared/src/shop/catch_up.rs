//! Catching a shop's stock up on what was made and sold while no one looked.

use super::{PURPOSE, Shop, TOP_UP_AFTER, sell_weight};
use crate::item::{ItemKind, Quality};
use crate::rng::{Rng, Seed};

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
pub(super) enum Slot {
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
pub(super) fn sell_one(slots: &mut Vec<Slot>, rng: &mut Rng) {
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
