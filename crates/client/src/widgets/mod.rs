//! The game's interface pieces, so they all look alike: panels with titles,
//! buttons, tooltips and panels that can be moved about. Use these rather than
//! styling Bevy's `Node`s directly. See `docs/context/client/widgets.md`.

mod moveable;

use bevy::input::InputSystems;
use bevy::prelude::*;

use crate::common::PointerSystems;

// (Only `dev_tools` makes one until the debug menu does, and it is native only.)
#[cfg_attr(target_arch = "wasm32", allow(unused_imports))]
pub use moveable::spawn_moveable_panel;

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
            .init_resource::<moveable::Dragging>()
            .add_systems(
                PreUpdate,
                moveable::take_presses
                    .in_set(WidgetSystems)
                    .after(InputSystems)
                    .after(PointerSystems),
            )
            .add_systems(Update, (show_button_states, moveable::place_panels));
    }
}

/// Where widgets take what they use of the mouse and fingers, in `PreUpdate`,
/// so nothing underneath them sees it. Anything else taking clicks there runs
/// after this.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetSystems;

/// A button marked `Pressable` was clicked or tapped, unless it was disabled.
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ButtonPressed(pub Entity);

/// Put this on a button (from `spawn_button`) for clicks on it to be sent as
/// `ButtonPressed` and go no further.
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
}

impl ButtonState {
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
pub fn spawn_tooltip(holder: &mut ChildSpawnerCommands, marker: impl Bundle) {
    holder.spawn(raised_box()).with_child((
        marker,
        Text::new(""),
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
