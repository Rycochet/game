//! What pois, shop doors and places are called.

use bevy::prelude::*;
use shared::protocol::SeenPoi;

pub(super) use crate::common::{poi_name, words};

use super::grid::CellLook;
use crate::common::{CurrentDungeon, CurrentWorld, FirstFloor, GameView, SmoothPosition};

/// The name of a shop door or world map poi at a grid position.
/// `icon_reach` is how far a poi's picture reaches, in logical pixels.
pub(super) fn hovered_name(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    (x, y): (f64, f64),
    icon_reach: f32,
    units_per_pixel: f64,
) -> Option<String> {
    if view != GameView::WorldMap {
        let (cell_x, cell_y) = ((x + 0.5).floor() as i64, (y + 0.5).floor() as i64);
        return match super::grid::look_of(dungeon, cell_x, cell_y) {
            CellLook::Sign(kind) => Some(words(&format!("{kind:?}"))),
            _ => None,
        };
    }
    hovered_poi(world, (x, y), icon_reach, units_per_pixel).map(|poi| poi_name(&poi))
}

/// The world map poi whose picture is at a grid position, the nearest if several.
pub(super) fn hovered_poi(
    world: Option<&CurrentWorld>,
    (x, y): (f64, f64),
    icon_reach: f32,
    units_per_pixel: f64,
) -> Option<SeenPoi> {
    let reach = f64::from(icon_reach) * units_per_pixel;
    let min = ((x - reach).floor() as i64, (y - reach).floor() as i64);
    let max = ((x + reach).ceil() as i64, (y + reach).ceil() as i64);
    let distance = |poi: &SeenPoi| (poi.x as f64 - x).hypot(poi.y as f64 - y);
    world?
        .pois
        .in_area(min, max)
        .into_iter()
        .filter(|poi| (poi.x as f64 - x).abs().max((poi.y as f64 - y).abs()) <= reach)
        .min_by(|a, b| distance(a).total_cmp(&distance(b)))
}

/// The full map's title: the place, with the floor or level being looked at,
/// numbered as its button is.
pub(super) fn location_title(
    view: GameView,
    world: Option<&CurrentWorld>,
    dungeon: &CurrentDungeon,
    first_floor: FirstFloor,
    player: &SmoothPosition,
) -> String {
    match (view, world) {
        (GameView::WorldMap, Some(world)) => {
            let (x, y) = (player.x.round() as i64, player.y.round() as i64);
            let height = world.surface.sample(x, y).height;
            let biome = world.climate.sample(x, y, height).biome();
            format!("World map: {}", words(&format!("{biome:?}")))
        }
        // A proper name already: not one written in code style.
        _ if dungeon
            .info
            .as_ref()
            .is_some_and(|info| info.kind.poi().is_none()) =>
        {
            dungeon.floor_name(first_floor)
        }
        _ => words(&dungeon.floor_name(first_floor)),
    }
}
