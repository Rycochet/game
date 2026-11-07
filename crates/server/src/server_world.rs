//! `ServerWorld`: one world as the server holds it, its time, sites and what has been forgotten.

use bevy::prelude::*;
use shared::celestial::Celestial;
use shared::item::StackLimits;
use shared::movement::{WorldConditions, WorldGround};
use shared::poi::PoiMap;
use shared::protocol::WorldInfo;
use shared::site::Generator;
use shared::surface::Surface;
use shared::world::{MINUTES_PER_HOUR, WORLD_MAP_VERSION, WorldParams, WorldSeed, WorldTime};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::clock::{ActionClock, ServerClock};
use super::entities::Populations;
use super::roads::RoadWorker;
use super::storage::{SAVE_FORMAT, SiteId, SiteSave, Storage, Unsaved};

mod find_item;
mod forgetting;
mod sharing;
mod time;

pub(crate) use forgetting::Forgotten;
pub(crate) use sharing::Shared;

/// One world the server runs (`ServerWorlds` holds them all).
pub struct ServerWorld {
    pub seed: WorldSeed,
    pub pois: PoiMap,
    /// The world's width in grid points: `x` wraps at this.
    pub width: i64,
    /// The ground as far as walking goes.
    pub(crate) ground: WorldGround,
    /// Works out the roads between towns (shared by every world).
    roads: Arc<RoadWorker>,
    /// The steps along every road found so far, which cost less.
    pub(crate) road_steps: shared::road::RoadSteps,
    params: WorldParams,
    /// The suns and moons: the tide follows the moons.
    celestial: Celestial,
    /// The server's clock, which the world's time is worked out from (`time`).
    pub(crate) server_clock: ServerClock,
    /// The world's time: game minutes since it began, by the server's clock.
    time: WorldTime,
    /// What creatures' energy goes by (the server's).
    pub(crate) action_clock: ActionClock,
    /// The time's whole hours. The tide follows this, so it moves once an
    /// hour rather than all the time.
    pub(crate) hour: u32,
    /// `time` has been moved or sped up, which everyone in the world needs telling.
    pub(crate) clock_moved: bool,
    pub(crate) storage: Storage,
    /// The sites entered so far (loaded when first needed).
    pub(super) sites: BTreeMap<SiteId, SiteSave>,
    unsaved_sites: Unsaved<SiteId>,
    /// Sites whose saves couldn't be reached, so what is in `sites` for them is
    /// blank, and must not be saved over the real one.
    unreachable_sites: BTreeSet<SiteId>,
    /// Who is on the site levels in use.
    pub(crate) populations: Populations,
    /// Mobs that died since players were last told who they see, so those who
    /// saw them go are told they died (`entities::sync`).
    pub(crate) died: BTreeSet<shared::id::Id>,
    /// Creatures harmed since players were last told who they see, so others
    /// who see them are told (`entities::sync`).
    pub(crate) harmed: Vec<crate::combat::Harm>,
    /// Mobs killed since `entities::sync` last told who helped.
    pub(crate) kills: Vec<crate::entities::Kill>,
    /// Developer tools' Freeze Mobs: no mob in this world thinks or moves.
    pub(crate) mobs_frozen: bool,
    /// How big stacks of items may get here.
    pub(crate) stacks: StackLimits,
    /// Sites forgotten since `super::connections::handle_messages` last looked, for
    /// putting out anyone else in them.
    forgotten: Forgotten,
    /// The wilderness's generator and what was made of it lately.
    pub(crate) wilderness: crate::wilderness::WildernessMaps,
}

impl ServerWorld {
    /// When and where something made now is made.
    pub(crate) fn made_now(&self) -> shared::item::MadeAt {
        shared::item::MadeAt {
            world: self.seed.id(),
            minute: self.minutes() as u64,
        }
    }

    /// A world that is never saved, with clocks of its own (tests).
    pub fn new(seed: WorldSeed) -> Self {
        Self::open(seed, Storage::none()).expect("nothing to read")
    }

    /// `open_in`, with clocks and a road thread of its own (tests).
    pub(super) fn open(seed: WorldSeed, storage: Storage) -> Result<Self, String> {
        Self::open_in(seed, storage, &Shared::new(0.0, 0))
    }

