//! How items look anywhere in the interface: cells with edges, grids to put them
//! in, and the name beside the pointer. What pressing or dropping them does stays
//! with the module showing them.

use bevy::prelude::*;
use bevy::ui::{CalculatedClip, ComputedStackIndex, UiGlobalTransform};
use bevy::window::PrimaryWindow;
use shared::item::{Item, ItemKind, Quality, Rotation, Shape};

use super::aura::{Aura, AuraMask, aura_glow};
use super::set_edge::{SetEdge, set_edge};
use super::{GRID_COLOUR, GRID_LINE, ITEM_EDGE, Pressable, TextSize, spawn_tooltip};
use crate::common::{NO_QUALITY_NAME, UiPointer, interface_area, quality_colour};
use crate::item_art::{ItemPicture, Look, look_of};

/// A grid cell, across, in logical pixels.
pub const CELL: f32 = 32.0;
/// How many pixels a picture has for each cell it covers: twice what it is shown at,
/// for sharpness.
pub const PICTURE_PIXELS_PER_CELL: u32 = 64;
/// What is written on an item.
pub const ITEM_LABEL_FONT: TextSize = TextSize::Small;
/// Above moveable panels, below full-screen layers.
pub const HELD_ITEM_Z: i32 = 900_000;

const CELL_BORDER: f32 = 1.0;
/// How thick the metal edge of a set item's name is.
const NAME_EDGE: f32 = 3.0;
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
    /// Gets a metal edge.
    pub set: bool,
    /// Where its top left goes, in the interface's pixels (`UiPointer`'s).
    pub at: Vec2,
}

/// On an item's cells and label for its name to show when the pointer is on it,
/// for items whose owner has nothing more to say about them.
#[derive(Component, Clone)]
pub struct ItemName {
    pub words: String,
    pub quality: Option<Quality>,
    pub set: bool,
}

/// What an item looks like.
#[derive(Clone, Debug)]
pub struct ItemLook {
    pub kind: ItemKind,
    pub quality: Option<Quality>,
    pub set: bool,
    /// Its picture, if its kind has a model, and how it is turned.
    pub picture: Option<Look>,
    pub rotation: Rotation,
    /// How it glows, if it does.
    pub aura: Option<Aura>,
}

impl ItemLook {
    pub fn of(item: &Item) -> Self {
        Self {
            kind: item.kind,
            quality: item.quality,
            set: item.set,
            picture: look_of(item),
            rotation: Rotation::None,
            aura: Aura::of(item),
        }
    }

    /// Turned this way.
    pub fn turned(self, rotation: Rotation) -> Self {
        Self { rotation, ..self }
    }
}

#[derive(Component)]
pub(super) struct NameBox;

#[derive(Component)]
pub(super) struct NameText;

/// The metal edge of the box, for a set item's name.
#[derive(Component)]
pub(super) struct NameEdge;

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

/// Where a node is on the screen, in the interface's pixels (`UiPointer`'s, as
/// `Node`s are sized in): its top left and size.
pub fn logical_rect(node: &ComputedNode, transform: &UiGlobalTransform) -> (Vec2, Vec2) {
    let size = node.size() * node.inverse_scale_factor;
    let centre = transform.translation * node.inverse_scale_factor;
    (centre - size / 2.0, size)
}

/// Whether a node is at this point, in the interface's pixels (`UiPointer`'s), and
/// not scrolled out of sight.
pub fn shown_at(
    at: Vec2,
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    clip: Option<&CalculatedClip>,
) -> bool {
    let physical = at / node.inverse_scale_factor;
    node.contains_point(*transform, physical)
        && clip.is_none_or(|clip| clip.contains_point(physical))
}

