//! An item's picture, placed and turned over its shape.

use bevy::prelude::*;
use shared::item::{Rotation, Shape};

use super::{CELL, Floating};
use crate::item_art::{ItemPicture, Look};

/// Where a picture goes over a shape as it lies, with the shape's corner at `corner`. A
/// node turns about its middle and keeps its own layout, so one turned a quarter is laid
/// out the other way round, with the same middle.
pub(super) fn picture_node(shape: Shape, rotation: Rotation, corner: (f32, f32)) -> Node {
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
pub(super) fn picture_turned(rotation: Rotation) -> UiTransform {
    let degrees = match rotation {
        Rotation::None => 0.0,
        Rotation::Quarter => 90.0,
        Rotation::Half => 180.0,
        Rotation::ThreeQuarters => 270.0,
    };
    UiTransform::from_rotation(Rot2::degrees(degrees))
}
