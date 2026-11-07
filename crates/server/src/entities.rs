//! Who else is on a map: the mobs on site levels in use, shared by everyone on
//! them, what those mobs are doing, and telling each player who they can see.
//!
//! A level is kept while someone is there, for a while after, so that someone
//! reconnecting finds the same mobs, and while any mob on it isn't settled, so that
//! nobody can leave a chasing mob frozen at the stairs for when they come back.

use serde::{Deserialize, Serialize};
use shared::combat::Wielded;
use shared::creature::CreatureKind;
use shared::dungeon::Level;
use shared::energy::{Energy, SITE_SCALE};
use shared::id::Id;
use shared::mob::Memory;
use shared::movement::STRAIGHT_COST;
use shared::protocol::CharacterId;
use shared::sheet::{Appearance, Attributes, Feeling};
use shared::vitals::Vitals;
use shared::wear::WornLook;
use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use crate::combat::Fight;
use crate::storage::SiteId;

// Split into files (`cargo xtask lint` keeps files small).
mod crossing;
mod fight;
mod gear;
mod harmed_by;
mod heal;
mod moves;
mod notice;
mod population;
mod run;
mod scent;
mod sync;
mod target;
mod territory;
mod tracking;
mod wild;

use crossing::cross;
pub(crate) use fight::{MobBlow, Striker, hit_mob};
use gear::dressed;
pub(super) use gear::{give, kill};
use harmed_by::HarmedBy;
pub(crate) use harmed_by::Kill;
pub(crate) use heal::heal_mob;
use moves::{Taken, act, nearest_free, take_step};
use notice::{Maps, Someone, Spot, cells_apart, everyone_playing, notice};
pub(super) use run::run_mobs;
use run::world_gap;
pub(crate) use scent::Scents;
pub(super) use sync::{in_hall, sync};
pub(crate) use wild::{WildAreas, keep_wild_areas};

/// How long a level nobody is on stays in use.
const KEPT_FOR: Duration = Duration::from_secs(60);

/// How near a mob going up to someone friendly stops.
const FRIENDLY_GAP: i64 = 2;
/// How near one running at someone stops, until there is combat.
const ATTACK_GAP: i64 = 1;
/// How many steps a mob wanders after losing someone, before going home.
const SETTLING_STEPS: u8 = 6;
/// How long going home may take, in ticks: past this it just gets there, once
/// nobody can see it.
const HOME_WITHIN: u64 = 1200;
/// What thinking with nothing to do costs: a straight step's worth.
const THINKING_COST: u32 = STRAIGHT_COST * SITE_SCALE;

/// What a mob is doing. Anything but `Idle` keeps it, and its level, going.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Doing {
    /// At rest, and needing no time until something wakes it.
    Idle,
    /// Going after someone, stopping this near.
    Going {
        after: Id,
        last_seen: (i64, i64),
        gap: i64,
    },
    /// Going after someone it saw take these stairs.
    Following {
        after: Id,
        stairs: (i64, i64),
        to_level: u32,
        arrives: (i64, i64),
        gap: i64,
    },
    /// Getting away from someone it can see.
    Fleeing {
        from: Id,
    },
    /// Wandering from where it lost someone, so as not to wait there.
    Settling {
        steps_left: u8,
    },
    GoingHome {
        since: u64,
    },
    /// Standing between someone and its home, at its territory's edge.
    Guarding {
        against: Id,
        last_seen: (i64, i64),
    },
    /// Following someone's scent, stopping this near once it sees them, by marks
    /// left at or after `since`.
    Tracking {
        after: Id,
        gap: i64,
        since: u64,
    },
}

/// Something on a level that isn't a player.
#[derive(Clone, Debug)]
pub(crate) struct Mob {
    pub id: Id,
    pub kind: CreatureKind,
    pub appearance: Appearance,
    pub position: (i64, i64),
    /// Where it started, and goes back to.
    pub home: (i64, i64),
    pub doing: Doing,
    /// Who it has in its sights: a player or another creature it means to get
    /// close to as an enemy (`Doing::target`). Never told to players.
    pub target: Option<Id>,
    /// Which of its level's mobs it is.
    number: u32,
    /// What it remembers.
    memory: Memory,
    /// The level it belongs on, and the stairs back there, last taken last.
    home_level: u32,
    way_back: Vec<Hop>,
    energy: Energy,
    speed: u32,
    /// What can be seen of what it wears, from when it was made: mobs don't change
    /// clothes.
    shown: Vec<WornLook>,
    /// How strong a trail it leaves, as a percentage.
    scent: u32,
    /// What it is like, for fighting (`Spawn::attributes`).
    attributes: Attributes,
    /// Its Health: not saved, so a mob unloaded is whole again.
    vitals: Vitals,
    /// What it holds to fight with, from what it was made with.
    wielded: Wielded,
    fight: Fight,
    /// Who has harmed it lately, to share in killing it. Not saved.
    harmed_by: HarmedBy,
}

impl Mob {
    /// How it feels about a creature of this kind, played by this character, with
    /// what it remembers of them.
    fn feeling(&self, kind: CreatureKind, character: Option<&CharacterId>) -> Feeling {
        let start = self.kind.feeling_about(kind).warmth();
        let warmed = character.map_or(0, |character| self.memory.warmth_for(character));
        Feeling::from_warmth(start + warmed)
    }
}

/// Stairs a mob has come by: where they are here, and where they lead back to.
#[derive(Clone, Copy, Debug)]
struct Hop {
    stairs: (i64, i64),
    to_level: u32,
    arrives: (i64, i64),
}

/// Someone has taken stairs: mobs after them that saw will follow.
pub(crate) struct StairsTaken {
    pub who: Id,
    pub site: SiteId,
    pub from_level: u32,
    pub stairs: (i64, i64),
    pub to_level: u32,
    pub arrives: (i64, i64),
}

/// A mob going to another level.
struct Crossing {
    mob: Id,
    site: SiteId,
    from_level: u32,
    to_level: u32,
    from_stairs: (i64, i64),
    arrives: (i64, i64),
    /// It is on its way home, not after someone.
    going_back: bool,
}

/// The mobs of one level, and how long nobody has been there.
#[derive(Debug)]
pub(crate) struct Population {
    pub(crate) mobs: Vec<Mob>,
    /// Its shape, to find ways about.
    level: Level,
    /// Who is due to act: the tick, and which mob. A mob at rest isn't here, so
    /// costs nothing however long it rests.
    due: BTreeSet<(u64, Id)>,
    /// How many of its mobs are on other levels, so it must stay for their return.
    away: u32,
    empty_since: Option<Instant>,
    /// Everyone's scent marks on it.
    scents: Scents,
    /// Where each player on it was last tick, and how strongly they smell, to
    /// mark the cells they leave.
    last_at: BTreeMap<Id, ((i64, i64), u32)>,
}

impl Population {
    /// Whether every mob is at rest, so nothing is going on here.
    fn is_settled(&self) -> bool {
        self.away == 0 && self.mobs.iter().all(|mob| mob.doing == Doing::Idle)
    }
}

/// The mobs on the levels in use.
#[derive(Debug, Default)]
pub(crate) struct Populations(pub(crate) BTreeMap<(SiteId, u32), Population>);

impl Populations {
    /// Forgets a site's: it is being made anew.
    pub(crate) fn forget_site(&mut self, id: SiteId) {
        self.0.retain(|(site, _), _| *site != id);
    }

    pub(crate) fn forget_all(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests;
