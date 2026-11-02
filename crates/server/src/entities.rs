//! Who else is on a map: the mobs on site levels in use, shared by everyone on
//! them, what those mobs are doing, and telling each player who they can see.
//!
//! A level is kept while someone is there, for a while after, so that someone
//! reconnecting finds the same mobs, and while any mob on it isn't settled, so that
//! nobody can leave a chasing mob frozen at the stairs for when they come back.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use shared::creature::CreatureKind;
use shared::dungeon::{Level, line_is_clear};
use shared::energy::{Energy, SITE_SCALE, site_step_cost};
use shared::id::Id;
use shared::item::{Inventory, ItemKind, MadeAt, Quality};
use shared::mob::{EntityLook, Event, GIVE_REACH, Memory, spawns};
use shared::movement::{STRAIGHT_COST, Step, step_on_level};
use shared::pathfinding::DistanceMap;
use shared::poi::{Poi, PoiKind};
use shared::protocol::{ServerMessage, WORLD_MAP_SIGHT};
use shared::rng::Seed;
use shared::sheet::{Appearance, Feeling, Reaction, Sheet};
use shared::site::SiteLevel;

use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::ids::new_id;
use crate::storage::{SavedMob, SiteId};
use shared::protocol::CharacterId;

/// How long a level nobody is on stays in use.
const KEPT_FOR: Duration = Duration::from_secs(60);

/// How far a mob notices someone it can see, in cells.
const NOTICES_WITHIN: i64 = 12;
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
    /// Which of its level's mobs it is.
    number: u32,
    /// What it remembers.
    memory: Memory,
    /// The level it belongs on, and the stairs back there, last taken last.
    home_level: u32,
    way_back: Vec<Hop>,
    energy: Energy,
    speed: u32,
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
    /// What kind of poi the site is behind, to reach its save.
    poi_kind: PoiKind,
    empty_since: Option<Instant>,
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

impl ServerWorld {
    /// Makes sure a level about to be used has its mobs. Each keeps the id it was
    /// first given, saved with the site.
    pub(crate) fn populate(&mut self, id: SiteId, poi: Poi, index: u32, level: &SiteLevel) {
        if self.populations.0.contains_key(&(id, index)) {
            return;
        }
        let stairs: Vec<_> = level.stairs.iter().map(|s| (s.x, s.y)).collect();
        let plan = spawns(poi.seed, index, poi.kind, &level.level, &stairs);
        let self_tick = self.action_clock.ticks();
        let made_at = self.made_now();
        let saved = self.site_mut(id, poi.kind).levels.entry(index).or_default();
        let unsettled = std::mem::take(&mut saved.unsettled);
        let records = saved.records.clone();
        let mut due = BTreeSet::new();
        let tick = self_tick;
        let mobs = plan
            .into_iter()
            .map(|spawn| {
                let carried_on = unsettled.get(&spawn.number);
                let id = *saved.mobs.entry(spawn.number).or_insert_with(new_id);
                let carrying = saved
                    .records
                    .entry(id)
                    .or_default()
                    .gear
                    .get_or_insert_with(|| dressed(spawn.kind, &spawn.gear, made_at))
                    .clone();
                if carried_on.is_some() {
                    due.insert((tick, id));
                }
                Mob {
                    id,
                    kind: spawn.kind,
                    appearance: spawn.appearance,
                    position: carried_on.map_or(spawn.position, |saved| saved.position),
                    home: spawn.position,
                    doing: carried_on.map_or(Doing::Idle, |saved| saved.doing),
                    number: spawn.number,
                    memory: records
                        .get(&id)
                        .map_or_else(Memory::default, |r| r.memory.clone()),
                    home_level: index,
                    way_back: Vec::new(),
                    energy: Energy::default(),
                    speed: Sheet::new(spawn.kind).speed(&carrying),
                }
            })
            .collect();
        let population = Population {
            mobs,
            level: level.level.clone(),
            due,
            away: 0,
            poi_kind: poi.kind,
            empty_since: None,
        };
        self.populations.0.insert((id, index), population);
    }

    /// Whether a mob on this cell won't let this player past.
    pub(crate) fn enemy_at(
        &self,
        place: (SiteId, u32),
        cell: (i64, i64),
        kind: CreatureKind,
        character: Option<&CharacterId>,
    ) -> bool {
        self.populations.0.get(&place).is_some_and(|population| {
            population.mobs.iter().any(|mob| {
                mob.position == cell && mob.feeling(kind, character) == Feeling::Unfriendly
            })
        })
    }

    /// A player has stepped from `from` to `to`: a mob there takes their place.
    pub(crate) fn swap_mob(&mut self, place: (SiteId, u32), to: (i64, i64), from: (i64, i64)) {
        if let Some(population) = self.populations.0.get_mut(&place) {
            for mob in &mut population.mobs {
                if mob.position == to {
                    mob.position = from;
                }
            }
        }
    }

    /// A player has arrived on this cell, not by a step: a mob there moves to the
    /// nearest free cell it can stand on.
    pub(crate) fn move_mob_aside(&mut self, place: (SiteId, u32), cell: (i64, i64)) {
        let Some(population) = self.populations.0.get_mut(&place) else {
            return;
        };
        let Some(index) = population.mobs.iter().position(|mob| mob.position == cell) else {
            return;
        };
        let taken = |spot: (i64, i64)| population.mobs.iter().any(|mob| mob.position == spot);
        if let Some(spot) = nearest_free(&population.level, cell, taken) {
            population.mobs[index].position = spot;
        }
    }

    /// Mobs after someone who has just taken stairs, and saw, follow if they can.
    pub(crate) fn saw_stairs_taken(&mut self, taken: StairsTaken) {
        let place = (taken.site, taken.from_level);
        let Some(population) = self.populations.0.get_mut(&place) else {
            return;
        };
        for mob in &mut population.mobs {
            if let Doing::Going { after, gap, .. } = mob.doing
                && after == taken.who
                && mob.kind.def().uses_stairs
                && line_is_clear(&population.level, mob.position, taken.stairs)
            {
                mob.doing = Doing::Following {
                    after,
                    stairs: taken.stairs,
                    to_level: taken.to_level,
                    arrives: taken.arrives,
                    gap,
                };
            }
        }
    }

