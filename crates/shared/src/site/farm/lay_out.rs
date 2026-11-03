//! Laying out a farm: its plots and the ground round the farmhouse.

use super::super::SiteLevel;
use super::super::building::{self};
use super::super::grid::{chance, roll};
use super::super::town::Rect;
use super::{
    ALL_CROPS_PERCENT, ALL_PASTURE_PERCENT, BORDER, FarmOptions, FieldLook, LANE, MIN_FIELD,
    PLACE_SIZE, POND_PERCENT, dig_pond, fence_field, lay_tracks,
};
use crate::dungeon::{Cell, Level};
use crate::rng::{Rng, Seed};

/// A rectangle by its edges, the far ones excluded.
#[derive(Clone, Copy, Debug)]
pub(in crate::site) struct Plot {
    pub x0: i64,
    pub y0: i64,
    pub x1: i64,
    pub y1: i64,
}

/// The ground floor, with everything round the house.
pub(super) fn lay_out(options: &FarmOptions, seed: Seed, house: &building::Plan) -> SiteLevel {
    let mut rng = seed.derive("farm").rng();
    let ruin = options.house.ruin;
    let inner = house.level(0);
    let (_, _, width, height) = inner.level.bounds();
    let entrance = inner.entrance.expect("a house has a way in");

    let plots = choose_plots(options, &mut rng, width, height);
    let pond = plots.len() >= 2 && chance(&mut rng, POND_PERCENT);
    let farming = match rng.range_u64(0, 100) {
        n if n < ALL_CROPS_PERCENT => Some(false),
        n if n < ALL_CROPS_PERCENT + ALL_PASTURE_PERCENT => Some(true),
        _ => None,
    };

    let mut level = surroundings(&inner.level, &plots);
    let mut fields = Vec::new();
    let mut gates = Vec::new();
    for (index, &plot) in plots.iter().enumerate() {
        if pond && index + 1 == plots.len() {
            dig_pond(&mut level, &mut rng, plot);
            continue;
        }
        let gate = fence_field(&mut level, &mut rng, plot, entrance, ruin);
        gates.push(gate);
        fields.push(FieldLook {
            rect: Rect {
                x: plot.x0 + 1,
                y: plot.y0 + 1,
                width: plot.x1 - plot.x0 - 2,
                height: plot.y1 - plot.y0 - 2,
            },
            livestock: farming.unwrap_or_else(|| chance(&mut rng, 40)),
            variety: rng.next_u64() as u32,
        });
    }
    lay_tracks(&mut level, house, entrance, &gates);

    SiteLevel {
        level,
        fields,
        ..inner
    }
}

/// Which of the eight places round the house are used, and the plot of each.
/// All the places on one side share how far out they may reach, so nothing
/// comes nearer another than `LANE`.
pub(super) fn choose_plots(
    options: &FarmOptions,
    rng: &mut Rng,
    width: i64,
    height: i64,
) -> Vec<Plot> {
    let count = roll(rng, options.fields.0, options.fields.1).clamp(0, 8) as usize;
    let mut places = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    for i in (1..places.len()).rev() {
        places.swap(i, rng.range_u64(0, i as u64 + 1) as usize);
    }
    let reach = [(); 4].map(|_| roll(rng, PLACE_SIZE.0, PLACE_SIZE.1));
    let [west, north, east, south] = reach;
    // The room in a column or row: from `from`, this big.
    let span = |side: i64, size: i64, before: i64, after: i64| match side {
        -1 => (-LANE - before, before),
        0 => (0, size),
        _ => (size + LANE, after),
    };
    places[..count]
        .iter()
        .map(|&(column, row)| {
            let (x, room_x) = span(column, width, west, east);
            let (y, room_y) = span(row, height, north, south);
            let wide = roll(rng, MIN_FIELD, room_x);
            let tall = roll(rng, MIN_FIELD, room_y);
            // Up against the house's side of the place, or anywhere along it.
            let along = |rng: &mut Rng, side: i64, from: i64, room: i64, size: i64| match side {
                -1 => from + room - size,
                0 => roll(rng, from, from + room - size),
                _ => from,
            };
            let x0 = along(rng, column, x, room_x, wide);
            let y0 = along(rng, row, y, room_y, tall);
            Plot {
                x0,
                y0,
                x1: x0 + wide,
                y1: y0 + tall,
            }
        })
        .collect()
}

/// The house's level on bare ground reaching round all the plots.
pub(in crate::site) fn surroundings(house: &Level, plots: &[Plot]) -> Level {
    let (_, _, width, height) = house.bounds();
    let edge = |near: fn(&Plot) -> i64, pick: fn(i64, i64) -> i64, house: i64, out: i64| {
        plots.iter().map(|plot| near(plot) + out).fold(house, pick)
    };
    let x0 = edge(|p| p.x0, i64::min, 0, -BORDER);
    let y0 = edge(|p| p.y0, i64::min, 0, -BORDER);
    let x1 = edge(|p| p.x1, i64::max, width, BORDER);
    let y1 = edge(|p| p.y1, i64::max, height, BORDER);
    let mut level = Level::new(x0, y0, x1 - x0, y1 - y0);
    for y in y0..y1 {
        for x in x0..x1 {
            if house.contains(x, y) {
                level.set(x, y, house.cell(x, y));
                level.set_room(x, y, house.room(x, y));
            } else {
                level.set(x, y, Cell::Ground);
            }
        }
    }
    level
}
