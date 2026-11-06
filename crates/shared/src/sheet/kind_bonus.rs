//! Kinds' bonuses, balanced: every kind has the same points to share out. What a
//! kind sets is taken first, and the rest is shared evenly among what it doesn't
//! (owner, 2026-11-06).

use super::{Attribute, Attributes};

/// Every kind's bonus points, all together. *To balance:* 6 each when a kind
/// sets none.
pub const BONUS_POINTS: u16 = 54;

/// A kind's bonus on each attribute, from what it sets. The points left after
/// those are shared evenly among the rest, and any left over after sharing go
/// one each to the first of them, row by row. Setting one to 0 keeps it out of
/// the sharing. A kind setting more than `BONUS_POINTS` keeps what it set, and
/// shares nothing.
pub fn bonus_from(set: &[(Attribute, u16)]) -> Attributes {
    let mut bonus = Attributes::default();
    for &(which, points) in set {
        bonus.set(which, points);
    }
    let unset: Vec<Attribute> = Attribute::ALL
        .into_iter()
        .filter(|which| !set.iter().any(|(chosen, _)| chosen == which))
        .collect();
    let used: u16 = set.iter().map(|&(_, points)| points).sum();
    let left = BONUS_POINTS.saturating_sub(used);
    if let Some(count) = (!unset.is_empty()).then_some(unset.len() as u16) {
        let (each, over) = (left / count, left % count);
        for (place, &which) in unset.iter().enumerate() {
            bonus.set(which, each + u16::from((place as u16) < over));
        }
    }
    bonus
}

#[cfg(test)]
mod tests;