    /// Notes in the sites' saves which mobs aren't at rest, to carry on after a
    /// restart. Those on other levels start again from home.
    pub(crate) fn save_mobs(&mut self) {
        let mut notes = Vec::new();
        for (&(site, level), population) in &self.populations.0 {
            let unsettled: BTreeMap<u32, SavedMob> = population
                .mobs
                .iter()
                .filter(|mob| {
                    mob.doing != Doing::Idle && mob.home_level == level && mob.way_back.is_empty()
                })
                .map(|mob| {
                    let saved = SavedMob {
                        position: mob.position,
                        doing: mob.doing,
                    };
                    (mob.number, saved)
                })
                .collect();
            notes.push((site, level, population.poi_kind, unsettled));
        }
        for (site, level, kind, unsettled) in notes {
            let known = self.site(site, kind).levels.get(&level);
            let same = known.is_none_or(|saved| saved.unsettled == unsettled);
            if !same || (known.is_none() && !unsettled.is_empty()) {
                let saved = self.site_mut(site, kind).levels.entry(level).or_default();
                saved.unsettled = unsettled;
            }
        }
    }
}

/// A new mob's belongings: it wears what it can, and keeps the rest in its
/// pockets while there is room.
fn dressed(kind: CreatureKind, gear: &[(ItemKind, Option<Quality>)], made_at: MadeAt) -> Inventory {
    let mut carrying = Inventory {
        creature: kind,
        outfitted: true,
        pockets_size: kind.def().pockets,
        ..Inventory::default()
    };
    for &(item, quality) in gear {
        let quality = quality.unwrap_or_default();
        let new = crate::connections::made_for(&carrying, item, None, false, quality, made_at);
        let id = new.id;
        if carrying.give(new).is_ok() {
            // Anything that can't be worn stays in the pockets.
            let _ = carrying.wear(id);
        }
    }
    carrying
}

/// A player gives a carried item to a mob near enough and in sight. It remembers,
/// and keeps the item.
pub(crate) fn give(
    client: &mut Client,
    world: &mut ServerWorld,
    item: Id,
    to: Id,
) -> Result<(), &'static str> {
    let character = client.character_id.ok_or("nobody to remember")?;
    let visit = client.visit.as_mut().ok_or("not in a site")?;
    let (from, place) = (visit.position(), (visit.id(), visit.index()));
    let population = world
        .populations
        .0
        .get_mut(&place)
        .ok_or("nobody on this level")?;
    let mob = population
        .mobs
        .iter_mut()
        .find(|mob| mob.id == to)
        .ok_or("no such mob here")?;
    if cells_apart(from, mob.position) > GIVE_REACH
        || !line_is_clear(&population.level, from, mob.position)
    {
        return Err("too far away");
    }
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't hand that over")?;
    let event = Event::Gave(item.kind);
    mob.memory.remember(character, event);
    // Saved with the level it belongs on, wherever it has got to.
    let (kind, home_level) = (population.poi_kind, mob.home_level);
    let memory = mob.memory.clone();
    let saved = world
        .site_mut(place.0, kind)
        .levels
        .entry(home_level)
        .or_default();
    let record = saved.records.entry(to).or_default();
    record.memory = memory;
    record.carried.push(item);
    Ok(())
}

/// A player as others see them.
struct Someone {
    id: Id,
    character: Option<CharacterId>,
    look: (CreatureKind, Appearance),
    /// The site level they are on, if any.
    level: Option<(SiteId, u32)>,
    position: (i64, i64),
    /// Mobs take no notice of them.
    god_mode: bool,
}

fn everyone_playing(clients: &mut [Client]) -> Vec<Someone> {
    clients
        .iter_mut()
        .filter(|client| client.phase == Phase::Playing && !client.is_gone())
        .map(|client| {
            let (level, position) = match &client.visit {
                Some(visit) => (Some((visit.id(), visit.index())), visit.position()),
                None => (None, client.character.world_position),
            };
            Someone {
                id: client.entity_id(),
                character: client.character_id,
                look: (client.inventory.creature, client.sheet.appearance.clone()),
                level,
                position,
                god_mode: client.god_mode,
            }
        })
        .collect()
}

/// How far apart, counting a diagonal as one.
fn cells_apart(a: (i64, i64), b: (i64, i64)) -> i64 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// The distance maps worked out this tick on one level: mobs after the same place
/// share one.
#[derive(Default)]
struct Maps(BTreeMap<(i64, i64), DistanceMap>);

impl Maps {
    fn to(&mut self, level: &Level, goal: (i64, i64)) -> &DistanceMap {
        self.0
            .entry(goal)
            .or_insert_with(|| DistanceMap::to(level, goal))
    }
}

/// A player on a level, as far as its mobs go.
struct Spot {
    id: Id,
    character: Option<CharacterId>,
    kind: CreatureKind,
    position: (i64, i64),
}

/// What a mob does about players now, if it should stop what it is doing.
fn notice(mob: &Mob, level: &Level, spots: &[Spot]) -> Option<Doing> {
    if matches!(
        mob.doing,
        Doing::Going { .. } | Doing::Following { .. } | Doing::Fleeing { .. }
    ) {
        return None;
    }
    spots
        .iter()
        .filter(|spot| {
            cells_apart(spot.position, mob.position) <= NOTICES_WITHIN
                && line_is_clear(level, mob.position, spot.position)
        })
        .find_map(|spot| {
            let going = |gap| Doing::Going {
                after: spot.id,
                last_seen: spot.position,
                gap,
            };
            match mob
                .kind
                .reaction(mob.feeling(spot.kind, spot.character.as_ref()))
            {
                Reaction::Ignore => None,
                Reaction::Flee => Some(Doing::Fleeing { from: spot.id }),
                Reaction::Approach => Some(going(FRIENDLY_GAP)),
                Reaction::Attack => Some(going(ATTACK_GAP)),
            }
        })
}

/// The cells someone is standing on: no mob can go on them.
type Taken = BTreeSet<(i64, i64)>;

