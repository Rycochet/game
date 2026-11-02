//! The game's interface pieces, so they all look alike: panels with titles,
//! buttons, tooltips and panels that can be moved about. Use these rather than
//! styling Bevy's `Node`s directly. See `docs/context/client/widgets.md`.

mod dropdown;
mod feedback;
mod filter_list;
mod fullscreen;
mod item;
mod moveable;
mod scroll;
mod set_edge;
mod slider;
mod soft_keyboard;
mod stock;
mod tabs;
mod text_box;
mod theme;

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;

pub use dropdown::{Dropdown, DropdownChosen, spawn_dropdown};
pub use feedback::{ButtonStyle, ButtonTooltip};
pub use filter_list::{HighlightStep, matches_filter, next_highlight};
pub use fullscreen::{
    darkening, layer_share, layer_z, leaves_presses, spawn_fullscreen_layer, spawn_sized_layer,
};
pub use item::{
    CELL, Floating, HELD_ITEM_Z, ITEM_LABEL_FONT, ItemLook, ItemTip, ItemTooltip,
    PICTURE_PIXELS_PER_CELL, at_cell, covering, item_colour, item_edge, item_edge_width,
    item_label, item_picture, logical_rect, shown_at, spawn_item_cells, spawn_item_grid,
};
pub use moveable::{spawn_moveable_panel, stops_presses};
pub use scroll::spawn_scroll_area;
pub use set_edge::set_edge;
pub use slider::{SliderMoved, spawn_slider};
pub use soft_keyboard::Key as SoftKey;
pub use stock::{StockAim, StockAimAt, StockGrid, StockGridNode, draw_stock_aim};
pub use tabs::{TabChosen, spawn_tabs};
pub use text_box::{SoftKeyPressed, TextBox, TextEdited, TextFocus, spawn_text_box};
pub use theme::{ALLOWED, GRID_COLOUR, GRID_LINE, INK, ITEM_EDGE, PARCHMENT, REFUSED, TextSize};
use theme::{
    DISABLED_BACKGROUND, DISABLED_EDGE, DISABLED_TEXT, LIT_EDGE, RAISED_BACKGROUND, SELECTED_TEXT,
    SHADED_EDGE, TOOLTIP_BACKGROUND, TOOLTIP_TEXT, soft_shadow,
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
/// The two strokes of a tick: thick, then each one's length, where it goes and how
/// it is turned.
const TICK_THICKNESS: f32 = 2.0;
const TICK_STROKES: [(f32, f32, f32, f32); 2] = [(6.0, 1.0, 7.0, 45.0), (10.0, 4.0, 6.0, -45.0)];
/// Between the edge of a button and its picture.
const ICON_PADDING: f32 = 4.0;
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
            .add_plugins(UiMaterialPlugin::<set_edge::SetEdgeMaterial>::default())
            .add_systems(Startup, set_edge::make_set_edges)
            .add_systems(Update, (set_edge::give_set_edges, set_edge::slide_bands))
            .add_systems(Startup, item::spawn_name_box)
            // After `Update`, where it is asked for, and in time to be laid out.
            .add_systems(
                PostUpdate,
                (item::name_item_at_pointer, item::show_item_tooltip)
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
                    moveable::place_panels,
                    scroll::show_scroll_bars,
                    slider::hold_sliders,
                    slider::place_marks,
                    text_box::show_text_boxes,
                    text_box::place_soft_keyboard,
                    dropdown::show_dropdowns,
                ),
            )
            .add_systems(
                PostUpdate,
                (feedback::show_button_looks, feedback::show_hovered_edges)
                    .before(bevy::ui::UiSystems::Layout),
            );
    }
}

/// Where widgets take what they use of the mouse and fingers, in `PreUpdate`,
/// so nothing underneath them sees it. Anything else taking clicks there runs
/// after these.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WidgetSystems {
    /// Presses on widgets become `ButtonPressed`.
    Presses,
    /// As `Modal`, for what stays in front of everything that is in `Modal`.
    Front,
    /// Something covering the screen reads them, and takes all other input.
    Modal,
}

/// Brings this moveable panel to the front.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaisePanel(pub Entity);

/// A button marked `Pressable` was clicked or tapped, unless it was disabled.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonPressed(pub Entity);

/// Put this on a button (from `spawn_button`), or any node, for clicks on it to
/// be sent as `ButtonPressed` and go no further.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Pressable;

/// How a button looks; change it to change the button.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonState {
    /// Standing up from what is behind it.
    #[default]
    Normal,
    /// The one chosen: darker and flat.
    Selected,
    /// Not chosen but special in some way, such as the floor the player is on
    /// while the map shows another: as normal, but with a flat border.
    Active,
    /// Can't be used now: grey, and clicks do nothing.
    Disabled,
    /// As `Disabled`, but with the keyboard on it: a dark border.
    DisabledSelected,
}

