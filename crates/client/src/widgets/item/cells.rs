//! Spawning item grids and an item's cells.

use bevy::prelude::*;
use shared::item::Shape;

use super::super::aura::{AuraMask, aura_glow};
use super::super::set_edge::set_edge;
use super::super::{ITEM_EDGE, Pressable};
use super::count::spawn_count;
use super::mixed::spawn_mixed_mark;
use super::{
    CELL, ITEM_LABEL_FONT, ItemLook, PICTURE_PIXELS_PER_CELL, at_cell, item_colour, item_edge,
    item_edge_width, item_label, item_picture, picture_node, picture_turned,
};

/// Over an item filtered out: `GRID_COLOUR`, letting a little of it through.
const DIMMING: Color = Color::srgba(0.55, 0.46, 0.33, 0.65);

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
        count,
        mixed,
        dimmed,
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
            // The same sides as its own edge, so a picture's goes round the whole shape.
            node.with_child(set_edge(quality, border, edge));
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
    spawn_count(parent, count, shape, corner, item_edge_width(quality));
    spawn_mixed_mark(parent, mixed, shape, corner, item_edge_width(quality));
    if dimmed {
        // Last, so over all of it: the grid's colour, mostly hiding it.
        for (x, y) in shape.cells() {
            let cell = at_cell(corner.0 + x as f32, corner.1 + y as f32);
            parent.spawn((cell, BackgroundColor(DIMMING)));
        }
    }
}

/// An edge of this width only on the sides with none of the shape beyond them.
pub(super) fn cell_edges(shape: Shape, x: u8, y: u8, edge: f32) -> UiRect {
    let side = |beyond: bool| if beyond { px(0) } else { px(edge) };
    UiRect {
        left: side(x > 0 && shape.has(x - 1, y)),
        right: side(shape.has(x + 1, y)),
        top: side(y > 0 && shape.has(x, y - 1)),
        bottom: side(shape.has(x, y + 1)),
    }
}
