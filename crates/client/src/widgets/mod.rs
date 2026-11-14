//! The game's interface pieces, so they all look alike: panels with titles,
//! buttons, tooltips and panels that can be moved about. Use these rather than
//! styling Bevy's `Node`s directly. See `docs/context/client/widgets.md`.

mod ability;
mod attribute_colour;
mod aura;
mod auto_scroll;
mod clipboard;
mod colour_picker;
#[cfg(not(target_arch = "wasm32"))]
mod copy_button;
#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
mod debug_api;
mod dropdown;
mod feedback;
mod filter_list;
mod fullscreen;
mod hold_button;
mod hover_tip;
mod item;
mod item_text;
mod key_box;
mod moveable;
mod new_seed_icon;
mod outlined;
mod progress;
mod queued_stripe;
mod scale;
mod scroll;
mod set_edge;
mod skill_ring;
mod slider;
mod soft_keyboard;
mod spinner;
mod stock;
mod switch;
mod tabs;
mod text_box;
mod theme;
mod ui_id;
mod weight;

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;
use crate::dom::DomInput;

pub use ability::{ABILITY, AbilityLook, spawn_ability, spawn_outlined_icon};
pub use attribute_colour::{Shade, attribute_colour, row_rgb};
pub use aura::{Aura, AuraMask, aura_glow};
pub use auto_scroll::AutoScroll;
#[cfg(not(target_arch = "wasm32"))]
pub use clipboard::copy_to_clipboard;
pub use colour_picker::{ColourChosen, ColourPicker, Swatch, spawn_colour_picker};
#[cfg(not(target_arch = "wasm32"))]
pub use copy_button::spawn_copy_button;
#[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
pub use debug_api::WidgetPlaces;
pub use dropdown::{Dropdown, DropdownChosen, spawn_dropdown, spawn_icon_dropdown};
pub use feedback::{ButtonStyle, ButtonTooltip};
pub use filter_list::{HighlightStep, matches_filter};
pub use fullscreen::{
    darkening, layer_share, layer_z, spawn_dialog, spawn_fullscreen_layer, spawn_sized_layer,
};
pub use hold_button::HoldButton;
pub use hover_tip::{HoverTip, Pinned, PinnedTip};
pub use item::{
    CELL, HELD_ITEM_Z, ITEM_LABEL_FONT, ItemLook, ItemName, ItemTip, ItemTooltip,
    PICTURE_PIXELS_PER_CELL, Pointed, PointedItem, at_cell, covering, item_colour, item_edge,
    item_edge_width, item_label, item_picture, logical_rect, picture_in_square, shown_at,
    spawn_item_cells,
};
pub use item_text::{ItemText, item_text};
pub use key_box::{KeyBox, KeyChosen, spawn_key_box};
pub use moveable::{MoveablePanel, PanelMoved, PanelPlace, PanelTitleOf};
pub use moveable::{spawn_grip_panel, spawn_moveable_panel};
pub use new_seed_icon::spawn_new_seed_button;
pub use outlined::spawn_outlined_text;
pub use progress::{Progress, spawn_progress_bar};
pub use queued_stripe::queued_stripe;
pub use scale::{INTERFACE_PERCENTS, TouchOnly};
pub use scroll::{spawn_filling_scroll_area, spawn_wide_scroll_area};
pub use set_edge::set_edge;
pub use skill_ring::{SkillRing, skill_ring};
pub use slider::{Slider, SliderMoved, spawn_slider};
pub use soft_keyboard::Key as SoftKey;
pub use spinner::Spinner;
pub use stock::{StockGrid, StockGridNode, StockOf, show_stock_aim};
pub use switch::{Switch, SwitchFlipped, spawn_switch};
pub use tabs::{ChooseTab, TabChosen, set_tabs, spawn_tabs, spawn_toggle_row};
pub use text_box::{SoftKeyPressed, TextBox, TextEdited, TextFocus, field_label, spawn_text_box};
pub use theme::{
    ALLOWED, GRID_COLOUR, GRID_LINE, INK, ITEM_EDGE, PARCHMENT, REFUSED, ROW_TINT, TextSize,
    WARNING_TEXT,
};
use theme::{RAISED_BACKGROUND, SHADED_EDGE};
#[cfg(any(test, feature = "debug-api"))]
pub use ui_id::duplicate_ids;
pub use ui_id::{Checked, UiId};
pub use weight::weight_words;

// Split into files (`cargo xtask lint` keeps files small).
mod boxes;
mod button_state;
mod buttons;

pub use boxes::{TOOLTIP_BOX_EDGE, panel, panel_title, tooltip_box, tooltip_colours, tooltip_trim};
use boxes::{raised_border, spawn_tooltip, sunken_border};
pub use button_state::{ButtonPressed, ButtonState, Pressable, RaisePanel};
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

