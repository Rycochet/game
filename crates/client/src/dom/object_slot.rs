//! `object-slot`: a place holding one thing, which draws itself. Whoever owns
//! it says what is in it (`SlotShown`); this draws the slot, the thing as one
//! cell whatever its shape, and the tint saying what letting go here would
//! do. What a press or a drop there does is still its owner's, which finds
//! the thing pressed by its `ThingCell`.
//!
//! Any node with a `SlotShown` is drawn as one: the element in markup, or a
//! node a screen not yet moved to markup spawns.

use bevy::prelude::*;

use super::Solid;
use shared::item::{Item, Rotation, Shape};

use super::object_grid::ThingCell;
use crate::item_art::{ItemPicture, look_of};
use crate::widgets::{
    ALLOWED, Aura, AuraMask, CELL, GRID_COLOUR, GRID_LINE, ITEM_EDGE, ITEM_LABEL_FONT,
    PICTURE_PIXELS_PER_CELL, Pressable, REFUSED, aura_glow, item_colour, item_edge,
    item_edge_width, item_label, item_picture, picture_in_square, set_edge,
};

/// How faded something under something else shows.
const FADED_ALPHA: f32 = 0.45;

/// How a slot shows what is in it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SlotLook {
    /// As it is in a grid, in one cell: its quality's edge, glow and set
    /// edge, its picture filling the slot.
    #[default]
    Worn,
    /// Only kept here (the bin): its picture whole inside the slot's frame.
    Kept,
}

/// The thing in a slot.
#[derive(Clone, Debug, PartialEq)]
pub struct SlotThing {
    pub item: Item,
    /// How many are here, this one on top: said after its letters if more
    /// than one.
    pub count: usize,
    /// Faded, as under something else.
    pub faded: bool,
    /// A press on it is a `ButtonPressed`: it can be picked up.
    pub pressable: bool,
}

/// What a slot shows, set by its owner. It is drawn again when this changes.
#[derive(Component, Clone, Debug, Default, PartialEq)]
#[require(Node, Solid)]
pub struct SlotShown {
    pub look: SlotLook,
    /// The picture of an empty slot saying what it is for.
    pub icon: Option<Handle<Image>>,
    pub thing: Option<SlotThing>,
    /// Something held is over it: green if it may go here, red if not.
    pub aim: Option<bool>,
}

/// What a slot was last drawn as.
#[derive(Component)]
pub(super) struct SlotDrawn(SlotShown);

/// Draws each slot again as far as what it shows has changed: its frame and
/// tint always, the thing in it only when that is another.
pub(super) fn draw_object_slots(
    mut commands: Commands,
    changed: Query<(Entity, &SlotShown, Option<&SlotDrawn>), Changed<SlotShown>>,
) {
    for (slot, shown, drawn) in &changed {
        let worn = shown.thing.is_some() && shown.look == SlotLook::Worn;
        let tint = match shown.aim {
            Some(true) => ALLOWED,
            Some(false) => REFUSED,
            None if shown.icon.is_some() || worn => Color::NONE,
            None => GRID_COLOUR,
        };
        // Something worn takes the whole slot, with its own edge.
        let border = if worn { 0.0 } else { 1.0 };
        let frame = Node {
            width: px(CELL),
            height: px(CELL),
            border: UiRect::all(px(border)),
            ..default()
        };
        let mut entity = commands.entity(slot);
        entity.insert((frame, BackgroundColor(tint), BorderColor::from(GRID_LINE)));
        match &shown.icon {
            Some(icon) => entity.insert(ImageNode::new(icon.clone())),
            None => entity.remove::<ImageNode>(),
        };
        let same = drawn
            .is_some_and(|SlotDrawn(was)| (was.look, &was.thing) == (shown.look, &shown.thing));
        if !same {
            entity.despawn_related::<Children>();
            if let Some(thing) = &shown.thing {
                entity.with_children(|inside| match shown.look {
                    SlotLook::Worn => spawn_worn(inside, slot, thing),
                    SlotLook::Kept => spawn_kept(inside, slot, thing),
                });
            }
        }
        entity.insert(SlotDrawn(shown.clone()));
    }
}

/// The cell of a thing in a slot: held by a cell it has, and never turned.
fn cell_of(slot: Entity, item: &Item) -> ThingCell {
    ThingCell {
        grid: slot,
        item: item.id,
        rotation: Rotation::None,
        grip: item.shape().cells().next().unwrap_or((0, 0)),
    }
}

fn filling() -> Node {
    Node {
        width: percent(100),
        height: percent(100),
        ..default()
    }
}

fn spawn_worn(inside: &mut ChildSpawnerCommands, slot: Entity, thing: &SlotThing) {
    let item = &thing.item;
    let alpha = if thing.faded { FADED_ALPHA } else { 1.0 };
    let picture = look_of(item);
    let tint = match picture {
        Some(_) => Color::NONE,
        None => item_colour(item.kind).with_alpha(alpha),
    };
    let cell = cell_of(slot, item);
    // A picture says what it is, leaving only how many.
    let label = match picture {
        Some(_) => String::new(),
        None => item_label(item.kind),
    };
    let text = match thing.count {
        0 | 1 => label,
        many => format!("{label}{many}"),
    };
    let width = item_edge_width(item.quality);
    let node = Node {
        border: UiRect::all(px(width)),
        ..filling()
    };
    let edge = BorderColor::from(item_edge(item.quality));
    let mut worn = inside.spawn((cell, node, BackgroundColor(tint), edge));
    if thing.pressable {
        worn.insert(Pressable);
    }
    if let Some(aura) = Aura::of(item) {
        // One cell here, whatever it takes up in a grid.
        let mask = match &picture {
            Some(look) => AuraMask::Picture(look.clone(), PICTURE_PIXELS_PER_CELL),
            None => AuraMask::Cells(Shape::rectangle(1, 1)),
        };
        worn.with_child(aura_glow(aura, mask, filling()));
    }
    if let Some(look) = picture {
        let node = Node {
            position_type: PositionType::Absolute,
            ..filling()
        };
        let pixels = PICTURE_PIXELS_PER_CELL;
        let image = ImageNode::default().with_color(Color::WHITE.with_alpha(alpha));
        worn.with_child((ItemPicture { look, pixels }, image, node));
    }
    if item.set {
        worn.with_child(set_edge(item.quality, UiRect::all(px(width)), width));
    }
    let words = (
        Text::new(text),
        ITEM_LABEL_FONT.font(),
        TextColor(ITEM_EDGE),
    );
    worn.with_child((cell, words));
}

fn spawn_kept(inside: &mut ChildSpawnerCommands, slot: Entity, thing: &SlotThing) {
    let item = &thing.item;
    let picture = look_of(item);
    let colour = match picture {
        Some(_) => Color::NONE,
        None => item_colour(item.kind),
    };
    let cell = cell_of(slot, item);
    let mut kept = inside.spawn((cell, filling(), BackgroundColor(colour)));
    if thing.pressable {
        kept.insert(Pressable);
    }
    match picture {
        // Any shape shows whole in the one cell.
        Some(look) => {
            let node = picture_in_square(&look);
            let pixels = look.longest() as u32 * PICTURE_PIXELS_PER_CELL;
            kept.with_child(item_picture(look, pixels, node, Rotation::None));
        }
        None => {
            let words = (
                Text::new(item_label(item.kind)),
                ITEM_LABEL_FONT.font(),
                TextColor(ITEM_EDGE),
            );
            kept.with_child((cell, words));
        }
    }
}

#[cfg(test)]
mod tests;
