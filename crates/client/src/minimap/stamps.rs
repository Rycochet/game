//! Pictures stamped on a map: shop signs and pois.

use bevy::prelude::*;
use std::collections::BTreeSet;

use super::grid::Looks;
use super::projection::Projection;
use super::sprites::Part;
use super::sprites::Sprites;
use super::{COMPLETED_OPACITY, ICON_PIXELS, SIGN_PIXELS};
use crate::common::{CurrentWorld, Portal, Token};
use crate::dungeon_map::{TOWN_ATLAS, door_picture};
use crate::world_map::{poi_picture, poi_picture_name};

/// A picture to stamp on a map: its path, the part of it, its size in pixels,
/// where it goes and how opaque it is.
pub(super) type Stamp = (String, Part, u32, (f32, f32), f32);

/// Gets ready what is stamped on a map: shop signs in a site, pois on the
/// world map. Also says whether any picture is still loading.
pub(super) fn prepare_stamps(
    looks: Option<&Looks>,
    world: Option<&CurrentWorld>,
    pois: &Query<(&Portal, &Token)>,
    projection: &Projection,
    grid_box: ((f64, f64), (f64, f64)),
    (zoom, dots): (f32, f32),
    (sprites, asset_server, images): (&mut ResMut<Sprites>, &AssetServer, &Assets<Image>),
) -> (Vec<Stamp>, bool) {
    let mut stamps = Vec::new();
    let mut busy = false;
    let mut prepare = |path: String, part, pixels: f32, at, opacity| {
        let pixels = (pixels * dots).round().max(1.0) as u32;
        if sprites.prepare(asset_server, images, &path, part, pixels) {
            stamps.push((path, part, pixels, at, opacity));
            false
        } else {
            true
        }
    };
    match (&looks, world) {
        (Some(looks), _) => {
            for ((x, y), kind) in looks.signs() {
                let at = projection.to_picture(x as f64, y as f64);
                let part = Some(door_picture(kind));
                busy |= prepare(TOWN_ATLAS.to_string(), part, SIGN_PIXELS * zoom, at, 1.0);
            }
        }
        (None, Some(world)) => {
            let width = world.surface.width();
            let completed: BTreeSet<_> = pois
                .iter()
                .filter(|(_, token)| token.completed)
                .filter_map(|(portal, _)| portal.poi())
                .map(|poi| (poi.x.rem_euclid(width), poi.y))
                .collect();
            let ((min_x, min_y), (max_x, max_y)) = grid_box;
            let area = (
                (min_x.floor() as i64, min_y.floor() as i64),
                (max_x.ceil() as i64, max_y.ceil() as i64),
            );
            for poi in world.pois.in_area(area.0, area.1) {
                let at = projection.to_picture(poi.x as f64, poi.y as f64);
                let opacity = if completed.contains(&(poi.x.rem_euclid(width), poi.y)) {
                    COMPLETED_OPACITY
                } else {
                    1.0
                };
                let pixels = ((ICON_PIXELS * zoom) * dots).round().max(1.0) as u32;
                let name = poi_picture_name(poi.kind, poi.look);
                sprites.make(&name, pixels, |size| poi_picture(poi.kind, poi.look, size));
                stamps.push((name, None, pixels, at, opacity));
            }
        }
        (None, None) => {}
    }
    (stamps, busy)
}
