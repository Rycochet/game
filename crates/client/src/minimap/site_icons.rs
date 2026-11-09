//! What is stamped upright on a site's map over its cells: secret doors and
//! remembered stairs.

use shared::site::StairsDirection;

use super::canvas::Canvas;
use super::grid::Looks;
use super::projection::Projection;
use super::{SECRET_DOOR_EDGE, SECRET_DOOR_PIXELS, STAIRS_PIXELS};
use crate::common::CurrentDungeon;
use crate::widgets::PARCHMENT;

/// `scale`: the map's zoom times the screen's pixels; `dots` the screen's.
pub(super) fn draw(
    canvas: &mut Canvas,
    looks: &Looks,
    dungeon: &CurrentDungeon,
    projection: &Projection,
    scale: f32,
    dots: f32,
) {
    for (x, y) in looks.secret_doors() {
        super::frame::secret_door_icon(
            canvas,
            projection.to_picture(x as f64, y as f64),
            SECRET_DOOR_PIXELS * scale,
            SECRET_DOOR_EDGE * dots,
            PARCHMENT,
        );
    }
    for stairs in &dungeon.stairs {
        if dungeon.remembered.is_seen(stairs.x, stairs.y) {
            super::frame::stairs_icon(
                canvas,
                projection.to_picture(stairs.x as f64, stairs.y as f64),
                STAIRS_PIXELS * scale,
                stairs.direction == StairsDirection::Down,
            );
        }
    }
}
