//! What the character carries: grids in moveable panels, and picking things up,
//! holding them and putting them down. See `docs/context/client/inventory.md`.

mod body;
mod give;
mod ground;
mod hands;
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
mod ui_ids;
mod wear_note;

#[cfg(not(target_arch = "wasm32"))]
use debug::run_debug;
use finger::{item_cell_in_front, pick_up_by_finger};
use hold_items::{aim_items, hold_items};
use holding::{Hand, HeldBy, Holding, cancel_held};
pub use server::Stacks;
use server::read_server;
use split::Split;

/// A press on an item this long after the last one on it opens it.
const DOUBLE_PRESS_SECONDS: f64 = 0.35;

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
            .init_resource::<wear_note::WearNote>()
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
                    (pick_up_by_finger, cancel_held)
                        .before(WidgetSystems::Presses)
                        .after(PointerSystems),
                    // Once the split box has said whether it closed itself.
                    split::close_split
                        .after(crate::dom::ClosePopups)
                        .before(WidgetSystems::Presses),
                    // Either side of `dom` moving what is held.
                    (
                        hold_items.before(crate::dom::HoldObjects),
                        aim_items.after(crate::dom::HoldObjects),
                    )
                        .after(WidgetSystems::Modal),
                ),
            )
            .add_systems(
                Update,
                (
                    read_server,
                    panels::toggle_inventory,
                    panels::forget_closed,
                    give::forget_closed,
                    panels::show_bins,
                    panels::show_drop,
                    (ground::read_server, filter::keep_filters).chain(),
                    (ground::bind_section, ground::show_section).chain(),
                    ground::show_aim,
                    (
                        (panels::bind_frames, panels::bind_pages),
                        (panels::show_grids, panels::show_weights),
                    )
                        .chain(),
                    body::show_clothes,
                    (body::show_body_aim, wear_note::show_wear_note),
                    (give::show_nearby, give::bind_rows).chain(),
                    give::show_give_aim,
                    panels::show_aim,
                    panels::show_tooltip,
                    panels::show_button,
                    split::move_split,
                    (split::show_split, split::bind_split).chain(),
                )
                    .chain(),
            );
        // Once the grids have drawn themselves.
        app.add_systems(
            PostUpdate,
            panels::mark_grids.after(crate::dom::DrawObjects),
        );
        app.add_systems(Update, early_pictures::ask_early);
        app.add_systems(Update, panels::switch_pages.after(panels::toggle_inventory));
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
    OpenInventory,
    /// Opens every bag in the inventory.
    OpenBags,
    /// Picks up the item in the inventory at this index, until the program ends.
    PickUp(usize),
    /// Puts all but the first of what is in the inventory on the ground.
    DropSome,
}
