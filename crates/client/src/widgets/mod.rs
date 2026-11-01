//! The game's interface pieces, so they all look alike: panels with titles,
//! buttons, tooltips and panels that can be moved about. Use these rather than
//! styling Bevy's `Node`s directly. See `docs/context/client/widgets.md`.

mod dropdown;
mod filter_list;
mod fullscreen;
mod moveable;
mod slider;
mod soft_keyboard;
mod text_box;

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;

pub use dropdown::{Dropdown, DropdownChosen, spawn_dropdown};
pub use filter_list::{HighlightStep, matches_filter, next_highlight};
pub use fullscreen::{
    LAYER_EDGE, darkening, layer_share, layer_z, leaves_presses, spawn_fullscreen_layer,
};
pub use moveable::{spawn_moveable_panel, stops_presses};
pub use slider::{SliderMoved, spawn_slider};
pub use soft_keyboard::Key as SoftKey;
pub use text_box::{SoftKeyPressed, TextBox, TextEdited, TextFocus, spawn_text_box};

/// The inside of panels, the colour of parchment.
pub const PARCHMENT: [u8; 3] = [240, 225, 190];
/// Writing and lines on parchment.
pub const INK: Color = Color::srgb(0.25, 0.17, 0.1);
/// A close button, across.
pub const CLOSE_PIXELS: f32 = 28.0;
/// The padding of a button with no size of its own.
pub const BUTTON_PADDING: UiRect = UiRect {
    left: Val::Px(14.0),
    right: Val::Px(14.0),
    top: Val::Px(5.0),
    bottom: Val::Px(5.0),
};

const TITLE_FONT: f32 = 24.0;
const TOOLTIP_FONT: f32 = 18.0;
const CLOSE_FONT: f32 = 18.0;
const BUTTON_BORDER: f32 = 2.0;
const CHECK_PIXELS: f32 = 18.0;
const CHECK_MARK_PIXELS: f32 = 10.0;
const CHECK_GAP: f32 = 8.0;

const RAISED_BACKGROUND: Color = Color::srgb(0.75, 0.65, 0.48);
const LIT_EDGE: Color = Color::srgb(0.95, 0.87, 0.7);
const SHADED_EDGE: Color = Color::srgb(0.42, 0.32, 0.2);
const SELECTED_TEXT: Color = LIT_EDGE;
const DISABLED_BACKGROUND: Color = Color::srgb(0.62, 0.59, 0.53);
const DISABLED_EDGE: Color = Color::srgb(0.5, 0.48, 0.44);
const DISABLED_TEXT: Color = Color::srgb(0.42, 0.4, 0.37);

pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ButtonPressed>()
            .add_message::<RaisePanel>()
            .add_message::<SliderMoved>()
            .add_message::<TextEdited>()
            .add_message::<SoftKeyPressed>()
            .add_message::<DropdownChosen>()
            .init_resource::<TextFocus>()
            .init_resource::<text_box::PressedThisFrame>()
            .init_resource::<dropdown::OpenList>()
            .configure_sets(
                PreUpdate,
                (WidgetSystems::Presses, WidgetSystems::Modal).chain(),
            )
            .init_resource::<moveable::Dragging>()
            .init_resource::<slider::HeldSlider>()
            .add_systems(
                PreUpdate,
                (
                    text_box::note_press,
                    slider::drag_sliders,
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
                    show_button_states,
                    moveable::place_panels,
                    slider::hold_sliders,
                    slider::place_marks,
                    text_box::show_text_boxes,
                    text_box::place_soft_keyboard,
                    dropdown::show_dropdowns,
                ),
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
            border: UiRect::all(px(3)),
            padding: BUTTON_PADDING,
            ..default()
        },
        BackgroundColor(RAISED_BACKGROUND),
        raised_border(),
    )
}

/// A parchment panel, placed by absolute position. Its parts go in as children.
pub fn panel() -> impl Bundle {
    let [red, green, blue] = PARCHMENT;
    (
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgb_u8(red, green, blue)),
    )
}

/// The writing for a panel's title. Add a `Node` to place it.
pub fn panel_title(title: impl Into<String>) -> impl Bundle {
    (
        Text::new(title),
        TextFont::from_font_size(TITLE_FONT),
        TextColor(INK),
    )
}

/// A named box, such as what is under the pointer; `marker` goes on its text,
/// to find it and change what it says. Hide the holder to hide it.
pub fn spawn_tooltip(
    holder: &mut ChildSpawnerCommands,
    marker: impl Bundle,
    text: impl Into<String>,
) {
    holder.spawn(raised_box()).with_child((
        marker,
        Text::new(text),
        TextFont::from_font_size(TOOLTIP_FONT),
        TextColor(INK),
    ));
}

/// A button. `node` says its size or padding and where it goes; its border and
/// the centring of its words are set here.
pub fn spawn_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    state: ButtonState,
    label: impl Into<String>,
    font_size: f32,
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
        TextFont::from_font_size(font_size),
        TextLayout::justify(Justify::Center),
        TextColor(text),
    ));
    button
}

/// A button showing a picture rather than words, which fills it.
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
    font_size: f32,
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
            slot.with_child((
                Node {
                    width: px(CHECK_MARK_PIXELS),
                    height: px(CHECK_MARK_PIXELS),
                    ..default()
                },
                BackgroundColor(INK),
            ));
        }
        inside.spawn((
            Text::new(label),
            TextFont::from_font_size(font_size),
            TextColor(text),
        ));
    });
    button
}

/// A close button, with `node` for where it goes.
pub fn spawn_close_button<'a>(
    parent: &'a mut ChildSpawnerCommands,
    node: Node,
) -> EntityCommands<'a> {
    let mut button = parent.spawn((
        Node {
            width: px(CLOSE_PIXELS),
            height: px(CLOSE_PIXELS),
            border: UiRect::all(px(BUTTON_BORDER)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BorderColor::from(INK),
    ));
    button.with_child((
        Text::new("X"),
        TextFont::from_font_size(CLOSE_FONT),
        TextColor(INK),
    ));
    button
}

/// Makes buttons look as their `ButtonState` says, when it changes.
fn show_button_states(
    mut buttons: Query<
        (
            &ButtonState,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        Changed<ButtonState>,
    >,
    mut labels: Query<&mut TextColor>,
) {
    for (state, mut background, mut border, children) in &mut buttons {
        let (new_background, new_border, new_text) = state.colours();
        background.0 = new_background;
        *border = new_border;
        for &child in children {
            if let Ok(mut text) = labels.get_mut(child) {
                text.0 = new_text;
            }
        }
    }
}
