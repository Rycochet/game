//! What the character carries: grids in moveable panels, and picking things up,
//! holding them and putting them down. See `docs/context/client/inventory.md`.

mod body;
mod give;
mod ground;
mod hold;
mod panels;

use bevy::prelude::*;
use give::NearbyPanel;
use panels::OpenGrids;
use shared::item::Inventory;

use crate::common::{GaveItem, HeldOverGifts, HeldStock, PointerSystems, TookItem};
use crate::widgets::WidgetSystems;

// Split into files (`cargo xtask lint` keeps files small).
#[cfg(not(target_arch = "wasm32"))]
mod debug;
mod early_pictures;
mod filter;
mod finger;
mod hold_items;
mod holding;
mod server;
mod sort;
mod split;

#[cfg(not(target_arch = "wasm32"))]
use debug::run_debug;
use finger::{item_cell_in_front, pick_up_by_finger};
use hold_items::hold_items;
use holding::{Hand, HeldBy, Holding, cancel_held};
use server::{Stacks, read_server};
use split::Split;

/// A press on an item this long after the last one on it opens it.
const DOUBLE_PRESS_SECONDS: f64 = 0.35;
/// A press and release this quick, this close together, is a click: what it picked up
/// stays held until the next.
const CLICK_SECONDS: f64 = 0.3;
const CLICK_PIXELS: f32 = 8.0;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Carried>()
            .init_resource::<Stacks>()
            .init_resource::<Holding>()
            .init_resource::<Split>()
            .init_resource::<OpenGrids>()
            .init_resource::<NearbyPanel>()
            .init_resource::<ground::Ground>()
            .init_resource::<HeldOverGifts>()
            .init_resource::<HeldStock>()
            .add_message::<TookItem>()
            .add_message::<OpenCharacter>()
            .add_message::<GaveItem>()
            .add_systems(
                Startup,
                (
                    panels::spawn_fixed_parts,
                    filter::make_icons,
                    sort::make_icon,
                ),
            )
            .add_systems(
                PreUpdate,
                (
                    // Before widgets stop a finger's drag on a panel, and before the
                    // menus see Esc.
                    (pick_up_by_finger, cancel_held, split::close_split)
                        .before(WidgetSystems::Presses)
                        .after(PointerSystems),
                    hold_items.after(WidgetSystems::Modal),
                ),
            )
            .add_systems(
                Update,
                (
                    read_server,
                    panels::toggle_pockets,
                    panels::forget_closed,
                    give::forget_closed,
                    panels::show_bins,
                    panels::show_drop,
                    (ground::read_server, filter::keep_filters).chain(),
                    ground::show_section,
                    ground::show_aim,
                    panels::show_grids,
                    body::show_clothes,
                    body::show_body_aim,
                    give::show_nearby,
                    give::show_give_aim,
                    panels::show_aim,
                    panels::show_held,
                    panels::show_tooltip,
                    panels::show_button,
                    split::move_split,
                    split::show_split,
                )
                    .chain(),
            );
        app.add_systems(Update, early_pictures::ask_early);
        // Before the grids are drawn, so a sort shows at once.
        app.add_systems(
            Update,
            (sort::press_sort, sort::show_icons).before(panels::show_grids),
        );
        #[cfg(not(target_arch = "wasm32"))]
        app.add_message::<InventoryDebug>()
            .add_systems(Update, run_debug.before(panels::forget_closed));
    }
}

/// Opens the Character panel, or brings it to the front.
#[derive(Message, Clone, Copy, Debug)]
pub struct OpenCharacter;

/// What the character carries, as last heard from the server plus any move
/// since, which it answers.
#[derive(Resource, Default)]
pub struct Carried(pub Inventory);

/// For looking at the panels (`dev_tools`).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Message, Clone, Copy, Debug)]
pub enum InventoryDebug {
    OpenPockets,
    /// Opens every bag in the pockets.
    OpenBags,
    /// Picks up the item in the pockets at this index, until the program ends.
    PickUp(usize),
    /// Puts all but the first of what is in the pockets on the ground.
    DropSome,
}

#[cfg(test)]
mod tests;