/// An empty grid with `extra` on it, and `on_lines` on each cell's lines. `fill`
/// adds what goes over its cells.
pub fn spawn_item_grid<L: Bundle + Clone>(
    parent: &mut ChildSpawnerCommands,
    width: u8,
    height: u8,
    extra: impl Bundle,
    on_lines: L,
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
                    grid.spawn((
                        at_cell(x as f32, y as f32),
                        BorderColor::from(GRID_LINE),
                        on_lines.clone(),
                    ));
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
    look: ItemLook,
    shape: Shape,
    corner: (f32, f32),
    pressable: bool,
    on_each: impl Fn(u8, u8) -> B,
) {
    let ItemLook {
        kind,
        quality,
        set,
        picture,
        rotation,
        aura,
    } = look;
    let drawn = picture.is_some();
    let pixels = shape.width.max(shape.height) as u32 * PICTURE_PIXELS_PER_CELL;
    // Behind a picture, with its frame drawn over it.
    if let (Some(aura), Some(picture)) = (aura, &picture) {
        let node = picture_node(shape, rotation, corner);
        let mask = AuraMask::Picture(picture.clone(), pixels);
        parent.spawn((aura_glow(aura, mask, node), picture_turned(rotation)));
    }
    // Before the cells, so that they are what the pointer finds.
    if let Some(picture) = picture {
        let node = picture_node(shape, rotation, corner);
        parent.spawn(item_picture(picture, pixels, node, rotation));
    }
    let mut first = true;
    for (x, y) in shape.cells() {
        let edge = item_edge_width(quality);
        // A picture goes across its cells, so only the outside of the shape has an edge.
        let border = if drawn {
            cell_edges(shape, x, y, edge)
        } else {
            UiRect::all(px(edge))
        };
        let colour = if drawn {
            Color::NONE
        } else {
            item_colour(kind)
        };
        let mut node = parent.spawn((
            Node {
                border,
                ..at_cell(corner.0 + x as f32, corner.1 + y as f32)
            },
            BackgroundColor(colour),
            BorderColor::from(item_edge(quality)),
            on_each(x, y),
        ));
        if pressable {
            node.insert(Pressable);
        }
        if first && !drawn {
            node.with_child((
                Text::new(item_label(kind)),
                ITEM_LABEL_FONT.font(),
                TextColor(ITEM_EDGE),
                on_each(x, y),
            ));
        }
        first = false;
        if set {
            let width = item_edge_width(quality);
            node.with_child(set_edge(quality, width, width));
        }
    }
    // Over plain cells, inside their frame.
    if let (Some(aura), false) = (aura, drawn) {
        let edge = item_edge_width(quality);
        let node = Node {
            left: px(corner.0 * CELL + edge),
            top: px(corner.1 * CELL + edge),
            width: px(shape.width as f32 * CELL - edge * 2.0),
            height: px(shape.height as f32 * CELL - edge * 2.0),
            ..default()
        };
        parent.spawn(aura_glow(aura, AuraMask::Cells(shape), node));
    }
}

/// Where a picture goes over a shape as it lies, with the shape's corner at `corner`. A
/// node turns about its middle and keeps its own layout, so one turned a quarter is laid
/// out the other way round, with the same middle.
fn picture_node(shape: Shape, rotation: Rotation, corner: (f32, f32)) -> Node {
    let (lying_wide, lying_long) = (shape.width as f32, shape.height as f32);
    let (wide, long) = match rotation {
        Rotation::None | Rotation::Half => (lying_wide, lying_long),
        Rotation::Quarter | Rotation::ThreeQuarters => (lying_long, lying_wide),
    };
    Node {
        position_type: PositionType::Absolute,
        left: px((corner.0 + (lying_wide - wide) / 2.0) * CELL),
        top: px((corner.1 + (lying_long - long) / 2.0) * CELL),
        width: px(wide * CELL),
        height: px(long * CELL),
        ..default()
    }
}

/// A picture of this look kept to its proportions, in the middle of a square parent.
pub fn picture_in_square(look: &Look) -> Node {
    let (wide, long) = look.cells();
    let longest = look.longest() as f32;
    let (wide, long) = (wide as f32 / longest * 100.0, long as f32 / longest * 100.0);
    Node {
        position_type: PositionType::Absolute,
        left: percent((100.0 - wide) / 2.0),
        top: percent((100.0 - long) / 2.0),
        width: percent(wide),
        height: percent(long),
        ..default()
    }
}

