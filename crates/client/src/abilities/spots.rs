//! What is at a point of the interface, for abilities: a slot of the bar, an
//! ability on the Abilities page, or elsewhere on that page.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use shared::ability::Ability;

use super::bar::SlotNode;
use super::page::PageAbility;
use crate::common::AbilitiesPage;
use crate::dom::Finder;

/// Where something can be picked up or dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Spot {
    /// A slot of the action bar.
    Slot(u8),
    /// An ability on the Abilities page.
    Listed(Ability),
    /// Anywhere else on the Abilities page.
    Page,
}

/// Finds the spot at a point: whatever is in front there (`dom`'s one
/// search), or holds it.
#[derive(SystemParam)]
pub(super) struct Spots<'w, 's> {
    finder: Finder<'w, 's>,
    parents: Query<'w, 's, &'static ChildOf>,
    slots: Query<'w, 's, &'static SlotNode>,
    listed: Query<'w, 's, &'static PageAbility>,
    pages: Query<'w, 's, (), With<AbilitiesPage>>,
}

impl Spots<'_, '_> {
    /// The spot at `at` (in interface pixels), if there is one.
    pub(super) fn at(&self, at: Vec2) -> Option<Spot> {
        self.of(self.finder.target_at(at)?)
    }

    /// The spot this node, in front somewhere, is or is in.
    pub(super) fn of(&self, front: Entity) -> Option<Spot> {
        std::iter::once(front)
            .chain(self.parents.iter_ancestors(front))
            .find_map(|entity| {
                if let Ok(slot) = self.slots.get(entity) {
                    Some(Spot::Slot(slot.0))
                } else if let Ok(listed) = self.listed.get(entity) {
                    Some(Spot::Listed(listed.0))
                } else if self.pages.contains(entity) {
                    Some(Spot::Page)
                } else {
                    None
                }
            })
    }
}
