//! A town's options and plan: how many buildings it aims for, which way its
//! main roads run, whether it has a wall, and generating it.

use super::super::SiteLevel;
use super::super::grid::roll;
use super::super::wall::{PoiWall, WallMaterial};
use super::Town;
use super::arms::choose_arms;
use crate::rng::Seed;

/// How a town is generated. The poi's kind decides these (and the road
/// network its sides), so its size, shape and wall are known before any visit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TownOptions {
    /// Its number of buildings, at least and at most.
    pub buildings: (i64, i64),
    /// The sides (north, east, south, west) roads come in from outside
    /// (`road::town_sides`): its main roads leave the middle that way.
    pub sides: [bool; 4],
    /// Percentage chance of a wall round it.
    pub wall_percent: u32,
    /// What a wall may be made of, and how likely each is.
    pub wall_materials: &'static [(WallMaterial, u32)],
}

/// A town's plan: how many buildings it aims for, its main roads and wall.
#[derive(Clone, Debug)]
pub struct Plan {
    pub(super) seed: Seed,
    target: i64,
    /// Which arms its middle has: north, east, south, west.
    arms: [bool; 4],
    wall: Option<WallMaterial>,
}

impl Plan {
    /// How many buildings it aims to have.
    pub fn target(&self) -> i64 {
        self.target
    }

    pub fn new(options: &TownOptions, seed: Seed) -> Self {
        let mut rng = Seed::new(seed.derive("town size").at(&[0])).rng();
        let target = roll(&mut rng, options.buildings.0, options.buildings.1);
        let mut rng = Seed::new(seed.derive("town arms").at(&[0])).rng();
        let arms = choose_arms(&mut rng, target, options.sides);
        let walled = seed.derive("town wall").at(&[0]) % 100 < u64::from(options.wall_percent);
        let wall = walled.then(|| WallMaterial::pick(seed, options.wall_materials));
        Self {
            seed,
            target,
            arms,
            wall,
        }
    }

    /// Which arms its middle has: north, east, south, west.
    pub fn arms(&self) -> [bool; 4] {
        self.arms
    }

    /// Its wall as the world map shows it: what it is made of, with a gate on
    /// each side a main road leaves by (side roads' gates aren't shown).
    pub fn poi_wall(&self) -> Option<PoiWall> {
        self.wall.map(|material| PoiWall {
            west: 0,
            north: 0,
            east: 0,
            south: 0,
            material,
            gates: self.arms(),
        })
    }

    /// Generates the town (its only level).
    pub fn level(&self) -> SiteLevel {
        Town::new(self.seed, self.target, self.arms, self.wall).generate()
    }
}
