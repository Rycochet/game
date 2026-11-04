//! The game's interface pieces, so they all look alike: panels with titles,
//! buttons, tooltips and panels that can be moved about. Use these rather than
//! styling Bevy's `Node`s directly. See `docs/context/client/widgets.md`.

mod aura;
mod auto_scroll;
mod clipboard;
mod container;
mod dropdown;
mod feedback;
mod filter_list;
mod fullscreen;
mod hover_tip;
mod item;
mod moveable;
mod progress;
mod scale;
mod scroll;
mod set_edge;
mod slider;
mod soft_keyboard;
mod spinner;
mod stock;
mod tabs;
mod text_box;
mod theme;
mod weight;

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;

pub use aura::{Aura, AuraMask, aura_glow};
pub use auto_scroll::AutoScroll;
#[cfg(not(target_arch = "wasm32"))]
pub use clipboard::copy_to_clipboard;
pub use container::{ContainerLook, ContainerSize, spawn_container};
pub use dropdown::{Dropdown, DropdownChosen, spawn_dropdown, spawn_icon_dropdown};
pub use feedback::{ButtonStyle, ButtonTooltip};
pub use filter_list::{HighlightStep, matches_filter, next_highlight};
pub use fullscreen::{
    darkening, layer_share, layer_z, leaves_presses, spawn_dialog, spawn_fullscreen_layer,
    spawn_sized_layer,
};
pub use hover_tip::HoverTip;
pub use item::{
    CELL, Floating, HELD_ITEM_Z, ITEM_LABEL_FONT, ItemLook, ItemTip, ItemTooltip,
    PICTURE_PIXELS_PER_CELL, at_cell, covering, item_colour, item_edge, item_edge_width,
    item_label, item_picture, logical_rect, picture_in_square, shown_at, spawn_item_cells,
    spawn_item_grid,
};
pub use moveable::{spawn_moveable_panel, stops_presses};
pub use progress::{Progress, spawn_progress_bar};
pub use scale::{INTERFACE_PERCENTS, TouchOnly};
pub use scroll::{spawn_filling_scroll_area, spawn_scroll_area};
pub use set_edge::set_edge;
pub use slider::{SliderMoved, spawn_slider};
pub use soft_keyboard::Key as SoftKey;
pub use spinner::Spinner;
pub use stock::{StockAim, StockAimAt, StockGrid, StockGridNode, draw_stock_aim};
pub use tabs::{TabChosen, spawn_tabs, spawn_toggle_row};
pub use text_box::{SoftKeyPressed, TextBox, TextEdited, TextFocus, field_label, spawn_text_box};
pub use theme::{ALLOWED, GRID_COLOUR, GRID_LINE, INK, ITEM_EDGE, PARCHMENT, REFUSED, TextSize};
use theme::{RAISED_BACKGROUND, SHADED_EDGE};
pub use weight::weight_words;

// Split into files (`cargo xtask lint` keeps files small).
mod boxes;
mod button_state;
mod buttons;

pub use boxes::{panel, panel_frame, panel_title, raised_box, spawn_tooltip, tooltip_box};
use boxes::{raised_border, sunken_border};
pub use button_state::{ButtonPressed, ButtonState, Pressable, RaisePanel, WidgetSystems};
use buttons::{Stroke, spawn_strokes};
pub use buttons::{
    spawn_button, spawn_check_button, spawn_close_button, spawn_icon_button,
    spawn_small_icon_button,
};

/// The width of the edge of a panel.
pub const PANEL_EDGE: f32 = 3.0;
/// A close button, across.
pub const CLOSE_PIXELS: f32 = 28.0;
/// The padding of a button with no size of its own.
pub const BUTTON_PADDING: UiRect = UiRect {
    left: Val::Px(12.0),
    right: Val::Px(12.0),
    top: Val::Px(4.0),
    bottom: Val::Px(4.0),
};

const TOOLTIP_FONT: TextSize = TextSize::Medium;
const CLOSE_FONT: TextSize = TextSize::Large;
const TOOLTIP_EDGE: f32 = 1.0;
const BUTTON_BORDER: f32 = 2.0;
const CHECK_PIXELS: f32 = 18.0;
const STROKE_THICKNESS: f32 = 2.0;
const CHECK_GAP: f32 = 8.0;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonPressed>()
            .add_message::<RaisePanel>()
            .add_message::<SliderMoved>()
            .add_message::<TextEdited>()
            .add_message::<SoftKeyPressed>()
            .add_message::<DropdownChosen>()
            .add_message::<TabChosen>()
            .init_resource::<TextFocus>()
            .init_resource::<ItemTooltip>()
            .init_resource::<crate::common::BandShift>()
            .init_resource::<aura::Auras>()
            .add_plugins(UiMaterialPlugin::<aura::AuraMaterial>::default())
            .add_systems(Update, (aura::give_auras, aura::pulse_auras))
            .add_plugins(UiMaterialPlugin::<set_edge::SetEdgeMaterial>::default())
            .add_systems(Startup, set_edge::make_set_edges)
            .add_systems(Update, (set_edge::give_set_edges, set_edge::slide_bands))
            .add_systems(
                Startup,
                (item::spawn_name_box, spinner::make_spinner_picture),
            )
            .add_systems(
                Update,
                (spinner::picture_spinners, spinner::turn_spinners).chain(),
            )
            // After `Update`, where it is asked for, and in time to be laid out.
            .add_systems(
                PostUpdate,
                (
                    item::name_item_at_pointer,
                    hover_tip::show_hover_tips,
                    item::show_item_tooltip,
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            )
            .init_resource::<text_box::PressedThisFrame>()
            .init_resource::<dropdown::OpenList>()
            .configure_sets(
                PreUpdate,
                (
                    WidgetSystems::Presses,
                    WidgetSystems::Front,
                    WidgetSystems::Modal,
                )
                    .chain(),
            )
            .insert_resource(scale::TouchOnly::detect())
            .add_systems(
                PreUpdate,
                scale::apply_interface_scale.before(PointerSystems),
            )
            .init_resource::<moveable::Dragging>()
            .init_resource::<feedback::Pointed>()
            .init_resource::<slider::HeldSlider>()
            .add_systems(
                PreUpdate,
                (
                    feedback::track_pointer,
                    text_box::note_press,
                    slider::drag_sliders,
                    scroll::scroll_areas,
                    moveable::take_presses,
                    text_box::take_text,
                    dropdown::run_dropdowns,
                )
                    .chain()
                    .in_set(WidgetSystems::Presses)
                    .after(InputSystems)
                    .after(PointerSystems),
            )
            .add_systems(
                Update,
                (
                    tabs::press_tabs,
                    feedback::spread_ripples,
                    feedback::show_button_tooltips,
                    progress::show_progress,
                    moveable::place_panels,
                    scroll::show_scroll_bars,
                    auto_scroll::auto_scroll,
                    slider::hold_sliders,
                    slider::place_marks,
                    text_box::show_text_boxes,
                    text_box::place_soft_keyboard,
                    dropdown::show_dropdowns,
                ),
            )
            .add_systems(
                PostUpdate,
                (feedback::show_button_looks, feedback::show_edges)
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}
