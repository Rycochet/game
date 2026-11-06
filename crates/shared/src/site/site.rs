//! A site behind a poi: its plan, levels and stairs.

use super::building::BuildingOptions;
use super::{
    BOXED_CAVE_VERSION, BUILDING_VERSION, CAVE_VERSION, Exit, FARM_VERSION, Generator,
    HERBALIST_VERSION, OASIS_VERSION, SiteLevel, Stairs, StairsStyle, TOWN_VERSION,
};
use crate::poi::PoiKind;
use crate::rng::Seed;

/// A poi's map: its plan, from which any level can be generated.
#[derive(Clone, Debug)]
pub struct Site {
    kind: PoiKind,
    seed: Seed,
    pub(super) plan: Plan,
    ruined: bool,
}

#[derive(Clone, Debug)]
pub(super) enum Plan {
    Building(super::building::Plan),
    Cave(super::cave::Plan),
    Town(super::town::Plan),
    Farm(super::farm::Plan),
    Herbalist(super::herbalist::Plan),
    Oasis(super::oasis::Plan),
}

impl Site {
    /// The site a poi leads to, from its kind and map seed, made with
    /// the current version of its generator.
    pub fn new(kind: PoiKind, seed: Seed) -> Self {
        let version = Generator::for_poi(kind).version();
        Self::with_version(kind, seed, version).expect("the current version runs")
    }

    /// The same, made with an older (or the current) version of its
    /// generator: `None` if that version can no longer be run.
    pub fn with_version(kind: PoiKind, seed: Seed, version: u32) -> Option<Self> {
        Self::with_generator(kind, seed, version, Generator::for_poi(kind))
    }

    /// The same, with options other than the kind's own (a ruined building).
    pub fn with_generator(
        kind: PoiKind,
        seed: Seed,
        version: u32,
        generator: Generator,
    ) -> Option<Self> {
        let ruined = match generator {
            Generator::Building(options) => options.ruin > 0,
            Generator::Farm(options) => options.house.ruin > 0,
            Generator::Herbalist(options) => options.house.ruin > 0,
            _ => false,
        };
        let plan = match generator {
            Generator::Building(options) if version == generator.version() => {
                Plan::Building(super::building::Plan::new(&options, seed))
            }
            Generator::Building(options) if options.stilts && version == BUILDING_VERSION => {
                let options = BuildingOptions {
                    min_floors: 1,
                    max_floors: 1,
                    same_size: false,
                    stilts: false,
                    ..options
                };
                Plan::Building(super::building::Plan::new(&options, seed))
            }
            Generator::Cave(options) if version == CAVE_VERSION => {
                Plan::Cave(super::cave::Plan::new(&options, seed, true))
            }
            Generator::Cave(options) if version == BOXED_CAVE_VERSION => {
                Plan::Cave(super::cave::Plan::new(&options, seed, false))
            }
            Generator::Town(options) if version == TOWN_VERSION => {
                Plan::Town(super::town::Plan::new(&options, seed))
            }
            Generator::Farm(options) if version == FARM_VERSION => {
                Plan::Farm(super::farm::Plan::new(&options, seed))
            }
            Generator::Herbalist(options) if version == HERBALIST_VERSION => {
                Plan::Herbalist(super::herbalist::Plan::new(&options, seed))
            }
            Generator::Oasis(options) if version == OASIS_VERSION => {
                Plan::Oasis(super::oasis::Plan::new(&options, seed))
            }
            _ => return None,
        };
        Some(Self {
            kind,
            seed,
            plan,
            ruined,
        })
    }

    /// Whether it is a ruined building, and is named as one.
    pub fn is_ruined(&self) -> bool {
        self.ruined
    }

    pub fn kind(&self) -> PoiKind {
        self.kind
    }

    /// Whether its levels are a building's floors (going up from the ground)
    /// rather than a cave's depths (going down).
    pub fn is_building(&self) -> bool {
        matches!(
            self.plan,
            Plan::Building(_) | Plan::Farm(_) | Plan::Herbalist(_)
        )
    }

    /// Whether its last level is a basement: below the ground floor, whatever
    /// its index says.
    pub fn has_basement(&self) -> bool {
        match &self.plan {
            Plan::Building(plan) => plan.has_basement(),
            Plan::Farm(plan) => plan.has_basement(),
            Plan::Herbalist(plan) => plan.house().has_basement(),
            _ => false,
        }
    }

    /// How many levels it has.
    pub fn levels(&self) -> u32 {
        match &self.plan {
            Plan::Building(plan) => plan.floors(),
            Plan::Cave(plan) => plan.levels(),
            Plan::Town(_) | Plan::Oasis(_) => 1,
            Plan::Farm(plan) => plan.levels(),
            Plan::Herbalist(plan) => plan.house().floors(),
        }
    }

    /// Generates level `index` (`0` is the one entered from the world map:
    /// a building's ground floor, a cave's first level).
    pub fn level(&self, index: u32) -> SiteLevel {
        assert!(index < self.levels(), "no level {index}");
        match &self.plan {
            Plan::Building(plan) => plan.level(index),
            Plan::Cave(plan) => plan.level(index),
            Plan::Town(plan) => plan.level(),
            Plan::Farm(plan) => plan.level(index),
            Plan::Herbalist(plan) => plan.level(index),
            Plan::Oasis(plan) => plan.level(),
        }
    }

    /// What `stairs` on level `index` are built as, the same at both ends:
    /// it depends on the site's seed and the upper of the two levels joined
    /// (the world map counts as above everything). Only for looks; it draws
    /// nothing from the generators' random sequences.
    pub fn stairs_style(&self, index: u32, stairs: Stairs) -> StairsStyle {
        let upper = match stairs.to {
            Exit::WorldMap => -1,
            Exit::Level(to) => i64::from(index.min(to)),
        };
        let choices = StairsStyle::choices(self.kind);
        let hash = self.seed.derive("stairs style").at(&[upper]);
        choices[(hash % choices.len() as u64) as usize]
    }
}
