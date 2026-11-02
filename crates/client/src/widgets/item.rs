//! How items look anywhere in the interface: cells with edges, grids to put them
//! in, and the name beside the pointer. What pressing or dropping them does stays
//! with the module showing them.

use bevy::prelude::*;
use bevy::ui::{ComputedStackIndex, UiGlobalTransform};
use shared::item::{ItemKind, Quality, Shape};

use super::{Pressable, spawn_tooltip};
use crate::common::{ITEM_NAME_BACKGROUND, NO_QUALITY_NAME, PointerPosition, quality_colour};

/// A grid cell, across, in logical pixels.
pub const CELL: f32 = 32.0;
/// What is written on an item.
pub const ITEM_LABEL_FONT: f32 = 13.0;
/// Above moveable panels, below full-screen layers.
pub const HELD_ITEM_Z: i32 = 900_000;

pub const GRID_COLOUR: Color = Color::srgb(0.55, 0.46, 0.33);
pub const GRID_LINE: Color = Color::srgb(0.42, 0.34, 0.24);
pub const ITEM_EDGE: Color = Color::srgb(0.15, 0.1, 0.05);
/// Cells a drop could take.
pub const ALLOWED: Color = Color::srgba(0.2, 0.8, 0.25, 0.55);
/// Cells a drop couldn't take.
pub const REFUSED: Color = Color::srgba(0.85, 0.15, 0.15, 0.55);

const CELL_BORDER: f32 = 1.0;
/// How far from the pointer an item's name goes.
const TOOLTIP_OFFSET: Vec2 = Vec2::new(16.0, 16.0);
/// How thick an edge showing quality is, to be seen beside the kind's colour.
const QUALITY_BORDER: f32 = 2.0;

/// Follows the pointer above everything: not what the pointer is on.
#[derive(Component, Clone, Copy)]
pub struct Floating;

/// The name to show this frame, if any. Whoever knows what the pointer is on sets
/// it in `Update`, and it is cleared once shown.
#[derive(Resource, Default)]
pub struct ItemTooltip(pub Option<ItemTip>);

pub struct ItemTip {
    pub words: String,
    pub quality: Option<Quality>,
    /// Where its top left goes.
    pub at: Vec2,
}

/// On an item's cells and label for its name to show when the pointer is on it,
/// for items whose owner has nothing more to say about them.
#[derive(Component, Clone)]
pub struct ItemName {
    pub words: String,
    pub quality: Option<Quality>,
}

#[derive(Component)]
pub(super) struct NameBox;

#[derive(Component)]
pub(super) struct NameText;

/// What a kind of item looks like until it has a picture: its colour.
pub fn item_colour(kind: ItemKind) -> Color {
    let index = ItemKind::ALL
        .iter()
        .position(|&other| other == kind)
        .unwrap_or(0);
    // Spread round the colour wheel by a step that doesn't repeat soon.
    Color::hsl((index as f32 * 67.0) % 360.0, 0.45, 0.5)
}

/// What it says on the first cell of an item.
pub fn item_label(kind: ItemKind) -> String {
    kind.def().name.chars().take(2).collect()
}

/// The colour of the edge of an item of this quality.
pub fn item_edge(quality: Option<Quality>) -> Color {
    quality.map_or(ITEM_EDGE, quality_colour)
}

/// How thick the edge of an item of this quality is.
pub fn item_edge_width(quality: Option<Quality>) -> f32 {
    if quality.is_some() {
        QUALITY_BORDER
    } else {
        CELL_BORDER
    }
}

/// A node one cell across at this place in a grid.
pub fn at_cell(x: f32, y: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x * CELL),
        top: px(y * CELL),
        width: px(CELL),
        height: px(CELL),
        border: UiRect::all(px(CELL_BORDER)),
        ..default()
    }
}

/// A node filling a grid, for its children to be placed in.
pub fn covering() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

