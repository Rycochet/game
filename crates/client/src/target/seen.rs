//! The creatures the player can see now, for the rules in `choose`.

use bevy::prelude::*;

use super::choose::Seen;
use crate::common::{MapPosition, Other};

/// Everyone else on the map: the server only tells of those in sight.
pub(super) fn seen_now(others: &Query<(&Other, &MapPosition)>) -> Vec<Seen> {
    others
        .iter()
        .map(|(other, at)| Seen {
            id: other.id,
            at: (at.x, at.y),
            feeling: other.feeling,
        })
        .collect()
}
