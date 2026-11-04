//! `ServerWorld`: the world as the server holds it, its clock, sites and what has been forgotten.

use bevy::prelude::*;
use shared::celestial::Celestial;
use shared::item::StackLimits;
use shared::movement::{WorldConditions, WorldGround};
use shared::poi::{PoiKind, PoiMap};
use shared::protocol::WorldInfo;
use shared::site::Generator;
use shared::surface::Surface;
use shared::world::{
    MINUTES_PER_HOUR, NORMAL_SNOW_LINE, WORLD_MAP_VERSION, WorldParams, WorldSeed,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use super::clock::{ActionClock, Repeating, WorldClock};
use super::entities::Populations;
use super::roads::RoadWorker;
use super::save_world;
use super::storage::{SAVE_FORMAT, SiteId, SiteSave, Storage, Unsaved};

/// How often the clock is saved, in real time. A crash loses this much of it.
const CLOCK_SAVE_EVERY: Duration = Duration::from_secs(10);

/// The world this server runs.
#[derive(Resource)]
pub struct ServerWorld {
    pub seed: WorldSeed,
    pub pois: PoiMap,
    /// The world's width in grid points: `x` wraps at this.
    pub width: i64,
    /// The ground as far as walking goes.
    pub(crate) ground: WorldGround,
    /// Works out the roads between towns.
    pub(crate) roads: RoadWorker,
    /// The steps along every road found so far, which cost less.
    pub(crate) road_steps: shared::road::RoadSteps,
    params: WorldParams,
    /// The suns and moons: the tide follows the moons.
    celestial: Celestial,
    /// Game minutes since the world began, always running.
    pub(crate) clock: WorldClock,
    /// What creatures' energy goes by.
    pub(crate) action_clock: ActionClock,
    /// The clock's whole hours. The tide follows this, so it moves once an
    /// hour rather than all the time.
    pub(crate) hour: u32,
    /// `clock` has been moved, which everyone playing needs telling.
    pub(crate) clock_moved: bool,
    /// Saves the clock (`None`: nowhere to save it).
    _clock_saver: Option<Repeating>,
    /// Above it everything is frozen (not saved: normal weather on starting).
    pub(crate) snow_line: u32,
    pub(crate) storage: Storage,
    /// The sites entered so far (loaded when first needed).
    pub(super) sites: BTreeMap<SiteId, SiteSave>,
    unsaved_sites: Unsaved<SiteId>,
    /// Sites whose saves couldn't be reached, so what is in `sites` for them is
    /// blank, and must not be saved over the real one.
    unreachable_sites: BTreeSet<SiteId>,
    /// Who is on the site levels in use.
    pub(crate) populations: Populations,
    /// How big stacks of items may get here.
    pub(crate) stacks: StackLimits,
    /// Sites forgotten since `super::connections::handle_messages` last looked, for
    /// putting out anyone else in them.
    forgotten: Forgotten,
}

/// Which sites have been forgotten.
#[derive(Default)]
pub(crate) struct Forgotten {
    all: bool,
    sites: BTreeSet<SiteId>,
}

impl Forgotten {
    pub(crate) fn is_empty(&self) -> bool {
        !self.all && self.sites.is_empty()
    }

    pub(crate) fn includes(&self, id: SiteId) -> bool {
        self.all || self.sites.contains(&id)
    }
}

impl ServerWorld {
    /// When and where something made now is made.
    pub(crate) fn made_now(&self) -> shared::item::MadeAt {
        shared::item::MadeAt {
            world: self.seed.id(),
            minute: self.clock.minutes() as u64,
        }
    }

    /// A world that is never saved (tests).
    pub fn new(seed: WorldSeed) -> Self {
        Self::open(seed, Storage::none()).expect("nothing to read")
    }

    /// The world saved in `storage`, or a new one (saved straight away). An
    /// error if the save can't be read or was made by a newer server.
    pub(super) fn open(seed: WorldSeed, storage: Storage) -> Result<Self, String> {
        let saved = storage.load_world()?;
        let saved_ticks = saved.as_ref().map_or(0, |save| save.action_ticks);
        let clock = match saved {
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
                // Older versions differ only in having no towns (1), or no farms or
                // herbalists' huts (2):
                // their worlds carry on with some pois turned into those.
                if !(1..=WORLD_MAP_VERSION).contains(&save.world_map_version) {
                    return Err(format!(
                        "it was made with world map version {}; this server has {WORLD_MAP_VERSION}",
                        save.world_map_version
                    ));
                }
                save.clock
            }
            None => 0.0,
        };
        let params = WorldParams::from_seed(seed.terrain());
        let world_clock = WorldClock::new(clock);
        let save_clock = world_clock.clone();
        let save_to = storage.clone();
        let action_clock = ActionClock::new(saved_ticks);
        let save_ticks = action_clock.clone();
        let clock_saver = storage.saves().then(|| {
            Repeating::start(CLOCK_SAVE_EVERY, move || {
                save_world(&save_to, seed, save_clock.minutes(), save_ticks.ticks());
            })
        });
        let world = Self {
            pois: PoiMap::new(&params, seed.pois()),
            width: Surface::new(&params).width(),
            ground: WorldGround::new(&params),
            roads: RoadWorker::start(seed, storage.clone()),
            road_steps: shared::road::RoadSteps::new(Surface::new(&params).width()),
            celestial: Celestial::from_seed(params.seed),
            params,
            clock: world_clock,
            action_clock,
            hour: (clock / MINUTES_PER_HOUR) as u32,
            clock_moved: false,
            _clock_saver: clock_saver,
            snow_line: NORMAL_SNOW_LINE,
            seed,
            storage,
            sites: BTreeMap::new(),
            unsaved_sites: Unsaved::default(),
            unreachable_sites: BTreeSet::new(),
            populations: Populations::default(),
            stacks: crate::stack_limits::from_environment(),
            forgotten: Forgotten::default(),
        };
        save_world(&world.storage, seed, clock, saved_ticks);
        Ok(world)
    }

    /// What decides where can be walked now.
    pub(crate) fn conditions(&self) -> WorldConditions {
        WorldConditions {
            water: self.params.water_level(&self.celestial, self.hour as f64),
            snow_line: self.snow_line as f64,
        }
    }

    /// Loads what is saved about a site (for a `kind` of poi), or makes it new
    /// (with its generator's current version) if nobody has been in. An error if
    /// its save couldn't be reached. Done on going into a site, where someone can
    /// be told, so that `site` doesn't need to fail.
    pub(crate) fn open_site(&mut self, id: SiteId, kind: PoiKind) -> Result<(), String> {
        if self.sites.contains_key(&id) && !self.unreachable_sites.contains(&id) {
            return Ok(());
        }
        let save = self.storage.load_site(id)?.unwrap_or_else(|| {
            self.unsaved_sites.mark(id);
            SiteSave {
                version: Generator::for_poi(kind).version(),
                ..default()
            }
        });
        self.unreachable_sites.remove(&id);
        self.sites.insert(id, save);
        Ok(())
    }

    /// What is saved about a site. Normally open already; if it isn't and its save
    /// can't be reached, it is blank for now and not saved.
    pub(crate) fn site(&mut self, id: SiteId, kind: PoiKind) -> &mut SiteSave {
        if !self.sites.contains_key(&id)
            && let Err(error) = self.open_site(id, kind)
        {
            error!("Using a blank {id:?}, which won't be saved: {error}");
            self.unreachable_sites.insert(id);
            let save = SiteSave {
                version: Generator::for_poi(kind).version(),
                ..default()
            };
            self.sites.insert(id, save);
        }
        self.sites.get_mut(&id).expect("just made sure")
    }

    /// `site`, to be saved: something about it is changing.
    pub(crate) fn site_mut(&mut self, id: SiteId, kind: PoiKind) -> &mut SiteSave {
        self.unsaved_sites.mark(id);
        self.site(id, kind)
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

    /// Forgets a site: it is made anew when next entered, with its generator's
    /// current version and no players' changes.
    pub(crate) fn forget_site(&mut self, id: SiteId) {
        self.sites.remove(&id);
        self.unreachable_sites.remove(&id);
        self.unsaved_sites.unmark(&id);
        self.storage.remove_site(id);
        self.populations.forget_site(id);
        self.forgotten.sites.insert(id);
    }

    /// Forgets every site, as `forget_site`.
    pub(crate) fn forget_sites(&mut self) {
        self.sites.clear();
        self.unreachable_sites.clear();
        self.unsaved_sites.take();
        self.storage.remove_sites();
        self.populations.forget_all();
        self.forgotten.all = true;
    }

    /// The sites forgotten since this was last asked.
    pub(crate) fn take_forgotten(&mut self) -> Forgotten {
        std::mem::take(&mut self.forgotten)
    }

    /// What a joining client is told about the world.
    pub fn info(&self) -> WorldInfo {
        WorldInfo {
            id: self.seed.id(),
            terrain_seed: self.seed.terrain(),
            world_map_version: WORLD_MAP_VERSION,
        }
    }
}
