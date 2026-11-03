//! The menu's parts: warning button, layers, close button, filter box, grid, entries.

use bevy::prelude::*;
use shared::poi::PoiKind;

use super::super::actions::DebugAction;

#[derive(Component)]
pub(in crate::debug_menu) struct WarningButton;

/// A layer of the menu.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct MenuRoot(pub(super) usize);

/// Holds the close button, and takes the presses on it.
#[derive(Component)]
pub(in crate::debug_menu) struct MenuClose;

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct FilterBox(pub(super) usize);

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct MenuGrid(pub(super) usize);

/// A button moving a layer's columns on or back; they only show when they can't
/// all fit.
#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct PageButton {
    pub(super) layer: usize,
    pub(super) forward: bool,
}

/// What an entry in a list is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Entry {
    Action(DebugAction),
    Kind(PoiKind),
}

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct MenuEntry(pub(super) Entry);
