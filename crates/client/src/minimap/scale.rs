//! How much the maps show: grid units per pixel, under the sky, still showing something known.

use super::{CELL_PIXELS, Room, minimap_across};
use crate::common::{CurrentDungeon, CurrentWorld, GameView};
use crate::view3d::OrbitCamera;
use crate::world_map::{self};

/// Whether the sky can be seen here, so the minimap shows what is in it.
pub(super) fn under_the_sky(view: GameView, dungeon: &CurrentDungeon) -> bool {
    view != GameView::Dungeon || dungeon.knows_outside()
}

/// Grid units per logical pixel at the minimap's scale. On the world map the minimap shows as much as
/// the window on screen does across, which zooming changes.
pub(super) fn units_per_pixel(view: GameView, window: &Room, camera: &OrbitCamera) -> f64 {
    match view {
        GameView::WorldMap => {
            let shown = 2.0 * (world_map::HALF_SIZE as f64) * f64::from(camera.zoom());
            shown / f64::from(minimap_across(window))
        }
        _ => 1.0 / f64::from(CELL_PIXELS),
    }
}

/// Whether a full map centred somewhere still shows something known.
pub(super) fn shows_known(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    centre: (f64, f64),
    half: (f64, f64),
) -> bool {
    let min = (centre.0 - half.0, centre.1 - half.1);
    let max = (centre.0 + half.0, centre.1 + half.1);
    match view {
        GameView::WorldMap => {
            world.is_some_and(|world| super::world::shows_known(world, (min, max)))
        }
        _ => {
            let cell = |at: f64| (at + 0.5).floor() as i64;
            (cell(min.1)..=cell(max.1))
                .any(|y| (cell(min.0)..=cell(max.0)).any(|x| dungeon.remembered.is_seen(x, y)))
        }
    }
}
