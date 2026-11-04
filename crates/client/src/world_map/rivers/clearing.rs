//! Cutting the woods back round rivers, roads and known pois.

use bevy::prelude::*;

use super::super::roads::{east_of, smoothed, width};
use super::{CLEARING_SIDE, KnownRivers, clearing_slot};
use crate::common::KnownRoads;

/// Woods are cut back round rivers, so they run on the ground: nothing grows
/// within `CLEARED` grid units of the middle of one, and woods are back to full
/// by `CLEARED_TO`. Wider than the river, as grid points are a unit apart.
const CLEARED: f32 = 1.2;
const CLEARED_TO: f32 = 4.0;

/// The same round a road, from its edge, so wider roads clear wider.
const ROAD_CLEARED: f32 = 1.0;
const ROAD_CLEARED_TO: f32 = 3.5;

/// The same round a poi: nothing grows under its model. Only pois the server
/// has told this character of clear anything, so the woods give nothing away;
/// a character learning of one, or another character playing, changes them.
const POI_CLEARED: f32 = 8.0;
pub(super) const POI_CLEARED_TO: f32 = 13.0;

impl KnownRivers {
    /// Works out `open` for the points around `centre`.
    pub(super) fn clear_woods(&mut self, centre: (i64, i64), world_width: i64, roads: &KnownRoads) {
        let mut open = std::mem::take(&mut self.open);
        open.clear();
        open.resize((CLEARING_SIDE * CLEARING_SIDE) as usize, 1.0);
        for river in self.rivers() {
            let east = east_of(river.source.0, centre.0, world_width);
            let offset = Vec2::new(east as f32, (river.source.1 - centre.1) as f32);
            for &point in &river.points {
                clear_round(&mut open, centre, point + offset, CLEARED, CLEARED_TO);
            }
        }
        for road in roads.iter() {
            let start = road.points[0];
            let east = east_of(start.0, centre.0, world_width);
            let offset = Vec2::new(east as f32, (start.1 - centre.1) as f32);
            let edge = width(road.kind) / 2.0;
            let near = (CLEARING_SIDE / 2) as f32;
            for point in smoothed(road) {
                let at = point + offset;
                if at.x.abs() < near && at.y.abs() < near {
                    let (from, to) = (edge + ROAD_CLEARED, edge + ROAD_CLEARED_TO);
                    clear_round(&mut open, centre, at, from, to);
                }
            }
        }
        for &(x, y) in &self.cleared_pois {
            let at = Vec2::new((x - centre.0) as f32, (y - centre.1) as f32);
            clear_round(&mut open, centre, at, POI_CLEARED, POI_CLEARED_TO);
        }
        self.open = open;
        self.cleared_roads = roads.changes();
    }
}

/// Clears the woods round a point `at` (relative to `centre`): none within
/// `cleared`, back to full by `cleared_to`.
fn clear_round(open: &mut [f32], centre: (i64, i64), at: Vec2, cleared: f32, cleared_to: f32) {
    let limit = CLEARING_SIDE / 2 - 1;
    let reach = cleared_to.ceil() as i64;
    let (nx, ny) = (at.x.round() as i64, at.y.round() as i64);
    for dy in ny - reach..=ny + reach {
        for dx in nx - reach..=nx + reach {
            if dx.abs() > limit || dy.abs() > limit {
                continue;
            }
            let distance = (dx as f32 - at.x).hypot(dy as f32 - at.y);
            let t = ((distance - cleared) / (cleared_to - cleared)).clamp(0.0, 1.0);
            let slot = clearing_slot(centre.0 + dx, centre.1 + dy);
            open[slot] = open[slot].min(t * t * (3.0 - 2.0 * t));
        }
    }
}