/// How far a panel's title is drawn above its place.
const TITLE_LIFT: f32 = 2.0;
const TOOLTIP_FONT: TextSize = TextSize::Medium;
const CLOSE_FONT: TextSize = TextSize::Large;
const TOOLTIP_EDGE: f32 = 1.0;
const BUTTON_BORDER: f32 = 2.0;
const CHECK_PIXELS: f32 = 18.0;
const STROKE_THICKNESS: f32 = 2.0;
const CHECK_GAP: f32 = 8.0;

/// Where the item the pointer is on is said (`PointedItem`, `PostUpdate`):
/// whoever reads it goes `.after` this.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ItemsPointed;

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonPressed>()
            .add_message::<RaisePanel>()
            .add_message::<PanelMoved>()
            .add_message::<SliderMoved>()
            .add_message::<ColourChosen>()
            .add_message::<TextEdited>()
            .add_message::<SoftKeyPressed>()
            .add_message::<DropdownChosen>()
            .add_message::<TabChosen>()
            .add_message::<ChooseTab>()
            .add_message::<SwitchFlipped>()
            .add_message::<KeyChosen>()
            .init_resource::<key_box::KeyWaiting>()
            .init_resource::<switch::Slides>()
            .init_resource::<PinnedTip>()
            .init_resource::<TextFocus>()
            .init_resource::<ItemTooltip>()
            .init_resource::<PointedItem>()
            .init_resource::<crate::common::BandShift>()
            .init_resource::<aura::Auras>()
            .add_plugins(UiMaterialPlugin::<aura::AuraMaterial>::default())
            .add_systems(Update, (aura::give_auras, aura::pulse_auras))
            .add_plugins(UiMaterialPlugin::<set_edge::SetEdgeMaterial>::default())
            .add_plugins(UiMaterialPlugin::<skill_ring::SkillRingMaterial>::default())
            .add_systems(Startup, set_edge::make_set_edges)
            .add_plugins(UiMaterialPlugin::<queued_stripe::QueuedStripeMaterial>::default())
            .add_systems(Startup, queued_stripe::make_queued_stripes)
            .add_systems(
                Update,
                (
                    queued_stripe::give_queued_stripes,
                    queued_stripe::chase_stripes,
                ),
            )
            .add_systems(Startup, new_seed_icon::make_new_seed_icon)
            .add_systems(
                Update,
                (
                    set_edge::give_set_edges,
                    (set_edge::drift_bands, set_edge::slide_bands).chain(),
                    skill_ring::give_rings,
                    skill_ring::slide_rings.after(set_edge::drift_bands),
                ),
            )
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
                    item::name_item_at_pointer.in_set(ItemsPointed),
                    hover_tip::show_hover_tips,
                    item::show_item_tooltip,
                )
                    .chain()
                    .before(bevy::ui::UiSystems::Layout),
            )
            .init_resource::<text_box::PressedThisFrame>()
            .init_resource::<text_box::Held>()
            .init_resource::<dropdown::OpenList>()
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
                    text_box::drag_to_select,
                    slider::drag_sliders,
                    scroll::scroll_areas,
                    moveable::take_presses,
                    text_box::select_by_pointer,
                    text_box::take_text,
                    key_box::take_keys,
                    dropdown::run_dropdowns,
                )
                    .chain()
                    .in_set(DomInput::Presses)
                    .after(InputSystems)
                    .after(PointerSystems),
            )
            .add_systems(
                Update,
                (
                    (tabs::choose_tabs, tabs::press_tabs, tabs::show_tab_bottoms).chain(),
                    feedback::spread_ripples,
                    feedback::show_button_tooltips,
                    progress::show_progress,
                    moveable::place_panels,
                    scroll::show_scroll_bars,
                    auto_scroll::auto_scroll,
                    slider::hold_sliders,
                    slider::place_marks,
                    (
                        text_box::take_dropped_items,
                        text_box::show_text_boxes,
                        text_box::show_carets,
                        text_box::show_selections,
                    )
                        .chain(),
                    text_box::place_soft_keyboard,
                    dropdown::show_dropdowns,
                    (switch::flip_switches, switch::show_switches).chain(),
                    buttons::show_checks,
                    (colour_picker::press_swatches, colour_picker::show_swatches).chain(),
                ),
            )
            .init_resource::<scroll::ScrollMemory>()
            .add_systems(
                PostUpdate,
                (
                    feedback::show_button_looks,
                    feedback::show_edges,
                    key_box::show_key_boxes,
                    scroll::bring_back_places,
                )
                    .before(bevy::ui::UiSystems::Layout),
            )
            .add_systems(
                PostUpdate,
                scroll::keep_places.after(bevy::ui::UiSystems::Layout),
            )
            .add_systems(Last, ui_id::warn_of_duplicates);
        item_text::add(app);
        stock::add(app);
        hold_button::add(app);
        #[cfg(all(feature = "debug-api", not(target_arch = "wasm32")))]
        debug_api::add(app);
    }
}