impl ButtonState {
    /// Whether clicks on it do nothing.
    pub fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled | Self::DisabledSelected)
    }

    /// Its background, border and text colours.
    fn colours(self) -> (Color, BorderColor, Color) {
        match self {
            Self::Normal => (RAISED_BACKGROUND, raised_border(), INK),
            Self::Selected => (SHADED_EDGE, BorderColor::from(INK), SELECTED_TEXT),
            Self::Active => (RAISED_BACKGROUND, BorderColor::from(INK), INK),
            Self::Disabled => (
                DISABLED_BACKGROUND,
                BorderColor::from(DISABLED_EDGE),
                DISABLED_TEXT,
            ),
            Self::DisabledSelected => (DISABLED_BACKGROUND, BorderColor::from(INK), DISABLED_TEXT),
        }
    }
}

/// Lit along its top and left, in shade along its bottom and right.
fn raised_border() -> BorderColor {
    BorderColor {
        top: LIT_EDGE,
        left: LIT_EDGE,
        right: SHADED_EDGE,
        bottom: SHADED_EDGE,
    }
}

/// A box standing up from what is behind it. Put its contents in as children.
pub fn raised_box() -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(PANEL_EDGE)),
            padding: BUTTON_PADDING,
            ..default()
        },
        BackgroundColor(RAISED_BACKGROUND),
        raised_border(),
    )
}

/// A parchment panel, placed by absolute position, with a soft shadow. Its parts
/// go in as children. Give its `Node` a border of `PANEL_EDGE`, or a `panel_frame`
/// if its children are placed by numbers that shouldn't allow for one.
pub fn panel() -> impl Bundle {
    let [red, green, blue] = PARCHMENT;
    (
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(red, green, blue)),
        raised_border(),
        soft_shadow(),
    )
}

/// A panel's edge, as a child covering it: spawn it first, to be behind the rest.
pub fn panel_frame() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            border: UiRect::all(px(PANEL_EDGE)),
            ..default()
        },
        raised_border(),
    )
}

/// The writing for a panel's title. Add a `Node` to place it.
pub fn panel_title(title: impl Into<String>) -> impl Bundle {
    (Text::new(title), TextSize::Title.font(), TextColor(INK))
}

/// A dark box naming something, such as what is under the pointer; `marker` goes on its text,
/// to find it and change what it says. Hide the holder to hide it.
pub fn spawn_tooltip(
    holder: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    text: impl Into<String>,
) {
    holder
        .spawn((
            Node {
                border: UiRect::all(px(TOOLTIP_EDGE)),
                padding: BUTTON_PADDING,
                ..default()
            },
            BackgroundColor(TOOLTIP_BACKGROUND),
            BorderColor::from(SHADED_EDGE),
            soft_shadow(),
        ))
        .with_child((
            marker,
            Text::new(text),
            TOOLTIP_FONT.font(),
            TextColor(TOOLTIP_TEXT),
        ));
}

/// A button. `node` says its size or padding and where it goes; its border and
/// the centring of its words are set here.
pub fn spawn_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    label: impl Into<String>,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, text) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((
        Text::new(label),
        size.font(),
        TextLayout::justify(Justify::Center),
        TextColor(text),
    ));
    button
}

/// A button showing a picture rather than words. Give it a `ButtonTooltip` to say
/// what it is.
pub fn spawn_icon_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    icon: Handle<Image>,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, _) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            padding: UiRect::all(px(ICON_PADDING)),
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((
        ImageNode::new(icon),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
    ));
    button
}

/// A button with its words on the left, after a checkbox if `checked` says
/// whether it is ticked, or the same space if it has none, so words line up.
pub fn spawn_check_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    checked: Option<bool>,
    label: impl Into<String>,
    size: TextSize,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, text) = state.colours();
    let mut button = parent.spawn((
        state,
        Node {
            border: UiRect::all(px(BUTTON_BORDER)),
            padding: UiRect::horizontal(px(CHECK_GAP)),
            column_gap: px(CHECK_GAP),
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_children(|inside| {
        let mut slot = inside.spawn(Node {
            width: px(CHECK_PIXELS),
            height: px(CHECK_PIXELS),
            flex_shrink: 0.0,
            border: UiRect::all(px(checked.map_or(0.0, |_| BUTTON_BORDER))),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        });
        slot.insert(BorderColor::from(INK));
        if checked == Some(true) {
            slot.with_children(|slot| {
                for (length, left, top, degrees) in TICK_STROKES {
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(left),
                            top: px(top),
                            width: px(length),
                            height: px(TICK_THICKNESS),
                            ..default()
                        },
                        UiTransform::from_rotation(Rot2::degrees(degrees)),
                        BackgroundColor(INK),
                    ));
                }
            });
        }
        inside.spawn((Text::new(label), size.font(), TextColor(text)));
    });
    button
}

/// A close button, with `node` for where it goes.
pub fn spawn_close_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    node: Node,
) -> EntityCommands<'a> {
    let (background, border, _) = ButtonState::Normal.colours();
    let mut button = parent.spawn((
        ButtonState::Normal,
        Node {
            width: px(CLOSE_PIXELS),
            height: px(CLOSE_PIXELS),
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(background),
        border,
    ));
    button.with_child((Text::new("X"), CLOSE_FONT.font(), TextColor(INK)));
    button
}