/// Moves a mob by a step, if it can go. What it cost.
fn take_step(mob: &mut Mob, level: &Level, step: Option<Step>, taken: &Taken) -> Option<u32> {
    let to = step_on_level(level, mob.position, step?)?;
    if taken.contains(&to) {
        return None;
    }
    let cost = site_step_cost(mob.position, to);
    mob.position = to;
    Some(cost)
}

/// A direction for wandering, different for each mob and moment.
fn wandering_step(mob: &Mob, tick: u64) -> Option<Step> {
    let (high, low) = mob.id.0.as_u64_pair();
    let roll = Seed::new(high ^ low).at(&[tick as i64, mob.position.0, mob.position.1]);
    Step::new((roll % 3) as i64 - 1, (roll / 3 % 3) as i64 - 1)
}

/// One mob's turn: what it does and what it costs. Afterwards it may be at rest.
/// `seen` says whether any player can see a cell. `crossing` is filled in if it
/// is going to another level.
#[allow(clippy::too_many_arguments)]
fn act(
    mob: &mut Mob,
    level: &Level,
    spots: &[Spot],
    maps: &mut Maps,
    tick: u64,
    seen: impl Fn((i64, i64)) -> bool,
    crossing: &mut Option<Crossing>,
    taken: &Taken,
    (site, level_index): (SiteId, u32),
) -> u32 {
    let free = |cell: (i64, i64)| !taken.contains(&cell);
    let settle = Doing::Settling {
        steps_left: SETTLING_STEPS,
    };
    match mob.doing {
        Doing::Idle => 0,
        Doing::Going {
            after,
            last_seen,
            gap,
        } => {
            let target = spots
                .iter()
                .find(|spot| spot.id == after && line_is_clear(level, mob.position, spot.position));
            let goal = target.map_or(last_seen, |spot| spot.position);
            mob.doing = Doing::Going {
                after,
                last_seen: goal,
                gap,
            };
            let near = if target.is_some() { gap } else { 0 };
            if cells_apart(mob.position, goal) <= near {
                // Lost them, here where they were last seen.
                if target.is_none() {
                    mob.doing = settle;
                }
                return THINKING_COST;
            }
            let step = maps.to(level, goal).towards(level, mob.position, free);
            take_step(mob, level, step, taken).unwrap_or_else(|| {
                // No way there.
                mob.doing = settle;
                THINKING_COST
            })
        }
        Doing::Following {
            stairs,
            to_level,
            arrives,
            ..
        } => {
            if mob.position == stairs {
                *crossing = Some(Crossing {
                    mob: mob.id,
                    site,
                    from_level: level_index,
                    to_level,
                    from_stairs: stairs,
                    arrives,
                    going_back: false,
                });
                return shared::energy::CHANGE_MAP_COST;
            }
            let step = maps.to(level, stairs).towards(level, mob.position, free);
            take_step(mob, level, step, taken).unwrap_or_else(|| {
                mob.doing = settle;
                THINKING_COST
            })
        }
        Doing::Fleeing { from } => {
            let threat = spots
                .iter()
                .find(|spot| spot.id == from && line_is_clear(level, mob.position, spot.position));
            let Some(threat) = threat else {
                mob.doing = settle;
                return THINKING_COST;
            };
            let step = maps
                .to(level, threat.position)
                .away(level, mob.position, free);
            take_step(mob, level, step, taken).unwrap_or(THINKING_COST)
        }
        Doing::Settling { steps_left } => {
            mob.doing = match steps_left.checked_sub(1) {
                Some(steps_left) => Doing::Settling { steps_left },
                None => Doing::GoingHome { since: tick },
            };
            take_step(mob, level, wandering_step(mob, tick), taken).unwrap_or(THINKING_COST)
        }
        Doing::GoingHome { since } => {
            // Home, or the stairs leading back towards it.
            let hop = mob.way_back.last().copied();
            let goal = hop.map_or(mob.home, |hop| hop.stairs);
            let late = tick.saturating_sub(since) > HOME_WITHIN;
            let step = maps.to(level, goal).towards(level, mob.position, free);
            let walked = take_step(mob, level, step, taken);
            // Stuck or lost, it gets there when nobody can see.
            if (late || walked.is_none()) && !seen(mob.position) && !seen(goal) && free(goal) {
                mob.position = goal;
            }
            if mob.position == goal {
                match hop {
                    Some(hop) => {
                        *crossing = Some(Crossing {
                            mob: mob.id,
                            site,
                            from_level: level_index,
                            to_level: hop.to_level,
                            from_stairs: hop.stairs,
                            arrives: hop.arrives,
                            going_back: true,
                        });
                    }
                    None => mob.doing = Doing::Idle,
                }
            }
            walked.unwrap_or(THINKING_COST)
        }
    }
}

/// The nearest walkable cell to this one that isn't taken, not far off.
fn nearest_free(
    level: &Level,
    cell: (i64, i64),
    taken: impl Fn((i64, i64)) -> bool,
) -> Option<(i64, i64)> {
    const FURTHEST: i64 = 8;
    (1..=FURTHEST).find_map(|reach| {
        (-reach..=reach).find_map(|dy| {
            (-reach..=reach).find_map(|dx| {
                let spot = (cell.0 + dx, cell.1 + dy);
                let on_edge = dx.abs().max(dy.abs()) == reach;
                (on_edge && level.is_walkable(spot.0, spot.1) && !taken(spot)).then_some(spot)
            })
        })
    })
}