/// An empty grid with `extra` on it. `fill` adds what goes over its cells.
pub fn spawn_item_grid(
    parent: &mut ChildSpawnerCommands,
    width: u8,
    height: u8,
    extra: impl Bundle,
    fill: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn((
            extra,
            Node {
                width: px(width as f32 * CELL),
                height: px(height as f32 * CELL),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(GRID_COLOUR),
        ))
        .with_children(|grid| {
            for y in 0..height {
                for x in 0..width {
                    grid.spawn((at_cell(x as f32, y as f32), BorderColor::from(GRID_LINE)));
                }
            }
            fill(grid);
        });
}

/// An item's cells, in a parent placing them, with the shape's corner at `corner`.
/// `on_each` gives what goes on a cell, and on the label if it is the first, from
/// its place in the shape. `pressable` makes presses on the cells `ButtonPressed`.
pub fn spawn_item_cells<B: Bundle>(
    parent: &mut ChildSpawnerCommands,
    kind: ItemKind,
    quality: Option<Quality>,
    shape: Shape,
    corner: (f32, f32),
    pressable: bool,
    on_each: impl Fn(u8, u8) -> B,
) {
    let mut first = true;
    for (x, y) in shape.cells() {
        let mut node = parent.spawn((
            Node {
                border: UiRect::all(px(item_edge_width(quality))),
                ..at_cell(corner.0 + x as f32, corner.1 + y as f32)
            },
            BackgroundColor(item_colour(kind)),
            BorderColor::from(item_edge(quality)),
            on_each(x, y),
        ));
        if pressable {
            node.insert(Pressable);
        }
        if first {
            first = false;
            node.with_child((
                Text::new(item_label(kind)),
                TextFont::from_font_size(ITEM_LABEL_FONT),
                TextColor(ITEM_EDGE),
                on_each(x, y),
            ));
        }
    }
}

pub(super) fn spawn_name_box(mut commands: Commands) {
    commands
        .spawn((
            NameBox,
            Floating,
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            GlobalZIndex(HELD_ITEM_Z + 1),
            Visibility::Hidden,
        ))
        .with_children(|holder| spawn_tooltip(holder, NameText, ""));
}

/// Asks for the name of the `ItemName` in front at the pointer, unless a name has
/// been asked for already.
#[allow(clippy::type_complexity)]
pub(super) fn name_item_at_pointer(
    pointer: Res<PointerPosition>,
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&ItemName>,
        ),
        (With<Node>, Without<Floating>),
    >,
    mut tooltip: ResMut<ItemTooltip>,
) {
    let Some(at) = pointer.0.filter(|_| tooltip.0.is_none()) else {
        return;
    };
    let front = nodes
        .iter()
        .filter(|(_, node, transform, visible, _)| {
            visible.get() && node.contains_point(**transform, at / node.inverse_scale_factor)
        })
        .max_by_key(|(stack, ..)| stack.0);
    if let Some((.., Some(name))) = front {
        tooltip.0 = Some(ItemTip {
            words: name.words.clone(),
            quality: name.quality,
            at: at + TOOLTIP_OFFSET,
        });
    }
}

/// Shows the name asked for this frame, on a dark box so every colour can be read.
pub(super) fn show_item_tooltip(
    mut tooltip: ResMut<ItemTooltip>,
    name_box: Single<(&mut Node, &mut Visibility), With<NameBox>>,
    text: Single<(&mut Text, &mut TextColor, &ChildOf), With<NameText>>,
    mut backgrounds: Query<&mut BackgroundColor>,
) {
    let (mut node, mut visibility) = name_box.into_inner();
    let Some(tip) = tooltip.0.take() else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let (mut text, mut colour, parent) = text.into_inner();
    if text.0 != tip.words {
        text.0 = tip.words;
    }
    colour.set_if_neq(TextColor(
        tip.quality.map_or(NO_QUALITY_NAME, quality_colour),
    ));
    if let Ok(mut background) = backgrounds.get_mut(parent.parent()) {
        background.set_if_neq(BackgroundColor(ITEM_NAME_BACKGROUND));
    }
    node.left = px(tip.at.x);
    node.top = px(tip.at.y);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_makes_an_edge_wider() {
        assert!(item_edge_width(Some(Quality::Poor)) > item_edge_width(None));
    }

    /// Bevy only finds two parameters clashing when a system starts.
    #[test]
    fn showing_a_name_asks_for_nothing_twice() {
        IntoSystem::into_system(show_item_tooltip).initialize(&mut World::new());
        IntoSystem::into_system(name_item_at_pointer).initialize(&mut World::new());
    }
}
