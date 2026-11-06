//! Who the player has targeted.

use bevy::prelude::*;
use shared::id::Id;

/// The creature the player has targeted, by its id, if any. Kept while it is
/// out of sight, so it is still the target if seen again; a death out of sight
/// is never told, so the id may be of someone who has gone for good. Set by
/// `target`.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target(pub Option<Id>);
