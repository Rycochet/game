//! A castle's plan round its keep: the wall, its towers, the pond and moat.

use super::super::super::farm::Plot;
use super::super::super::grid::{chance, roll};
use super::super::super::wall::{Gate, WallLook, WallMaterial, ruin_gaps};
use super::super::Shape;
use super::towers::gate_x;
use super::{CastleOptions, CastleSize, Tower, tower_spots};
use crate::rng::Seed;
use crate::site::Rect;

/// Percentage chance, for each step of ruin, that a tower's top floor has
/// fallen (and then the next, as long as one is left).
const FALLEN_TOP_PERCENT: u32 = 15;

/// A castle round a keep, as planned.
#[derive(Clone, Debug)]
pub(in crate::site) struct Castle {
    pub(in crate::site) look: WallLook,
    pub(super) towers: Vec<Tower>,
    pub(super) pond: Option<Plot>,
    /// How wide its moat is; 0 for none.
    pub(super) moat: i64,
}

impl Castle {
    /// The castle round `keep` (the keep's ground floor outline) that the
    /// options, its size and the seed give, with towers no taller than the
    /// keep's `storeys` and fallen as a `ruin`.
    pub(in crate::site) fn plan(
        options: &CastleOptions,
        size: &CastleSize,
        seed: Seed,
        keep: &Shape,
        (ruin, storeys): (u8, u32),
    ) -> Self {
        let [west, north, east, south] = size.gaps;
        let rect = Rect {
            x: keep.x - west - 1,
            y: keep.y - north - 1,
            width: keep.width + west + east + 2,
            height: keep.height + north + south + 2,
        };
        let gate = Gate {
            x: gate_x(rect),
            y: rect.y + rect.height - 1,
            runs_east_west: true,
        };
        let spots = tower_spots(rect);
        let mut rng = seed.derive("castle towers").rng();
        let mut fall = seed.derive("castle ruin").rng();
        let (least, most) = options.tower_floors;
        let towers = (spots.iter())
            .map(|&spot| {
                let mut floors = if spot.by_gate {
                    least as i64
                } else {
                    roll(&mut rng, least as i64, most as i64)
                } as u32;
                while ruin > 0
                    && floors > 1
                    && chance(&mut fall, FALLEN_TOP_PERCENT * u32::from(ruin))
                {
                    floors -= 1;
                }
                Tower::new(spot, floors.min(storeys.max(1)), rect, &spots)
            })
            .collect::<Vec<_>>();
        let mut look = WallLook {
            rect,
            material: WallMaterial::pick(seed, options.materials),
            gates: vec![gate],
            towers: spots.iter().map(|spot| spot.rect()).collect(),
            gaps: Vec::new(),
        };
        look.gaps = ruin_gaps(&look, seed, ruin);

        let mut rng = seed.derive("castle moat").rng();
        let moat = if chance(&mut rng, options.moat_percent) {
            let wide = i64::from(rect.width + rect.height >= options.wide_moat_from);
            roll(&mut rng, options.moat.0, options.moat.1 + wide)
        } else {
            0
        };
        let pond = pond_plot(size, seed, &look, keep);
        Self {
            look,
            towers,
            pond,
            moat,
        }
    }

    /// The tallest of its towers, in floors.
    pub(in crate::site) fn tower_floors(&self) -> u32 {
        self.towers
            .iter()
            .map(|tower| tower.floors)
            .max()
            .unwrap_or(1)
    }
}

/// Where the pond goes: a plot clear of the wall, the towers and the keep by a
/// cell all round, so the ground round it stays one piece, and out of the way
/// in from the gate. On its own side if it fits, else anywhere it does; a
/// smaller one if it fits nowhere.
fn pond_plot(size: &CastleSize, seed: Seed, wall: &WallLook, keep: &Shape) -> Option<Plot> {
    let rect = wall.rect;
    let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
    let gate_x = gate_x(rect);
    let keep_rect = Rect {
        x: keep.x,
        y: keep.y,
        width: keep.width,
        height: keep.height,
    };
    // Ground every plot cell and the cells round it must be: inside the wall,
    // in no tower, not the keep, not in front of the gate.
    let free = |x: i64, y: i64| {
        x > rect.x
            && x < x1
            && y > rect.y
            && y < y1
            && !wall.towers.iter().any(|tower| tower.contains(x, y))
            && !keep_rect.contains(x, y)
            && !((gate_x - 1..=gate_x + 2).contains(&x) && y > keep.y + keep.height)
    };
    let fits = |x0: i64, y0: i64, (wide, tall): (i64, i64)| {
        (y0 - 1..=y0 + tall).all(|y| (x0 - 1..=x0 + wide).all(|x| free(x, y)))
    };
    let on_side = |x0: i64, y0: i64, (wide, tall): (i64, i64)| match size.pond_side {
        0 => x0 + wide <= keep.x,
        1 => y0 + tall <= keep.y,
        _ => x0 >= keep.x + keep.width,
    };
    let mut rng = seed.derive("castle pond place").rng();
    let (wide, tall) = size.pond;
    for pond in [(wide, tall), (3, 3)] {
        let spots: Vec<(i64, i64)> = (rect.y..=y1)
            .flat_map(|y| (rect.x..=x1).map(move |x| (x, y)))
            .filter(|&(x, y)| fits(x, y, pond))
            .collect();
        let near: Vec<(i64, i64)> = (spots.iter().copied())
            .filter(|&(x, y)| on_side(x, y, pond))
            .collect();
        let choices = if near.is_empty() { &spots } else { &near };
        if !choices.is_empty() {
            let (x0, y0) = choices[rng.range_u64(0, choices.len() as u64) as usize];
            return Some(Plot {
                x0,
                y0,
                x1: x0 + pond.0,
                y1: y0 + pond.1,
            });
        }
    }
    None
}
