//! Keeping each skill's ring up to date as it is practised, in place, so the tree
//! isn't drawn again for every blow.

use bevy::prelude::*;

use super::build::{RingOf, ring_value};
use super::view::SkillsView;
use crate::character::CharacterSheet;
use crate::widgets::SkillRing;

/// Sets each ring from the skill's value while the page is on show, changing only
/// those that differ (the ring widget redraws what changed).
pub(super) fn show_values(
    view: Res<SkillsView>,
    sheet: Res<CharacterSheet>,
    mut rings: Query<(&RingOf, &mut SkillRing)>,
) {
    if !view.shown {
        return;
    }
    let skills = &sheet.0.skills;
    for (of, mut ring) in &mut rings {
        ring.set_if_neq(SkillRing::of(ring_value(skills, of.0), false));
    }
}