/// Takes a mob from one level to another. It stays where it is if there is
/// nowhere to go: it can only be somewhere nobody has been since, going forward,
/// as a level with its mobs away is kept.
fn cross(populations: &mut Populations, crossing: Crossing, tick: u64, players: &[Someone]) {
    let Crossing {
        mob,
        site,
        from_level,
        to_level,
        from_stairs,
        arrives,
        going_back,
    } = crossing;
    let settle = Doing::Settling {
        steps_left: SETTLING_STEPS,
    };
    let Some(from) = populations.0.get_mut(&(site, from_level)) else {
        return;
    };
    let Some(index) = from.mobs.iter().position(|one| one.id == mob) else {
        return;
    };
    if !populations.0.contains_key(&(site, to_level)) {
        let from = populations
            .0
            .get_mut(&(site, from_level))
            .expect("just used");
        from.mobs[index].doing = settle;
        from.due.insert((tick + 1, mob));
        return;
    }
    // Someone may be standing there: it arrives beside them, or if it can't, waits
    // at the stairs and tries again.
    let there = populations.0.get(&(site, to_level)).expect("just checked");
    let occupied = |spot: (i64, i64)| {
        there.mobs.iter().any(|one| one.position == spot)
            || players
                .iter()
                .any(|one| one.level == Some((site, to_level)) && one.position == spot)
    };
    let landing = if occupied(arrives) {
        nearest_free(&there.level, arrives, occupied)
    } else {
        Some(arrives)
    };
    let Some(landing) = landing else {
        let from = populations
            .0
            .get_mut(&(site, from_level))
            .expect("just used");
        from.due.insert((tick + 1, mob));
        return;
    };
    let from = populations
        .0
        .get_mut(&(site, from_level))
        .expect("just used");
    let mut moving = from.mobs.remove(index);
    from.due.retain(|&(_, id)| id != mob);
    let home = moving.home_level;
    if going_back {
        moving.way_back.pop();
    } else {
        moving.way_back.push(Hop {
            stairs: arrives,
            to_level: from_level,
            arrives: from_stairs,
        });
        if let Doing::Following { after, gap, .. } = moving.doing {
            moving.doing = Doing::Going {
                after,
                last_seen: arrives,
                gap,
            };
        }
    }
    moving.position = landing;
    let back_home = to_level == home && moving.way_back.is_empty();
    let left_home = from_level == home && !going_back && moving.way_back.len() == 1;
    let ready = moving.energy.ready_at(tick + 1, moving.speed);
    let there = populations
        .0
        .get_mut(&(site, to_level))
        .expect("checked above");
    there.due.insert((ready, moving.id));
    there.mobs.push(moving);
    if back_home {
        there.away = there.away.saturating_sub(1);
    }
    if left_home && let Some(home_level) = populations.0.get_mut(&(site, home)) {
        home_level.away += 1;
    }
}

/// Lets each level's mobs do what they are doing, as far as they have the energy.
pub(crate) fn run_mobs(clients: &mut [Client], world: &mut ServerWorld) {
    let tick = world.action_clock.ticks();
    let everyone = everyone_playing(clients);
    let mut crossings = Vec::new();
    for (&place, population) in &mut world.populations.0 {
        let here = || {
            everyone
                .iter()
                .filter(|someone| someone.level == Some(place))
        };
        // Those in god mode still stand somewhere and can see, but aren't noticed.
        let standing: Vec<(i64, i64)> = here().map(|someone| someone.position).collect();
        let spots: Vec<Spot> = here()
            .filter(|someone| !someone.god_mode)
            .map(|someone| Spot {
                id: someone.id,
                character: someone.character,
                kind: someone.look.0,
                position: someone.position,
            })
            .collect();
        let Population {
            mobs, level, due, ..
        } = population;

        // Anyone at rest or settling may notice someone.
        for mob in mobs.iter_mut() {
            if let Some(doing) = notice(mob, level, &spots) {
                let at_rest = mob.doing == Doing::Idle;
                mob.doing = doing;
                if at_rest {
                    due.insert((mob.energy.ready_at(tick, mob.speed), mob.id));
                }
            }
        }

        let mut taken: Taken = standing.iter().copied().collect();
        taken.extend(mobs.iter().map(|mob| mob.position));
        let mut maps = Maps::default();
        while let Some(&(when, id)) = due.first() {
            if when > tick {
                break;
            }
            due.remove(&(when, id));
            let Some(mob) = mobs.iter_mut().find(|mob| mob.id == id) else {
                continue;
            };
            let mut crossing = None;
            let seen = |cell| {
                standing
                    .iter()
                    .any(|&spot| line_is_clear(level, spot, cell))
            };
            let was_at = mob.position;
            let cost = act(
                mob,
                level,
                &spots,
                &mut maps,
                tick,
                seen,
                &mut crossing,
                &taken,
                place,
            );
            taken.remove(&was_at);
            taken.insert(mob.position);
            mob.energy.spend(tick, mob.speed, cost);
            if let Some(crossing) = crossing {
                crossings.push(crossing);
            } else if mob.doing != Doing::Idle {
                // At least the next tick, so one turn can't run for ever here.
                due.insert((mob.energy.ready_at(tick + 1, mob.speed), id));
            }
        }
    }
    for crossing in crossings {
        cross(&mut world.populations, crossing, tick, &everyone);
    }
}

/// How far apart on the world map, the nearer way round the planet.
fn world_gap(a: (i64, i64), b: (i64, i64), width: i64) -> i64 {
    let dx = (a.0 - b.0).rem_euclid(width);
    dx.min(width - dx).max((a.1 - b.1).abs())
}

/// Tells everyone playing who has come into sight, moved or gone, and lets go of
/// levels nobody has been on for a while, once they have settled.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld) {
    let playing = |client: &&mut Client| client.phase == Phase::Playing;
    let everyone = everyone_playing(clients);

    let now = Instant::now();
    let in_use: BTreeSet<_> = everyone
        .iter()
        .filter_map(|someone| someone.level)
        .collect();
    world.populations.0.retain(|level, population| {
        if in_use.contains(level) || !population.is_settled() {
            population.empty_since = None;
            return true;
        }
        now.duration_since(*population.empty_since.get_or_insert(now)) < KEPT_FOR
    });

    let width = world.width;
    for client in clients.iter_mut().filter(playing) {
        let here = client
            .visit
            .as_ref()
            .map(|visit| (visit.id(), visit.index()));
        let me = client.character.world_position;
        let my_kind = client.inventory.creature;
        let detect = client.detect_monsters;
        let mut seen: Vec<EntityLook> = Vec::new();
        for someone in &everyone {
            let in_sight = someone.id != client.entity_id()
                && someone.level == here
                && match &mut client.visit {
                    Some(visit) => visit.can_see(someone.position.0, someone.position.1),
                    None => world_gap(me, someone.position, width) <= WORLD_MAP_SIGHT,
                };
            if in_sight {
                seen.push(EntityLook {
                    id: someone.id,
                    kind: someone.look.0,
                    appearance: someone.look.1.clone(),
                    feeling: Feeling::None,
                    x: someone.position.0,
                    y: someone.position.1,
                });
            }
        }
        if let (Some(level), Some(visit)) = (here, &mut client.visit)
            && let Some(population) = world.populations.0.get(&level)
        {
            for mob in &population.mobs {
                if detect || visit.can_see(mob.position.0, mob.position.1) {
                    seen.push(EntityLook {
                        id: mob.id,
                        kind: mob.kind,
                        appearance: mob.appearance.clone(),
                        feeling: mob.feeling(my_kind, client.character_id.as_ref()),
                        x: mob.position.0,
                        y: mob.position.1,
                    });
                }
            }
        }
        tell(client, seen);
    }
}

