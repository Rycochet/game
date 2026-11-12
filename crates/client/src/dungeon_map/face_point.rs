//! A place pointed at on the map (`common::PointedAtMap`: a right click or a
//! long press, where a menu of choices will open) turns the player towards
//! the ground there (`common::FaceTowards`).

use bevy::prelude::*;
use shared::spot::{CELL_SPAN, Spot};

use super::CELL_SIZE;
use super::ground_hit::Ground;
use crate::common::{DrawnGround, FaceTowards, GameView, PointedAtMap};
use crate::view3d::ViewCamera;

pub(super) fn face_pointed_at(
    view: Res<State<GameView>>,
    mut pointed: MessageReader<PointedAtMap>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    heights: Res<DrawnGround>,
    mut face: MessageWriter<FaceTowards>,
) {
    let (camera, camera_transform) = *camera;
    for PointedAtMap(at) in pointed.read() {
        if *view.get() != GameView::Dungeon {
            continue;
        }
        let Ok(ray) = camera.viewport_to_world(camera_transform, *at) else {
            continue;
        };
        if let Some((_, point, _)) = Ground(Some(&heights)).hit(ray) {
            let units = |world: f32| (world / CELL_SIZE * CELL_SPAN as f32).round() as i64;
            face.write(FaceTowards(Spot::new(units(point.x), units(point.z))));
        }
    }
}
