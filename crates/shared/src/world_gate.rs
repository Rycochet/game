//! World Gates: the ways onto a planet, the same number on every one. Worked
//! out from the public terrain seed, so client and server agree and anyone can
//! be shown them. Each stands on land with room to explore round it, well apart
//! from the others.
//!
//! Every seed is a planet, so every planet must have all its gates. Spots are
//! tried in rounds: each round tries new ones, and only if gates are still
//! missing does the next ask for less room round them and less space between.
//! See `docs/context/shared/world_gate.md`.

mod land;
mod reach;
mod spot;

use core::f64::consts::FRAC_PI_2;

use crate::poi::{CHUNK_SIZE, ChunkPos};
use crate::surface::Surface;
use crate::world::WorldParams;
use land::{CELL, CoarseLand};

/// World Gates on every planet.
pub const WORLD_GATES: usize = 16;
/// A screen's worth of land, in coarse cells: the world map shows about 400
/// grid points across.
const SCREEN: f64 = (400 * 400) as f64 / (CELL * CELL) as f64;
/// Gates go no further from the equator than this, in degrees, clear of the
/// ice; the land round them may reach a little further.
const FURTHEST_LATITUDE: f64 = 60.0;
const ROOM_LATITUDE: f64 = 70.0;
/// Gates' spacing on large planets, in grid points; smaller planets space them
/// by a share of their width instead, so all of them fit.
const MOST_SPACING: f64 = 1000.0;
const SPACING_SHARE: f64 = 1.0 / 12.0;
/// New spots tried each round.
const TRIES_PER_ROUND: u32 = 512;

/// One round: how many screens of land a gate needs joined to it, and its
/// spacing as a share of the planet's.
struct Round {
    screens: f64,
    spacing: f64,
}

const ROUNDS: [Round; 4] = [
    Round {
        screens: 2.0,
        spacing: 1.0,
    },
    Round {
        screens: 1.5,
        spacing: 0.7,
    },
    Round {
        screens: 1.0,
        spacing: 0.4,
    },
    Round {
        screens: 0.25,
        spacing: 0.15,
    },
];

/// One World Gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldGate {
    /// Where it stands, on dry land (`x` in `0..width`).
    pub x: i64,
    pub y: i64,
    /// Its chunk, which has no other poi.
    pub chunk: ChunkPos,
    /// The round it was found in, from 0: later rounds ask for less.
    pub round: u8,
}

/// A planet's World Gates, in the order found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldGates {
    pub gates: Vec<WorldGate>,
    /// How many spots were tried, all rounds together.
    pub tries: u32,
}

impl WorldGates {
    /// The World Gates of the planet with this surface. Fewer than
    /// `WORLD_GATES` only if even the last round couldn't place them all
    /// (`complete` says).
    pub fn find(world: &WorldParams, surface: &Surface) -> Self {
        let width = surface.width();
        let dry_above = world.high_water();
        let mut land = CoarseLand::new(surface, dry_above);
        let mut rng = world.seed.derive("world gates").rng();
        let spacing = (width as f64 * SPACING_SHARE).min(MOST_SPACING);
        let furthest_y = latitude_y(ROOM_LATITUDE, width);
        let mut found = WorldGates {
            gates: Vec::new(),
            tries: 0,
        };
        for (number, round) in ROUNDS.iter().enumerate() {
            let needed = (round.screens * SCREEN).round() as usize;
            let apart = spacing * round.spacing;
            for _ in 0..TRIES_PER_ROUND {
                if found.complete() {
                    return found;
                }
                found.tries += 1;
                // Spread evenly over the area, not by latitude.
                let x = rng.range_u64(0, width as u64) as i64;
                let most = libm::sin(FURTHEST_LATITUDE.to_radians());
                let latitude = libm::asin(rng.range_f64(-most, most)) / FRAC_PI_2;
                let y = libm::round(latitude * width as f64 / 4.0) as i64;
                let cell = land.cell_of(x, y);
                if !reach::has_room(&mut land, cell, needed, furthest_y) {
                    continue;
                }
                let Some((x, y)) = spot::dry_spot(surface, land.middle(cell), dry_above) else {
                    continue;
                };
                let x = x.rem_euclid(width);
                if found
                    .gates
                    .iter()
                    .any(|gate| distance(gate, (x, y), width) < apart)
                {
                    continue;
                }
                found.gates.push(WorldGate {
                    x,
                    y,
                    chunk: chunk_of(x, y, width),
                    round: number as u8,
                });
            }
        }
        found
    }

    /// Whether every gate was placed.
    pub fn complete(&self) -> bool {
        self.gates.len() >= WORLD_GATES
    }
}

/// The grid `y` of a latitude in degrees, on a planet `width` round (the
/// poles a quarter of the way round).
fn latitude_y(degrees: f64, width: i64) -> i64 {
    libm::round(degrees / 90.0 * width as f64 / 4.0) as i64
}

/// How far a gate is from a point, the near way round the planet.
fn distance(gate: &WorldGate, (x, y): (i64, i64), width: i64) -> f64 {
    let across = (gate.x - x).rem_euclid(width);
    let dx = across.min(width - across) as f64;
    let dy = (gate.y - y) as f64;
    libm::sqrt(dx * dx + dy * dy)
}

/// The chunk of a point, as `PoiMap::chunk_at` has it.
fn chunk_of(x: i64, y: i64, width: i64) -> ChunkPos {
    let columns = (width / CHUNK_SIZE).max(1);
    ChunkPos {
        x: (x.rem_euclid(width) / CHUNK_SIZE).min(columns - 1),
        y: y.div_euclid(CHUNK_SIZE),
    }
}

#[cfg(test)]
mod tests;
