//! Drop tables: what kinds of item drop, at what quality.

use super::{ItemKind, Quality};
use crate::rng::Rng;

/// What `Quality::odds` add up to.
pub const ODDS_OUT_OF: u64 = 10_000;

/// One line of a drop table: this kind can drop at this quality, more likely the
/// more it weighs against the others at that quality.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Drop {
    pub quality: Quality,
    pub kind: ItemKind,
    pub weight: u64,
}

/// Both rolls of a drop: first the quality, then the item from those at that
/// quality. If the table has none so good, it takes the best it has below that,
/// or failing that the worst above. `None` if the table is empty. A kind without
/// quality comes back without one.
pub fn roll_drop(rng: &mut Rng, table: &[Drop]) -> Option<(ItemKind, Option<Quality>)> {
    let rolled = Quality::roll(rng);
    let at = table
        .iter()
        .map(|drop| drop.quality)
        .filter(|&quality| quality <= rolled)
        .max()
        .or_else(|| table.iter().map(|drop| drop.quality).min())?;
    let lines = || table.iter().filter(|drop| drop.quality == at);
    let total: u64 = lines().map(|drop| drop.weight).sum();
    let mut left = rng.range_u64(0, total.max(1));
    let mut chosen = None;
    for drop in lines() {
        chosen = Some(drop.kind);
        if left < drop.weight {
            break;
        }
        left -= drop.weight;
    }
    let kind = chosen?;
    Some((kind, kind.def().has_quality.then_some(at)))
}