    /// The world saved in `storage`, or a new one (saved straight away), going
    /// by `shared`'s clocks. An error if the save can't be read or was made by
    /// a newer server.
    pub(crate) fn open_in(
        seed: WorldSeed,
        storage: Storage,
        shared: &Shared,
    ) -> Result<Self, String> {
        let saved = storage.load_world()?;
        let now = shared.clock.seconds();
        let mut changed = saved.is_none();
        let time = match saved {
            Some(save) => {
                if save.format > SAVE_FORMAT {
                    return Err(format!(
                        "it was saved in format {}; this server reads up to {SAVE_FORMAT}",
                        save.format
                    ));
                }
                if save.seed != seed {
                    return Err("its saved seed isn't the one asked for".into());
                }
                // Every world made before the generators were rolled per world
                // (version 5) was wiped with the saves (`OLDEST_SAVE_FORMAT`).
                // Version 6 only added oases: a world from 5 carries on, with
                // some of its desert pois turned into them.
                if !(5..=WORLD_MAP_VERSION).contains(&save.world_map_version) {
                    return Err(format!(
                        "it was made with world map version {}; this server has {WORLD_MAP_VERSION}",
                        save.world_map_version
                    ));
                }
                // Its time has run on from when it was saved, as the server's clock
                // has; from now if saved before that clock was (or on another
                // server, whose clock may be ahead).
                changed = save.server_clock.is_none();
                let then = save.server_clock.unwrap_or(now).min(now);
                WorldTime::at(save.clock, then, save.rate)
            }
            None => WorldTime::at(0.0, now, Default::default()),
        };
        let params = WorldParams::from_seed(seed.terrain());
        let minutes = time.minutes(now);
        let world = Self {
            pois: PoiMap::new(&params, seed.pois()),
            width: Surface::new(&params).width(),
            ground: WorldGround::new(&params),
            roads: shared.roads.clone(),
            road_steps: shared::road::RoadSteps::new(Surface::new(&params).width()),
            celestial: Celestial::from_seed(params.seed),
            server_clock: shared.clock.clone(),
            time,
            action_clock: shared.action_clock.clone(),
            hour: (minutes / MINUTES_PER_HOUR) as u32,
            clock_moved: false,
            seed,
            storage,
            sites: BTreeMap::new(),
            unsaved_sites: Unsaved::default(),
            unreachable_sites: BTreeSet::new(),
            populations: Populations::default(),
            died: BTreeSet::new(),
            harmed: Vec::new(),
            kills: Vec::new(),
            mobs_frozen: false,
            stacks: crate::stack_limits::from_environment(),
            forgotten: Forgotten::default(),
            wilderness: crate::wilderness::WildernessMaps::new(&params),
            params,
        };
        // Only new worlds, and those saved before going by the server's clock:
        // otherwise `world.ron` only changes when its time is moved.
        if changed {
            world.save_world();
        }
        Ok(world)
    }

    /// What decides where can be walked now.
    pub(crate) fn conditions(&self) -> WorldConditions {
        WorldConditions {
            water: self.params.water_level(&self.celestial, self.hour as f64),
            ice: true,
        }
    }

    /// Loads what is saved about a site, or makes it new (with its generator's
    /// current version) if nobody has been in. An error if its save couldn't be
    /// reached. Done on going into a site, where someone can be told, so that
    /// `site` doesn't need to fail.
    pub(crate) fn open_site(&mut self, id: SiteId) -> Result<(), String> {
        if self.sites.contains_key(&id) && !self.unreachable_sites.contains(&id) {
            return Ok(());
        }
        let version = self.new_version(id);
        let save = self.storage.load_site(id)?.unwrap_or_else(|| {
            self.unsaved_sites.mark(id);
            SiteSave {
                version,
                ..default()
            }
        });
        self.unreachable_sites.remove(&id);
        self.sites.insert(id, save);
        Ok(())
    }

    /// What is saved about a site. Normally open already; if it isn't and its save
    /// can't be reached, it is blank for now and not saved.
    pub(crate) fn site(&mut self, id: SiteId) -> &mut SiteSave {
        if !self.sites.contains_key(&id)
            && let Err(error) = self.open_site(id)
        {
            error!("Using a blank {id:?}, which won't be saved: {error}");
            self.unreachable_sites.insert(id);
            let save = SiteSave {
                version: self.new_version(id),
                ..default()
            };
            self.sites.insert(id, save);
        }
        self.sites.get_mut(&id).expect("just made sure")
    }

    /// `site`, to be saved: something about it is changing.
    pub(crate) fn site_mut(&mut self, id: SiteId) -> &mut SiteSave {
        self.unsaved_sites.mark(id);
        self.site(id)
    }

    /// The generator version a site is made with when first entered: its
    /// generator's current one.
    fn new_version(&self, id: SiteId) -> u32 {
        let kind = match id {
            SiteId::Poi(chunk) => self.pois.in_chunk(chunk).map(|poi| poi.kind),
            SiteId::Dev { kind, .. } => Some(kind),
            SiteId::Wilderness => return shared::wilderness::WILDERNESS_VERSION,
            SiteId::DevArea(_) => return shared::site::DEV_AREA_VERSION,
        };
        kind.map_or(0, |kind| Generator::for_poi(kind).version())
    }

    /// Saves the sites that have changed.
    pub(crate) fn save_sites(&mut self) {
        for id in self.unsaved_sites.take() {
            if let Some(save) = self.sites.get(&id)
                && !self.unreachable_sites.contains(&id)
            {
                self.storage.save_site(id, save);
            }
        }
    }

    /// What a joining client is told about the world.
    pub fn info(&self) -> WorldInfo {
        WorldInfo {
            id: self.seed.id(),
            terrain_seed: self.seed.terrain(),
            world_map_version: WORLD_MAP_VERSION,
            gates: self.gates(),
        }
    }
}
