//! Showing where what is held would go.

use bevy::prelude::*;
use shared::item::Rotation;

use super::super::hold::Target;
use super::super::{Carried, Holding};
use super::AimLayer;
use crate::widgets::{ALLOWED, REFUSED, at_cell};

/// Colours the cells a drop would take, green if it can go there and red if not.
pub(in crate::inventory) fn show_aim(
    carried: Res<Carried>,
    holding: Res<Holding>,
    layers: Query<(Entity, &AimLayer)>,
    mut commands: Commands,
    mut drawn: Local<Option<(Target, Rotation)>>,
) {
    let held = holding.held.map(|held| held.item);
    let aim = holding
        .aim
        .zip(held)
        .map(|(aim, held)| (aim, held.rotation));
    if *drawn == aim {
        return;
    }
    *drawn = aim;
    for (entity, _) in &layers {
        commands.entity(entity).despawn_related::<Children>();
    }
    let (Some((aim, rotation)), Some(_)) = (aim, held) else {
        return;
    };
    let Some(item) = holding.item(&carried.0) else {
        return;
    };
    let (holder, cells, allowed) = match aim {
        Target::Nowhere
        | Target::Bin
        | Target::Drop { .. }
        | Target::Body { .. }
        | Target::Someone(_) => return,
        Target::Cells {
            holder,
            x,
            y,
            allowed,
        } => {
            let shape = item.shape().turned(rotation);
            let cells: Vec<_> = shape
                .cells()
                .map(|(cell_x, cell_y)| (x + cell_x as i32, y + cell_y as i32))
                .collect();
            (holder, cells, allowed)
        }
        Target::Bag { bag, place, onto } => {
            let Some((holder, placed)) = carried.0.placed(bag) else {
                return;
            };
            let cells = placed.cells().map(|(x, y)| (x as i32, y as i32)).collect();
            (holder, cells, place.is_some() || onto.is_some())
        }
        Target::Stack { onto, allowed } => {
            let Some((holder, placed)) = carried.0.placed(onto) else {
                return;
            };
            let cells = placed.cells().map(|(x, y)| (x as i32, y as i32)).collect();
            (holder, cells, allowed)
        }
    };
    let Some((width, height)) = carried.0.grid_size(holder) else {
        return;
    };
    let tint = if allowed { ALLOWED } else { REFUSED };
    for (entity, layer) in &layers {
        if layer.holder != holder {
            continue;
        }
        commands.entity(entity).with_children(|layer| {
            for &(x, y) in &cells {
                if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
                    layer.spawn((at_cell(x as f32, y as f32), BackgroundColor(tint)));
                }
            }
        });
    }
}
