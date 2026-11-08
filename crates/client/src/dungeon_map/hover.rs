//! Naming the door or stairs the pointer is over, for the hover box.

use bevy::prelude::*;
use shared::site::StairsDirection;

use super::Shown;
use super::ground_hit::Ground;
use super::picking::{door_clicked, pick};
use crate::common::{CurrentDungeon, GameView, HoveredOnMap, PointerPosition, WildCellHeights};
use crate::view3d::ViewCamera;

/// What the hatched border round a poi in the wilderness is called.
const BORDER_NAME: &str = "Way back to the world map";

/// Names what the pointer is over on the site's map: a building's door (a
/// shop's by what it sells), the border round a poi, or stairs. Nothing elsewhere, or off the site.
pub(super) fn name_hovered_place(
    view: Res<State<GameView>>,
    pointer: Res<PointerPosition>,
    camera: Single<(&Camera, &GlobalTransform), With<ViewCamera>>,
    dungeon: Res<CurrentDungeon>,
    shown: Res<Shown>,
    heights: Res<WildCellHeights>,
    mut hovered: ResMut<HoveredOnMap>,
) {
    let (camera, camera_transform) = *camera;
    let ground = Ground(Some(&heights));
    let name = pointer
        .0
        .filter(|_| *view.get() == GameView::Dungeon)
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor).ok())
        .and_then(|ray| {
            let door = door_clicked(ray, &dungeon.buildings, &shown.solid_faces, ground);
            if let Some(id) = door {
                let building = dungeon.buildings.iter().find(|b| b.id == id)?;
                return Some(building.kind.name().to_owned());
            }
            let (x, y) = pick(
                ray,
                &dungeon.level,
                &dungeon.remembered,
                &shown.solid_faces,
                ground,
            )?;
            if dungeon.borders.contains(&(x, y)) {
                return Some(BORDER_NAME.to_owned());
            }
            let stairs = dungeon.stairs.iter().find(|s| (s.x, s.y) == (x, y))?;
            Some(stairs_name(stairs.direction).to_owned())
        });
    if hovered.0 != name {
        hovered.0 = name;
    }
}

/// What stairs are called, by the way they go.
fn stairs_name(direction: StairsDirection) -> &'static str {
    match direction {
        StairsDirection::Up => "Stairs up",
        StairsDirection::Down => "Stairs down",
    }
}
