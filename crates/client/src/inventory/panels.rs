//! Drawing what is carried: the Character panel, with the inventory's grid in it, a
//! moveable panel per open bag, the button and keys, the item being held, and its
//! name.

use bevy::prelude::*;
use shared::id::Id;
use shared::item::{Holder, Rotation};

use crate::widgets::TextSize;

// Split into files (`cargo xtask lint` keeps files small).
mod aim;
mod bins;
mod fixed;
mod grids;
mod held;
mod icons;
mod mark;
mod own;
mod pages;
mod show;
mod shown;

pub(super) use aim::show_aim;
pub(super) use bins::{show_bins, show_drop};
pub(super) use fixed::spawn_fixed_parts;
pub(super) use held::{show_button, show_tooltip};
use icons::{bin_icon, drop_icon, person_icon};
pub(super) use mark::mark_grids;
pub(super) use own::{forget_closed, toggle_inventory};
pub(super) use pages::switch_pages;
use pages::{Page, Pages};
pub(super) use show::{show_grids, show_weights};

const BUTTON: f32 = 34.0;
const COLUMN_GAP: f32 = 16.0;
const CORNER_GAP: f32 = 8.0;
const FONT: TextSize = TextSize::Medium;
/// How far from the pointer an item's name goes.
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);
/// How far above a held item its name goes, to its top.
const HELD_NAME_ABOVE: f32 = 48.0;
/// Where the first panel goes, and how far along each one after it.
const FIRST_AT: Vec2 = Vec2::new(90.0, 140.0);
const NEXT_AT: Vec2 = Vec2::new(36.0, 36.0);
/// The most of the window's height a growing bag's grid takes, before it scrolls.
const MOST_HEIGHT: Val = Val::Vh(60.0);

/// The panels open, in the order they were opened.
#[derive(Resource, Default)]
pub(super) struct OpenGrids {
    /// Each with the size its grid was drawn at.
    open: Vec<(Holder, Entity, (u8, u8))>,
    opened: u32,
    /// The Character panel's tabs, while it is open.
    pages: Option<Pages>,
    /// The tab it shows, or will open on: the last chosen.
    page: Page,
}

/// A grid as its panel spawns it: `dom`'s grid of things (`GridShown`), which
/// draws itself, and whose cells `mark_grids` makes ours.
#[derive(Component, Clone, Copy)]
pub(super) struct GridOwner {
    pub holder: Holder,
}

/// The words under a grid saying how heavy things are.
#[derive(Component, Clone, Copy)]
pub(super) struct WeightText {
    holder: Holder,
}

/// One cell of an item drawn in a grid: a press on it picks the item up by this
/// cell. Also on the label over it, for finding what a finger is on.
#[derive(Component, Clone, Copy, Debug)]
pub(super) struct ItemCell {
    pub id: Id,
    pub rotation: Rotation,
    pub grip: (u8, u8),
}

#[derive(Component)]
pub(super) struct InventoryButton;

/// The bin and what is in it: what is dropped here is thrown away.
#[derive(Component)]
pub(super) struct BinNode;

/// The bin itself, under what is in it.
#[derive(Component)]
pub(super) struct BinSlot;

/// The bin's picture.
#[derive(Resource)]
pub(super) struct BinIcon(Handle<Image>);

/// The Drop icon: what is dropped here goes on the ground under the player.
#[derive(Component)]
pub(super) struct DropNode;

/// The Drop icon's picture.
#[derive(Resource)]
pub(super) struct DropIcon(Handle<Image>);
