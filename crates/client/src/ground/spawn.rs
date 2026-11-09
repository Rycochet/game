//! Making a bag's parts, and keeping the list up to date.

use bevy::prelude::*;
use shared::ground::Load;

use super::bag::size_of;
use super::{
    Bag, BagAssets, BagPart, Bags, GLINT_SCALE, OfBag, Sparkle, Star, sparkle_scale, sparkle_up,
};
use crate::dungeon_map::CELL_SIZE;

/// A number from a cell, the same each time, for what differs from bag to bag.
pub(super) fn cell_hash(cell: (i64, i64)) -> u32 {
    let mixed = (cell.0 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (cell.1 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    (mixed >> 32) as u32
}

pub(super) fn spawn_bag(
    commands: &mut Commands,
    assets: &BagAssets,
    cell: (i64, i64),
    load: Load,
) -> Entity {
    let hash = cell_hash(cell);
    let variety = (hash % super::bag::VARIETIES as u32) as u8;
    let meshes = &assets.bags[&(load, variety)];
    let size = size_of(load);
    let turned = Quat::from_rotation_y((hash >> 8) as f32 % 8.0 * std::f32::consts::FRAC_PI_4);
    let of_bag = OfBag(cell);
    let place = Transform::from_xyz(cell.0 as f32 * CELL_SIZE, 0.0, cell.1 as f32 * CELL_SIZE);
    let body = Transform::from_rotation(turned);
    let phase = (hash >> 16) as f32 % 100.0;
    let root = commands.spawn((place, Visibility::Inherited)).id();
    let hidden = Visibility::Hidden;
    let parts = [
        commands
            .spawn((
                BagPart::Lit,
                of_bag,
                Mesh3d(meshes.lit.clone()),
                MeshMaterial3d(assets.lit.clone()),
                body,
                hidden,
            ))
            .id(),
        commands
            .spawn((
                BagPart::Remembered,
                of_bag,
                Mesh3d(meshes.remembered.clone()),
                MeshMaterial3d(assets.remembered.clone()),
                body,
                hidden,
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(meshes.outline.clone()),
                MeshMaterial3d(assets.outline.clone()),
                body,
            ))
            .id(),
        // Seen through whatever hides it, whether in sight or remembered.
        commands
            .spawn((
                Mesh3d(meshes.lit.clone()),
                MeshMaterial3d(assets.xray_fill.clone()),
                body,
            ))
            .id(),
        commands
            .spawn((
                Mesh3d(meshes.outline.clone()),
                MeshMaterial3d(assets.xray_outline.clone()),
                body,
            ))
            .id(),
        commands
            .spawn((
                BagPart::HoverOutline,
                of_bag,
                Mesh3d(meshes.hover_outline.clone()),
                MeshMaterial3d(assets.outline.clone()),
                body,
                hidden,
            ))
            .id(),
    ];
    let sparkle = commands
        .spawn((
            Sparkle {
                cell,
                up: sparkle_up(load),
                behind: size.radius,
                scale: sparkle_scale(load),
                phase,
            },
            of_bag,
            Transform::default(),
            hidden,
        ))
        .id();
    // The main star, and glints off to each side of it.
    for (scale, offset, star_phase) in [
        (1.0, Vec3::ZERO, 0.0),
        (GLINT_SCALE, Vec3::new(5.5, 4.5, 0.2), 1.9),
        (GLINT_SCALE * 0.8, Vec3::new(-6.0, 2.5, 0.2), 3.7),
    ] {
        let star = commands
            .spawn((
                Star {
                    scale,
                    phase: star_phase,
                },
                Mesh3d(assets.star.clone()),
                MeshMaterial3d(assets.sparkle.clone()),
                Transform::from_translation(offset).with_scale(Vec3::splat(scale)),
                ChildOf(sparkle),
            ))
            .id();
        // Just behind it.
        commands.spawn((
            Mesh3d(meshes.star_outline.clone()),
            MeshMaterial3d(assets.star_edge.clone()),
            Transform::from_xyz(0.0, 0.0, -0.5),
            ChildOf(star),
        ));
        commands.spawn((
            BagPart::SparkleHoverOutline,
            of_bag,
            Mesh3d(meshes.star_hover_outline.clone()),
            MeshMaterial3d(assets.star_edge.clone()),
            Transform::from_xyz(0.0, 0.0, -0.6),
            hidden,
            ChildOf(star),
        ));
    }
    commands
        .entity(root)
        .add_children(&parts)
        .add_child(sparkle);
    root
}

/// Puts a bag in the list, or changes how it looks.
pub(super) fn set_bag(
    bags: &mut Bags,
    commands: &mut Commands,
    assets: &BagAssets,
    cell: (i64, i64),
    load: Load,
) {
    if let Some(old) = bags.0.get(&cell) {
        if old.load == load {
            return;
        }
        commands.entity(old.root).despawn();
    }
    let root = spawn_bag(commands, assets, cell, load);
    bags.0.insert(cell, Bag { load, root });
}
