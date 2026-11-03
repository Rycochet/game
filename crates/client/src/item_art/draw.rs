//! Drawing the pictures asked for, a page at a time.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use std::time::Duration;

use super::copy::PageCopies;
use super::parts::parts_of;
use super::{
    Batch, EdgeMaterial, ItemPicture, ItemPictures, OUTLINE_PIXELS, PAGES, PictureLight,
    StillCompiling, spawn_batch, spawn_cell, spawn_lights,
};

/// How long building models may take in a frame, so a panel full of new things
/// doesn't stall one. What is built in a frame is drawn as one page.
const BUILD_TIME: Duration = Duration::from_millis(10);

pub(super) fn fill_pictures(
    mut pictures: ResMut<ItemPictures>,
    mut images: ResMut<Assets<Image>>,
    mut new: Query<(&ItemPicture, &mut ImageNode), Added<ItemPicture>>,
) {
    for (picture, mut node) in &mut new {
        node.image = pictures.picture(picture.look.clone(), picture.pixels, &mut images);
    }
}

type Stores<'w> = (
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
    ResMut<'w, Assets<EdgeMaterial>>,
    ResMut<'w, Assets<Image>>,
);

/// Finishes pages once drawn and copied, and starts one with as many waiting
/// pictures as can be built this frame, while there is a page free.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_pictures(
    mut commands: Commands,
    mut pictures: ResMut<ItemPictures>,
    mut stores: Stores,
    mut batches: Query<(Entity, &mut Batch)>,
    lights: Query<Entity, With<PictureLight>>,
    compiling: Res<StillCompiling>,
    mut copies: ResMut<PageCopies>,
) {
    let started = Instant::now();
    copies.0.clear();
    let mut busy = [false; PAGES];
    for (entity, mut batch) in &mut batches {
        busy[batch.slot] = true;
        if compiling.0 > 0 {
            continue;
        }
        batch.frames_left -= 1;
        if batch.frames_left == 0 {
            // Copied in the frames before.
            pictures.drawn += batch.cells.len() as u64;
            commands.entity(entity).despawn();
            continue;
        }
        let page = batch.page.id();
        copies.0.extend(
            batch
                .cells
                .iter()
                .map(|(rect, picture)| (page, *rect, picture.id())),
        );
    }
    let Some(first) = pictures.waiting.front() else {
        if batches.is_empty() {
            for light in &lights {
                commands.entity(light).despawn();
            }
        }
        return;
    };
    let Some(slot) = (0..PAGES).find(|&slot| !busy[slot]) else {
        return;
    };
    if lights.is_empty() {
        spawn_lights(&mut commands);
    }
    let size = first.look.picture_size(first.pixels);
    let mut batch = Batch::new(slot, size, &mut stores.3, &mut stores.0);
    let entity = commands.spawn_empty().id();
    fill(
        &mut commands,
        (entity, &mut batch),
        &mut pictures,
        &mut stores,
        started,
    );
    spawn_batch(&mut commands, entity, batch);
}

/// Takes waiting pictures onto the page while there is room and time this frame.
fn fill(
    commands: &mut Commands,
    (entity, batch): (Entity, &mut Batch),
    pictures: &mut ItemPictures,
    (meshes, materials, edges, _): &mut Stores,
    started: Instant,
) {
    while let Some(next) = pictures.waiting.front() {
        if started.elapsed() > BUILD_TIME && !batch.cells.is_empty() {
            return;
        }
        let Some(rect) = batch.shelves.place(next.look.picture_size(next.pixels)) else {
            return;
        };
        let Some(next) = pictures.waiting.pop_front() else {
            return;
        };
        let building = Instant::now();
        let parts = parts_of(next.look, meshes, materials);
        pictures.building += building.elapsed();
        // As thick on the screen whatever the detail.
        let reach = (OUTLINE_PIXELS * next.pixels as f32 / next.asked as f32).round() as u32;
        let edge = MeshMaterial3d(batch.edge(reach, edges));
        spawn_cell(commands, (entity, batch), rect, parts, edge);
        batch.cells.push((rect, next.picture));
    }
}