/// Tells a client how what it can see differs from what it knew. A client that
/// has gone is noticed when its messages are next read.
fn tell(client: &mut Client, seen: Vec<EntityLook>) {
    let now_seen: BTreeSet<Id> = seen.iter().map(|look| look.id).collect();
    let feelings_changed: BTreeSet<Id> = seen
        .iter()
        .filter(|look| {
            client
                .seen_feelings
                .get(&look.id)
                .is_some_and(|known| *known != look.feeling)
        })
        .map(|look| look.id)
        .collect();
    client.seen_feelings = seen.iter().map(|look| (look.id, look.feeling)).collect();
    client.seen_entities.retain(|id, _| {
        let stays = now_seen.contains(id);
        if !stays {
            client.link.send(&ServerMessage::EntityGone { id: *id });
        }
        stays
    });
    for look in seen {
        let at = (look.x, look.y);
        let known = client.seen_entities.insert(look.id, at);
        // Told afresh, the token takes the new colour.
        let known = known.filter(|_| !feelings_changed.contains(&look.id));
        match known {
            None => {
                client.link.send(&ServerMessage::EntitySeen(look));
            }
            Some(known) if known != at => {
                client.link.send(&ServerMessage::EntityMoved {
                    id: look.id,
                    x: at.0,
                    y: at.1,
                });
            }
            Some(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use shared::poi::PoiKind;
    use shared::protocol::{ClientMessage, local_pair};
    use shared::world::WorldSeed;

    use super::*;
    use crate::connections::tests::{HELLO, TestClient};
    use crate::storage::Storage;
    use crate::storage::tests::TestFolder;

    const CASTLE: ClientMessage = ClientMessage::DevEnterSite {
        kind: PoiKind::Castle,
        seed: 7,
        level: 0,
        ruin: 0,
    };

    fn seen_ids(answers: &[ServerMessage]) -> BTreeSet<Id> {
        answers
            .iter()
            .filter_map(|answer| match answer {
                ServerMessage::EntitySeen(look) => Some(look.id),
                _ => None,
            })
            .collect()
    }

    /// The id saved for each of the level's mobs.
    fn mob_ids(test: &mut TestClient) -> BTreeSet<Id> {
        let id = SiteId::Dev {
            kind: PoiKind::Castle,
            seed: 7,
            ruin: 0,
        };
        let site = test.world.site(id, PoiKind::Castle);
        site.levels[&0].mobs.values().copied().collect()
    }

    #[test]
    fn only_mobs_in_sight_are_told_of_and_they_say_nothing_of_stats() {
        let mut test = TestClient::new();
        let answers = test.send(&[CASTLE]);
        let everyone = mob_ids(&mut test);
        assert!(!everyone.is_empty());
        let seen = seen_ids(&answers);
        assert!(seen.is_subset(&everyone));
        // A castle has more rooms than the first one in sight.
        assert!(seen.len() < everyone.len());
    }

    #[test]
    fn walking_shows_what_comes_into_sight_and_says_when_it_goes() {
        // Reveal the level for a route, and walk until something new is seen.
        let mut test = TestClient::new();
        test.send(&[CASTLE]);
        let everyone = mob_ids(&mut test);
        let mob = test.world.populations.0.values().next().unwrap().mobs[0].clone();
        let answers = test.send(&[ClientMessage::DevTeleport {
            x: mob.position.0,
            y: mob.position.1,
        }]);
        // Standing on it, it is in sight: a teleport starts afresh.
        assert!(seen_ids(&answers).contains(&mob.id));
        assert!(seen_ids(&answers).is_subset(&everyone));
        // Nothing has changed: nothing more is said.
        let answers = test.send(&[]);
        assert!(seen_ids(&answers).is_empty());
    }

    #[test]
    fn mobs_keep_their_ids_after_a_restart() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        test.send(&[CASTLE]);
        let before = mob_ids(&mut test);
        let world = test.leave();
        world.storage.finish_writing();
        drop(world);
        let mut test = TestClient::of(open());
        assert_eq!(mob_ids(&mut test), before);
    }

    #[test]
    fn a_forgotten_site_gets_new_mobs() {
        let mut test = TestClient::new();
        test.send(&[CASTLE]);
        let before = mob_ids(&mut test);
        test.send(&[ClientMessage::DevForgetMapOnServer]);
        assert!(mob_ids(&mut test).is_disjoint(&before));
    }

    /// Two clients of one world, through the real tick.
    struct Two {
        connections: crate::Connections,
        world: ServerWorld,
        ends: [shared::protocol::ClientLink; 2],
    }

    impl Two {
        fn new() -> Self {
            let mut connections = crate::Connections::default();
            let ends = [1, 2].map(|n| {
                let (end, link) = local_pair();
                connections.add_playing(link, crate::CharacterId(Id::for_test(n)));
                end.send(&HELLO);
                end
            });
            let mut two = Self {
                connections,
                world: ServerWorld::new(WorldSeed::new(1)),
                ends,
            };
            two.tick();
            two
        }

        fn tick(&mut self) -> [Vec<ServerMessage>; 2] {
            crate::connections::tests::tick_both(&mut self.connections, &mut self.world);
            [0, 1].map(|n| std::iter::from_fn(|| self.ends[n].receive().ok().flatten()).collect())
        }
    }

    #[test]
    fn players_on_the_world_map_see_each_other_and_their_moves() {
        let mut two = Two::new();
        // Both start at the same place: already told by the first tick.
        two.ends[0].send(&ClientMessage::DevTeleport { x: 3, y: 0 });
        let [_, second] = two.tick();
        assert!(
            second
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntityMoved { x: 3, y: 0, .. }))
        );
    }

    #[test]
    fn a_player_going_into_a_site_goes_out_of_sight_and_is_seen_inside() {
        let mut two = Two::new();
        two.ends[0].send(&CASTLE);
        let [_, second] = two.tick();
        assert!(
            second
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntityGone { .. }))
        );
        two.ends[1].send(&CASTLE);
        let [first, second] = two.tick();
        // In the same place, they see each other.
        assert!(
            second
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Human))
        );
        assert!(
            first
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Human))
        );
    }

    /// A client in a site, and a mob on its level made this kind, `apart` cells east
    /// of the player in the same room, so that they can see each other.
    fn beside_a_mob(kind: CreatureKind, apart: i64) -> TestClient {
        let mut test = TestClient::new();
        test.send(&[CASTLE]);
        let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(apart);
        test.send(&[ClientMessage::DevTeleport { x, y }]);
        let population = test.world.populations.0.values_mut().next().unwrap();
        let mob = &mut population.mobs[0];
        mob.kind = kind;
        mob.position = (x + apart, y);
        mob.home = (x + apart, y);
        test
    }

    /// Lets this many ticks go by, with mobs acting.
    fn run(test: &mut TestClient, ticks: u64) {
        for _ in 0..ticks {
            test.world.action_clock.advance(1);
            run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
        }
    }

    fn the_mob(test: &TestClient) -> Mob {
        test.world.populations.0.values().next().unwrap().mobs[0].clone()
    }

    fn player(test: &TestClient) -> (i64, i64) {
        test.client.visit.as_ref().unwrap().position()
    }

    #[test]
    fn mobs_at_rest_need_no_time_until_they_notice_someone() {
        let mut test = TestClient::new();
        test.send(&[CASTLE]);
        // Kinds that ignore the player, wherever they are.
        let population = test.world.populations.0.values_mut().next().unwrap();
        for mob in &mut population.mobs {
            mob.kind = CreatureKind::Horse;
        }
        run(&mut test, 20);
        let population = test.world.populations.0.values().next().unwrap();
        assert!(population.due.is_empty());
        assert!(population.is_settled());
    }

    #[test]
    fn something_unfriendly_runs_at_a_player_and_stops_beside_them() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        run(&mut test, 40);
        assert!(cells_apart(the_mob(&test).position, player(&test)) <= ATTACK_GAP);
        assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
    }

    #[test]
    fn something_timid_runs_away() {
        let mut test = beside_a_mob(CreatureKind::Chicken, 2);
        let started = the_mob(&test).position;
        run(&mut test, 10);
        let gap = cells_apart(the_mob(&test).position, player(&test));
        assert!(gap > cells_apart(started, player(&test)));
    }

    #[test]
    fn a_friendly_one_comes_near_and_keeps_its_distance() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        test.send(&[ClientMessage::DevSetCreature(CreatureKind::Kobold)]);
        run(&mut test, 40);
        assert_eq!(
            cells_apart(the_mob(&test).position, player(&test)),
            FRIENDLY_GAP
        );
    }

    #[test]
    fn with_nobody_left_a_mob_settles_goes_home_and_the_level_can_go() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        run(&mut test, 10);
        let home = the_mob(&test).home;
        assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));

        // The player leaves for the world map, and the level keeps going.
        test.send(&[ClientMessage::DevBackToStart]);
        let population = test.world.populations.0.values().next().unwrap();
        assert!(!population.is_settled());

        run(&mut test, HOME_WITHIN + 200);
        assert_eq!(the_mob(&test).doing, Doing::Idle);
        assert_eq!(the_mob(&test).position, home);
        let population = test.world.populations.0.values().next().unwrap();
        assert!(population.is_settled());
        assert!(population.due.is_empty());
    }

    #[test]
    fn nothing_moves_while_the_action_clock_stands_still() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        for _ in 0..50 {
            run_mobs(std::slice::from_mut(&mut test.client), &mut test.world);
        }
        // It may take the one step it had the energy for, and no more.
        let moved = cells_apart(the_mob(&test).position, the_mob(&test).home);
        assert!(moved <= 1, "{moved}");
    }

    #[test]
    fn the_token_says_how_the_mob_feels_about_this_player() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        // Told afresh: going away and back would do the same.
        test.client.seen_entities.clear();
        let answers = test.send(&[]);
        let feeling = answers.iter().find_map(|answer| match answer {
            ServerMessage::EntitySeen(look) if look.kind == CreatureKind::Kobold => {
                Some(look.feeling)
            }
            _ => None,
        });
        assert_eq!(feeling, Some(Feeling::Unfriendly));
    }

    /// `beside_a_mob`, with the player on the level's first stairs and the mob after
    /// them, having seen.
    fn being_chased_at_stairs(kind: CreatureKind) -> TestClient {
        let mut test = beside_a_mob(kind, 3);
        let visit = test.client.visit.as_ref().unwrap();
        let stairs = visit.first_stairs();
        test.send(&[ClientMessage::DevTeleport {
            x: stairs.0,
            y: stairs.1,
        }]);
        // Beside the stairs, so it sees them taken.
        let population = test.world.populations.0.values_mut().next().unwrap();
        let mob = &mut population.mobs[0];
        mob.position = stairs;
        mob.home = stairs;
        run(&mut test, 2);
        test
    }

    /// Which level each mob with this id is on.
    fn levels_with(test: &TestClient, id: Id) -> Vec<u32> {
        let levels = test.world.populations.0.iter();
        levels
            .filter(|(_, population)| population.mobs.iter().any(|mob| mob.id == id))
            .map(|(&(_, level), _)| level)
            .collect()
    }

    #[test]
    fn a_mob_that_sees_stairs_taken_follows_and_later_goes_home() {
        let mut test = being_chased_at_stairs(CreatureKind::Kobold);
        let mob = the_mob(&test).id;
        assert_eq!(levels_with(&test, mob), [0]);
        test.send(&[ClientMessage::UseExit]);
        let upstairs = test.client.visit.as_ref().unwrap().index();
        run(&mut test, 60);
        assert_eq!(levels_with(&test, mob), [upstairs]);

        // The player leaves. It loses them, wanders, and takes the stairs home;
        // its home level stays in use meanwhile.
        test.send(&[ClientMessage::DevBackToStart]);
        let home_level = test.world.populations.0.keys().find(|(_, l)| *l == 0);
        assert!(home_level.is_some());
        crate::entities::sync(std::slice::from_mut(&mut test.client), &mut test.world);
        run(&mut test, HOME_WITHIN + 400);
        assert_eq!(levels_with(&test, mob), [0]);
        let home = test.world.populations.0.iter().find(|((_, l), _)| *l == 0);
        let (_, home) = home.unwrap();
        assert_eq!(home.away, 0);
        assert!(home.is_settled());
    }

    #[test]
    fn a_kind_that_cant_use_stairs_stays_behind() {
        let mut test = being_chased_at_stairs(CreatureKind::Giant);
        // (A giant could, so this makes one that can't by its kind.)
        let (after, last_seen) = (test.client.entity_id(), player(&test));
        let population = test.world.populations.0.values_mut().next().unwrap();
        population.mobs[0].kind = CreatureKind::Horse;
        let mob = population.mobs[0].id;
        population.mobs[0].doing = Doing::Going {
            after,
            last_seen,
            gap: ATTACK_GAP,
        };
        test.send(&[ClientMessage::UseExit]);
        run(&mut test, 60);
        assert_eq!(levels_with(&test, mob), [0]);
    }

    #[test]
    fn a_mob_part_way_through_something_carries_on_after_a_restart() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        test.send(&[CASTLE]);
        let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
        test.send(&[ClientMessage::DevTeleport { x, y }]);
        let population = test.world.populations.0.values_mut().next().unwrap();
        let spot = (x + 2, y);
        let settling = Doing::Settling { steps_left: 4 };
        population.mobs[0].position = spot;
        population.mobs[0].doing = settling;
        test.world.save_mobs();
        let world = test.leave();
        world.storage.finish_writing();
        drop(world);

        let test = TestClient::of(open());
        let carried_on = the_mob(&test);
        assert_eq!(carried_on.position, spot);
        assert_eq!(carried_on.doing, settling);
        let population = test.world.populations.0.values().next().unwrap();
        assert!(!population.due.is_empty());
    }

    /// `beside_a_mob`, with something to give it.
    fn with_a_gift(kind: CreatureKind) -> (TestClient, Id) {
        let mut test = beside_a_mob(kind, 2);
        let answers = test.send(&[ClientMessage::DevGiveItem {
            kind: shared::item::ItemKind::Coin,
            size: None,
            magical: false,
            quality: Default::default(),
        }]);
        let gift = answers.iter().rev().find_map(|answer| match answer {
            ServerMessage::Inventory(carried) => Some(carried.pockets[0].item.id),
            _ => None,
        });
        (test, gift.expect("told of it"))
    }

    #[test]
    fn a_beggar_given_something_remembers_and_warms_to_the_giver() {
        let (mut test, coin) = with_a_gift(CreatureKind::Beggar);
        let beggar = the_mob(&test).id;
        run(&mut test, 3);
        // Neutral, and nothing to do about it, to begin with.
        assert_eq!(the_mob(&test).doing, Doing::Idle);

        let answers = test.send(&[ClientMessage::GiveItem {
            item: coin,
            to: beggar,
        }]);
        assert!(test.client.inventory.find(coin).is_none());
        // The token changes colour.
        let told = answers.iter().find_map(|answer| match answer {
            ServerMessage::EntitySeen(look) if look.id == beggar => Some(look.feeling),
            _ => None,
        });
        assert_eq!(told, Some(Feeling::Friendly));
        // And it comes along.
        run(&mut test, 3);
        assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
    }

    #[test]
    fn a_mob_is_dressed_once_and_keeps_its_things_after_a_restart() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        test.send(&[CASTLE]);
        let belongings = |test: &mut TestClient| -> Vec<(Id, Inventory)> {
            let (&(site, level), mobs) = test.world.populations.0.iter().next().unwrap();
            let kind = mobs.poi_kind;
            let ids: Vec<Id> = mobs.mobs.iter().map(|mob| mob.id).collect();
            let saved = &test.world.site(site, kind).levels[&level];
            let of = |id: Id| (id, saved.records[&id].gear.clone().unwrap());
            ids.into_iter().map(of).collect()
        };
        let before = belongings(&mut test);
        let wearers = before
            .iter()
            .filter(|(_, carrying)| !carrying.worn.is_empty());
        assert!(wearers.count() > 0, "someone is dressed: {before:?}");
        let armed = before.iter().any(|(_, carrying)| {
            let carried = carrying.pockets.iter().map(|placed| &placed.item);
            carried
                .chain(&carrying.worn)
                .any(|item| item.made_at.is_some())
        });
        assert!(armed);
        let world = test.leave();
        world.storage.finish_writing();
        drop(world);

        let mut test = TestClient::of(open());
        assert_eq!(belongings(&mut test), before);
    }

    #[test]
    fn what_a_mob_remembers_and_was_given_comes_back_after_a_restart() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(1);
        let open = || ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap();
        let mut test = TestClient::of(open());
        test.send(&[CASTLE]);
        let (x, y) = test.client.visit.as_ref().unwrap().room_to_walk_east(3);
        test.send(&[ClientMessage::DevTeleport { x, y }]);
        let population = test.world.populations.0.values_mut().next().unwrap();
        population.mobs[0].position = (x + 1, y);
        let mob = population.mobs[0].id;
        let answers = test.send(&[ClientMessage::DevGiveItem {
            kind: shared::item::ItemKind::Coin,
            size: None,
            magical: false,
            quality: Default::default(),
        }]);
        let coin = answers.iter().rev().find_map(|answer| match answer {
            ServerMessage::Inventory(carried) => Some(carried.pockets[0].item.id),
            _ => None,
        });
        test.send(&[ClientMessage::GiveItem {
            item: coin.unwrap(),
            to: mob,
        }]);
        let world = test.leave();
        world.storage.finish_writing();
        drop(world);

        let mut test = TestClient::of(open());
        let tester = test.client.character_id.unwrap();
        assert_eq!(the_mob(&test).id, mob);
        assert_eq!(the_mob(&test).memory.warmth_for(&tester), 30);
        let id = SiteId::Dev {
            kind: PoiKind::Castle,
            seed: 7,
            ruin: 0,
        };
        let record = &test.world.site(id, PoiKind::Castle).levels[&0].records[&mob];
        assert_eq!(record.carried.len(), 1);
    }

    #[test]
    fn nothing_can_be_given_from_too_far_or_that_isnt_carried() {
        let (mut test, coin) = with_a_gift(CreatureKind::Beggar);
        let beggar = the_mob(&test).id;
        // Not carried.
        test.send(&[ClientMessage::GiveItem {
            item: beggar,
            to: beggar,
        }]);
        assert_eq!(the_mob(&test).memory, Memory::default());
        // Too far.
        let population = test.world.populations.0.values_mut().next().unwrap();
        population.mobs[0].position.0 += GIVE_REACH + 1;
        test.send(&[ClientMessage::GiveItem {
            item: coin,
            to: beggar,
        }]);
        assert!(test.client.inventory.find(coin).is_some());
        assert_eq!(the_mob(&test).memory, Memory::default());
    }

    #[test]
    fn mobs_never_step_onto_each_other_or_a_player() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        // Two more running at the player from the same place.
        let population = test.world.populations.0.values_mut().next().unwrap();
        let spot = population.mobs[0].position;
        for (index, mob) in population.mobs.iter_mut().enumerate().take(3).skip(1) {
            mob.kind = CreatureKind::Kobold;
            mob.position = (spot.0, spot.1 + index as i64 - 1);
            mob.home = mob.position;
        }
        for _ in 0..40 {
            run(&mut test, 1);
            let population = test.world.populations.0.values().next().unwrap();
            let mut places: Vec<_> = population.mobs.iter().map(|mob| mob.position).collect();
            places.push(player(&test));
            let all = places.len();
            places.sort();
            places.dedup();
            assert_eq!(places.len(), all, "two on one cell");
        }
    }

    #[test]
    fn a_player_swaps_with_someone_friendly_but_not_with_an_enemy() {
        let east = (1, 0);
        let mut test = beside_a_mob(CreatureKind::Horse, 1);
        let from = player(&test);
        test.steps(&[east]);
        assert_eq!(player(&test), (from.0 + 1, from.1));
        assert_eq!(the_mob(&test).position, from);

        let mut test = beside_a_mob(CreatureKind::Kobold, 1);
        let from = player(&test);
        let answers = test.steps(&[east]);
        assert_eq!(player(&test), from);
        assert!(
            answers
                .iter()
                .any(|answer| matches!(answer, ServerMessage::Position { .. }))
        );
        // Given enough, it stops being an enemy and moves over.
        let mob = the_mob(&test).id;
        let tester = test.client.character_id.unwrap();
        let population = test.world.populations.0.values_mut().next().unwrap();
        let event = Event::Gave(shared::item::ItemKind::Coin);
        population.mobs[0].memory.remember(tester, event);
        assert_eq!(the_mob(&test).id, mob);
        test.steps(&[east]);
        assert_eq!(player(&test), (from.0 + 1, from.1));
    }

    #[test]
    fn a_mob_on_the_cell_someone_arrives_on_moves_aside() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        let spot = the_mob(&test).position;
        test.send(&[ClientMessage::DevTeleport {
            x: spot.0,
            y: spot.1,
        }]);
        assert_eq!(player(&test), spot);
        assert_ne!(the_mob(&test).position, spot);
    }

    #[test]
    fn in_god_mode_mobs_take_no_notice() {
        for kind in [
            CreatureKind::Kobold,
            CreatureKind::Chicken,
            CreatureKind::Beggar,
        ] {
            let mut test = beside_a_mob(kind, 3);
            test.send(&[ClientMessage::DevGodMode(true)]);
            let started = the_mob(&test).position;
            run(&mut test, 40);
            assert_eq!(the_mob(&test).doing, Doing::Idle);
            assert_eq!(the_mob(&test).position, started);
        }
    }

    #[test]
    fn a_chase_ends_when_god_mode_is_turned_on() {
        let mut test = beside_a_mob(CreatureKind::Kobold, 3);
        run(&mut test, 40);
        assert!(matches!(the_mob(&test).doing, Doing::Going { .. }));
        test.send(&[ClientMessage::DevGodMode(true)]);
        run(&mut test, 40);
        assert!(!matches!(the_mob(&test).doing, Doing::Going { .. }));
        assert_ne!(the_mob(&test).position, player(&test));
    }

    #[test]
    fn in_god_mode_a_player_walks_through_an_enemy() {
        let east = (1, 0);
        let mut test = beside_a_mob(CreatureKind::Kobold, 1);
        test.send(&[ClientMessage::DevGodMode(true)]);
        let from = player(&test);
        test.steps(&[east]);
        assert_eq!(player(&test), (from.0 + 1, from.1));
        assert_eq!(the_mob(&test).position, from);

        // Turned off, it is in the way again.
        test.send(&[ClientMessage::DevGodMode(false)]);
        let from = player(&test);
        test.steps(&[(-1, 0)]);
        assert_eq!(player(&test), from);
    }

    #[test]
    fn god_mode_needs_dev_tools() {
        let mut test = TestClient::new();
        test.dev_tools = false;
        test.send(&[ClientMessage::DevGodMode(true)]);
        assert!(!test.client.god_mode);
    }

    #[test]
    fn detecting_shows_mobs_out_of_sight() {
        let mut test = TestClient::new();
        let answers = test.send(&[CASTLE]);
        let everyone = mob_ids(&mut test);
        assert!(seen_ids(&answers).len() < everyone.len());
        let answers = test.send(&[ClientMessage::DevDetectMonsters(true)]);
        let found = seen_ids(&answers);
        assert!(!found.is_empty());
        assert_eq!(test.client.seen_entities.len(), everyone.len());
        // Turned off, those out of sight go.
        let answers = test.send(&[ClientMessage::DevDetectMonsters(false)]);
        assert!(
            answers
                .iter()
                .any(|answer| matches!(answer, ServerMessage::EntityGone { .. }))
        );
    }
}
