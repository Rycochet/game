//! The entries on show for what is typed, and what Return chooses.

use bevy::prelude::*;
use shared::poi::PoiKind;

use super::super::actions::{DebugAction, Facts, kind_name};
use super::{ACTIONS, Entry};
use crate::widgets::matches_filter;

/// The actions on show, in order.
pub(super) fn on_show(filter: &str) -> Vec<DebugAction> {
    DebugAction::ALL
        .into_iter()
        .filter(|action| action.matches(filter))
        .collect()
}

/// The kinds of poi on show in the submenu: alphabetical, as the menu is.
pub(super) fn kinds_on_show(filter: &str) -> Vec<PoiKind> {
    let mut kinds: Vec<PoiKind> = PoiKind::ALL
        .into_iter()
        .filter(|&kind| matches_filter(&kind_name(kind), filter))
        .collect();
    kinds.sort_by_key(|&kind| kind_name(kind));
    kinds
}

/// The entries on show in a layer.
pub(super) fn entries_on_show(layer: usize, filter: &str) -> Vec<Entry> {
    if layer == ACTIONS {
        on_show(filter).into_iter().map(Entry::Action).collect()
    } else {
        kinds_on_show(filter).into_iter().map(Entry::Kind).collect()
    }
}

/// The heading each entry is under: its group's, or none in the submenu.
pub(super) fn headings_of(entries: &[Entry]) -> Vec<Option<&'static str>> {
    entries
        .iter()
        .map(|entry| match entry {
            Entry::Action(action) => Some(action.group().name()),
            Entry::Kind(_) => None,
        })
        .collect()
}

/// What Return chooses: the highlighted entry, or with none, the first that
/// can be used. A disabled entry that is highlighted does nothing.
pub(super) fn return_chooses(
    shown: &[DebugAction],
    highlight: Option<usize>,
    facts: &Facts,
) -> Option<DebugAction> {
    match highlight {
        Some(at) => shown.get(at).copied(),
        None => shown.iter().copied().find(|a| a.is_available(facts)),
    }
    .filter(|action| action.is_available(facts))
}
