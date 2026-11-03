//! Drawing the pictures asked for, a few at a time.

use bevy::prelude::*;

use super::{
    Drawing, ItemPicture, ItemPictures, Look, Part, PictureLight, SLOTS, StillCompiling,
    VIEW_WIDTH, outline_copies, spawn_drawing, spawn_lights,
};

pub(super) fn fill_pictures(
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut new: Query<(&ItemPicture, &mut ImageNode), Added<ItemPicture>>,
) {
    for (picture, mut node) in &mut new {
        node.image = pictures.picture(picture.look.clone(), picture.pixels, &mut images);
    }
}

pub(super) fn draw_pictures(
    mut commands: Commands,
    mut pictures: ResMut<ItemPictures>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut drawing: Query<(Entity, &mut Drawing)>,
    lights: Query<Entity, With<PictureLight>>,
    compiling: Res<StillCompiling>,
) {
    let mut free = [true; SLOTS];
    for (entity, mut one) in &mut drawing {
        if one.frames_left == 0 {
            commands.entity(entity).despawn();
        } else {
            if compiling.0 == 0 {
                one.frames_left -= 1;
            }
            free[one.slot] = false;
        }
    }
    let any_left = free.iter().any(|&slot| !slot);
    if !any_left && pictures.waiting.is_empty() {
        for light in &lights {
            commands.entity(light).despawn();
        }
        return;
    }
    if lights.is_empty() {
        spawn_lights(&mut commands);
    }
    for slot in (0..SLOTS).filter(|&slot| free[slot]) {
        let Some((look, asked, picture)) = pictures.waiting.pop_front() else {
            break;
        };
        let (wide, long) = look.cells();
        let longest = look.longest() as f32;
        let view = Vec2::new(wide as f32, long as f32) / longest * VIEW_WIDTH;
        let parts = match look {
            Look::Gem { gem, quality } => {
                super::gem::gem_parts(&gem, quality, &mut meshes, &mut materials)
                    .into_iter()
                    .map(|(mesh, material)| Part {
                        mesh,
                        material,
                        transform: Transform::default(),
                    })
                    .collect()
            }
            Look::Ring { ring, quality } => {
                super::ring::ring_parts(&ring, quality, &mut meshes, &mut materials)
            }
            Look::Blade { blade, quality } => {
                super::blade::blade_parts(&blade, quality, &mut meshes, &mut materials)
            }
            Look::Body { kind } => super::body::body_parts(kind, &mut meshes, &mut materials),
            look => super::armour::armour_parts(look, &mut meshes, &mut materials),
        };
        let black = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            cull_mode: None,
            ..default()
        });
        // As thick on the screen whatever the detail.
        let outline = outline_copies(&parts, asked, &black);
        spawn_drawing(&mut commands, slot, picture, view, parts, outline);
    }
}
