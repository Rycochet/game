//! How a poi looks from the world map: ruined or not, from its seed.

use serde::{Deserialize, Serialize};

use super::Generator;
use super::building::Outline;
use super::cave::CaveEntrance;
use crate::poi::PoiKind;
use crate::rng::Seed;

/// What a site looks like from outside, for choosing a poi's model. Worked
/// out without generating any level, and holding nothing a look from outside
/// wouldn't show, so the server can send it to whoever knows of the poi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoiLook {
    Building {
        outline: Outline,
        /// Its ground floor, in cells.
        width: u16,
        height: u16,
        floors: u8,
        /// How ruined it is (`BuildingOptions::ruin`).
        ruin: u8,
    },
    Cave {
        entrance: CaveEntrance,
    },
    Town {
        buildings: u16,
    },
    Farm {
        /// The farmhouse's ground floor, in cells.
        width: u16,
        height: u16,
        ruin: u8,
        fields: u8,
    },
}

impl PoiLook {
    /// Whether it is a ruin, and is named as one.
    pub fn is_ruined(self) -> bool {
        matches!(self, Self::Building { ruin, .. } | Self::Farm { ruin, .. } if ruin > 0)
    }

    /// The look of the site a poi of this kind and map seed leads to, in
    /// desert or not (`Poi::desert`).
    pub fn of(kind: PoiKind, seed: Seed, desert: bool) -> Self {
        match Generator::for_site(kind, seed, desert) {
            Generator::Building(options) => {
                let plan = super::building::Plan::new(&options, seed);
                let (width, height) = plan.footprint();
                Self::Building {
                    outline: plan.outline(),
                    width: width as u16,
                    height: height as u16,
                    floors: plan.floors() as u8,
                    ruin: options.ruin,
                }
            }
            Generator::Cave(options) => Self::Cave {
                entrance: options.entrance,
            },
            Generator::Town(options) => Self::Town {
                buildings: super::town::Plan::new(&options, seed).target() as u16,
            },
            Generator::Herbalist(options) => {
                let plan = super::building::Plan::new(&options.house, seed);
                let (width, height) = plan.footprint();
                Self::Building {
                    outline: plan.outline(),
                    width: width as u16,
                    height: height as u16,
                    floors: 1,
                    ruin: options.house.ruin,
                }
            }
            Generator::Farm(options) => {
                let plan = super::farm::Plan::new(&options, seed);
                let (width, height) = plan.footprint();
                Self::Farm {
                    width: width as u16,
                    height: height as u16,
                    ruin: options.house.ruin,
                    fields: plan.fields() as u8,
                }
            }
        }
    }
}