/// An item's picture in this node, turned this way. It never takes the pointer.
pub fn item_picture(look: Look, pixels: u32, node: Node, rotation: Rotation) -> impl Bundle {
    (
        Floating,
        ItemPicture { look, pixels },
        ImageNode::default(),
        node,
        picture_turned(rotation),
    )
}

/// How something drawn from a picture is turned.
fn picture_turned(rotation: Rotation) -> UiTransform {
    let degrees = match rotation {
        Rotation::None => 0.0,
        Rotation::Quarter => 90.0,
        Rotation::Half => 180.0,
        Rotation::ThreeQuarters => 270.0,
    };
    UiTransform::from_rotation(Rot2::degrees(degrees))
}

/// An edge of this width only on the sides with none of the shape beyond them.
fn cell_edges(shape: Shape, x: u8, y: u8, edge: f32) -> UiRect {
    let side = |beyond: bool| if beyond { px(0) } else { px(edge) };
    UiRect {
        left: side(x > 0 && shape.has(x - 1, y)),
        right: side(shape.has(x + 1, y)),
        top: side(y > 0 && shape.has(x, y - 1)),
        bottom: side(shape.has(x, y + 1)),
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
        .with_children(|holder| {
            spawn_tooltip(holder, NameText, "");
            holder.spawn((NameEdge, set_edge(None, NAME_EDGE, 0.0), Visibility::Hidden));
        });
}

/// Asks for the name of the `ItemName` in front at the pointer, unless a name has
/// been asked for already.
#[allow(clippy::type_complexity)]
pub(super) fn name_item_at_pointer(
    pointer: Res<UiPointer>,
    nodes: Query<
        (
            &ComputedStackIndex,
            &ComputedNode,
            &UiGlobalTransform,
            &InheritedVisibility,
            Option<&ItemName>,
            Option<&CalculatedClip>,
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
        .filter(|(_, node, transform, visible, _, clip)| {
            visible.get() && shown_at(at, node, transform, *clip)
        })
        .max_by_key(|(stack, ..)| stack.0);
    if let Some((.., Some(name), _)) = front {
        tooltip.0 = Some(ItemTip {
            words: name.words.clone(),
            quality: name.quality,
            set: name.set,
            at: at + TOOLTIP_OFFSET,
        });
    }
}

#[allow(clippy::type_complexity)]
/// Shows the name asked for this frame, on a dark box so every colour can be read.
pub(super) fn show_item_tooltip(
    mut tooltip: ResMut<ItemTooltip>,
    window: Single<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    name_box: Single<(&mut Node, &mut Visibility, &ComputedNode), With<NameBox>>,
    text: Single<(&mut Text, &mut TextColor), With<NameText>>,
    edge: Single<(&mut SetEdge, &mut Visibility), (With<NameEdge>, Without<NameBox>)>,
) {
    let (mut node, mut visibility, computed) = name_box.into_inner();
    let Some(tip) = tooltip.0.take() else {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    };
    visibility.set_if_neq(Visibility::Inherited);
    let (mut edge, mut edge_visible) = edge.into_inner();
    edge.set_if_neq(SetEdge(tip.quality));
    edge_visible.set_if_neq(if tip.set {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    let (mut text, mut colour) = text.into_inner();
    if text.0 != tip.words {
        text.0 = tip.words;
    }
    colour.set_if_neq(TextColor(
        tip.quality.map_or(NO_QUALITY_NAME, quality_colour),
    ));
    // Kept on the screen, by the size it last had.
    let size = computed.size() * computed.inverse_scale_factor;
    let at = tip
        .at
        .min(interface_area(&window, &ui_scale) - size)
        .max(Vec2::ZERO);
    node.left = px(at.x);
    node.top = px(at.y);
}

#[cfg(test)]
mod tests;
