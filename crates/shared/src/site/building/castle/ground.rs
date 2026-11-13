//! A castle's ground floor: the keep's, with the courtyard, pond, wall,
//! towers and moat round it. You arrive outside the gate, past any bridge.

use super::super::super::SiteLevel;
use super::super::super::farm::{Plot, surroundings};
use super::super::super::pond::{PondShape, dig_pond};
use super::super::super::wall::build_wall;
use super::{Castle, next_room};
use crate::dungeon::{Cell, Level};
use crate::rng::Seed;
use crate::site::Rect;

/// Ground left between the castle and its moat.
const BANK: i64 = 1;

/// A pond in a castle's courtyard: small, so it nearly fills its plot.
const POND_SHAPE: PondShape = PondShape {
    blobs: (2, 3),
    reach: (1, 2),
};

impl Castle {
    /// The keep's ground floor with the castle round it.
    pub(super) fn surround(&self, seed: Seed, floor: SiteLevel) -> SiteLevel {
        let reach = if self.moat > 0 { BANK + self.moat } else { 0 };
        let outline = self.outline();
        let x0 = outline.iter().map(|r| r.x).min().unwrap_or(0) - reach;
        let y0 = outline.iter().map(|r| r.y).min().unwrap_or(0) - reach;
        let x1 = outline.iter().map(|r| r.x + r.width).max().unwrap_or(0) + reach;
        let y1 = outline.iter().map(|r| r.y + r.height).max().unwrap_or(0) + reach;
        let mut level = surroundings(&floor.level, &[Plot { x0, y0, x1, y1 }]);
        if let Some(pond) = self.pond {
            let mut rng = seed.derive("castle pond").rng();
            dig_pond(&mut level, &mut rng, pond, POND_SHAPE);
        }
        build_wall(&mut level, &self.look);
        let mut stairs = floor.stairs.clone();
        for tower in &self.towers {
            let room = next_room(&level);
            stairs.extend(tower.build(&mut level, 0, room));
        }
        let entrance = self.dig_moat(&mut level, &outline);
        SiteLevel {
            level,
            stairs,
            entrance: Some(entrance),
            walls: vec![self.look.clone()],
            ..floor
        }
    }

    /// What the moat runs round: the wall's rectangle and its towers.
    fn outline(&self) -> Vec<Rect> {
        let towers = self.towers.iter().map(|tower| tower.spot.rect());
        std::iter::once(self.look.rect).chain(towers).collect()
    }

    /// Water round `outline`, past a bank, and a bridge over it from the gate:
    /// where you arrive, just past the bridge (or the gate, with no moat).
    fn dig_moat(&self, level: &mut Level, outline: &[Rect]) -> (i64, i64) {
        let away = |x: i64, y: i64| {
            (outline.iter())
                .map(|r| {
                    let dx = (r.x - x).max(x - (r.x + r.width - 1)).max(0);
                    let dy = (r.y - y).max(y - (r.y + r.height - 1)).max(0);
                    dx.max(dy)
                })
                .min()
                .unwrap_or(0)
        };
        // Straight outside, round all of it; inside, a bank from the wall and
        // each tower jutting into the water.
        let x0 = outline.iter().map(|r| r.x).min().unwrap_or(0);
        let y0 = outline.iter().map(|r| r.y).min().unwrap_or(0);
        let x1 = outline.iter().map(|r| r.x + r.width - 1).max().unwrap_or(0);
        let y1 = outline
            .iter()
            .map(|r| r.y + r.height - 1)
            .max()
            .unwrap_or(0);
        let round = |x: i64, y: i64| (x0 - x).max(x - x1).max(y0 - y).max(y - y1);
        let in_moat = |x: i64, y: i64| away(x, y) > BANK && round(x, y) <= BANK + self.moat;
        if self.moat > 0 {
            let (min_x, min_y, width, height) = level.bounds();
            for y in min_y..min_y + height {
                for x in min_x..min_x + width {
                    if in_moat(x, y) {
                        level.set(x, y, Cell::Water);
                        level.set_room(x, y, None);
                    }
                }
            }
        }
        let gate = self.look.gates[0];
        let mut y = gate.y + 1;
        while self.moat > 0 && round(gate.x, y) <= BANK + self.moat {
            if in_moat(gate.x, y) {
                for (x, y) in gate.cells().map(|(x, _)| (x, y)) {
                    level.set(x, y, Cell::Bridge);
                }
            }
            y += 1;
        }
        (gate.x, y)
    }
}
