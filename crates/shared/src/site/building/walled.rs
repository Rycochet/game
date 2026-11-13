//! A wall round a building (forts): planned with it from its own seed, so
//! nothing else about the building changes, and set round its ground floor.
//!
//! Each side has its own gap of ground to the building. A pond, if any, goes
//! on the west, north or east (never by the gate), with a lane of ground
//! between it and both the wall and the building, so the ground inside the
//! wall stays one ring that can be walked all the way round. The gate is in
//! the middle of the south side, the way in; you arrive just outside it.

use super::super::Rect;
use super::super::SiteLevel;
use super::super::farm::{Plot, surroundings};
use super::super::grid::{chance, roll};
use super::super::pond::{PondShape, dig_pond};
use super::super::wall::{Gate, WallLook, WallMaterial, WallOptions, build_wall};
use super::Shape;
use crate::rng::Seed;

/// How a pond inside a wall is made: small, so it nearly fills its plot.
const POND_SHAPE: PondShape = PondShape {
    blobs: (2, 3),
    reach: (1, 2),
};

/// A building's wall, as planned.
#[derive(Clone, Debug)]
pub(in crate::site) struct Walled {
    pub(in crate::site) look: WallLook,
    pond: Option<Plot>,
}

impl Walled {
    /// The wall round `ground` (the ground floor's outline) that `options` and
    /// the seed give, if it has one.
    pub(super) fn plan(options: &WallOptions, seed: Seed, ground: &Shape) -> Option<Self> {
        let mut rng = seed.derive("wall").rng();
        if !chance(&mut rng, options.percent) {
            return None;
        }
        // West, north, east, south.
        let mut gaps = [(); 4].map(|_| roll(&mut rng, options.gap.0, options.gap.1));
        let pond_size = options
            .pond
            .map(|(least, most)| (roll(&mut rng, least, most), roll(&mut rng, least, most)));
        let side = rng.range_u64(0, 3) as usize;
        if let Some((wide, tall)) = pond_size {
            // A lane each side of it.
            let across = if side == 1 { tall } else { wide };
            gaps[side] = gaps[side].max(across + 2);
        }
        let [west, north, east, south] = gaps;
        let rect = Rect {
            x: ground.x - west - 1,
            y: ground.y - north - 1,
            width: ground.width + west + east + 2,
            height: ground.height + north + south + 2,
        };
        let (x1, y1) = (rect.x + rect.width - 1, rect.y + rect.height - 1);
        let pond = pond_size.map(|(wide, tall)| {
            // Anywhere in its side's strip, a cell from the wall and the building.
            let (x0, y0) = match side {
                0 => (
                    roll(&mut rng, rect.x + 2, ground.x - 1 - wide),
                    roll(&mut rng, rect.y + 2, y1 - 1 - tall),
                ),
                1 => (
                    roll(&mut rng, rect.x + 2, x1 - 1 - wide),
                    roll(&mut rng, rect.y + 2, ground.y - 1 - tall),
                ),
                _ => (
                    roll(&mut rng, ground.x + ground.width + 1, x1 - 1 - wide),
                    roll(&mut rng, rect.y + 2, y1 - 1 - tall),
                ),
            };
            Plot {
                x0,
                y0,
                x1: x0 + wide,
                y1: y0 + tall,
            }
        });
        let gate = Gate {
            x: rect.x + (rect.width - 2) / 2,
            y: y1,
            runs_east_west: true,
        };
        let look = WallLook {
            rect,
            material: WallMaterial::pick(seed, options.materials),
            gates: vec![gate],
        };
        Some(Self { look, pond })
    }

    /// The ground floor with the wall round it, on ground reaching past it.
    pub(super) fn surround(&self, seed: Seed, floor: SiteLevel) -> SiteLevel {
        let rect = self.look.rect;
        let plot = Plot {
            x0: rect.x,
            y0: rect.y,
            x1: rect.x + rect.width,
            y1: rect.y + rect.height,
        };
        let mut level = surroundings(&floor.level, &[plot]);
        if let Some(pond) = self.pond {
            let mut rng = seed.derive("wall pond").rng();
            dig_pond(&mut level, &mut rng, pond, POND_SHAPE);
        }
        build_wall(&mut level, &self.look);
        let gate = self.look.gates[0];
        SiteLevel {
            level,
            entrance: Some((gate.x, gate.y + 1)),
            walls: vec![self.look.clone()],
            ..floor
        }
    }
}
