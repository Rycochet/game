//! Who else is on a map: the mobs on site levels in use, shared by everyone on
//! them, what those mobs are doing, and telling each player who they can see.
//!
//! A level is kept while someone is there, for a while after, so that someone
//! reconnecting finds the same mobs, and while any mob on it isn't settled, so that
//! nobody can leave a chasing mob frozen at the stairs for when they come back.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use shared::creature::CreatureKind;
use shared::dungeon::{Level, line_is_clear};
use shared::energy::{Energy, SITE_SCALE, site_step_cost};
use shared::id::Id;
use shared::item::Inventory;
use shared::mob::{EntityLook, spawns};
use shared::movement::{STRAIGHT_COST, Step, step_on_level};
use shared::pathfinding::DistanceMap;
use shared::poi::Poi;
use shared::protocol::{ServerMessage, WORLD_MAP_SIGHT};
use shared::rng::Seed;
use shared::sheet::{Appearance, Feeling, Reaction, Sheet};
use shared::site::SiteLevel;

use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::ids::new_id;
use crate::storage::SiteId;

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Doing {
    /// At rest, and needing no time until something wakes it.
    Idle,
    /// Going after someone, stopping this near.
    Going {
        after: Id,
        last_seen: (i64, i64),
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
    energy: Energy,
    speed: u32,
}

/// The mobs of one level, and how long nobody has been there.
#[derive(Debug)]
pub(crate) struct Population {
    pub(crate) mobs: Vec<Mob>,
    /// Its shape, to find ways about.
    level: Level,
    /// Who is due to act: the tick, and which mob. A mob at rest isn't here, so
    /// costs nothing however long it rests.
    due: BTreeSet<(u64, usize)>,
    empty_since: Option<Instant>,
}

impl Population {
    /// Whether every mob is at rest, so nothing is going on here.
    fn is_settled(&self) -> bool {
        self.mobs.iter().all(|mob| mob.doing == Doing::Idle)
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
        let saved = self.site_mut(id, poi.kind).levels.entry(index).or_default();
        let mobs = plan
            .into_iter()
            .map(|spawn| {
                let carrying = Inventory {
                    creature: spawn.kind,
                    ..Inventory::default()
                };
                Mob {
                    id: *saved.mobs.entry(spawn.number).or_insert_with(new_id),
                    kind: spawn.kind,
                    appearance: spawn.appearance,
                    position: spawn.position,
                    home: spawn.position,
                    doing: Doing::Idle,
                    energy: Energy::default(),
                    speed: Sheet::new(spawn.kind).speed(&carrying),
                }
            })
            .collect();
        let population = Population {
            mobs,
            level: level.level.clone(),
            due: BTreeSet::new(),
            empty_since: None,
        };
        self.populations.0.insert((id, index), population);
    }
}

/// A player as others see them.
struct Someone {
    id: Id,
    look: (CreatureKind, Appearance),
    /// The site level they are on, if any.
    level: Option<(SiteId, u32)>,
    position: (i64, i64),
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
                id: client.entity_id,
                look: (client.inventory.creature, client.sheet.appearance.clone()),
                level,
                position,
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
    kind: CreatureKind,
    position: (i64, i64),
}

/// What a mob does about players now, if it should stop what it is doing.
fn notice(mob: &Mob, level: &Level, spots: &[Spot]) -> Option<Doing> {
    if matches!(mob.doing, Doing::Going { .. } | Doing::Fleeing { .. }) {
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
            match mob.kind.reaction(mob.kind.feeling_about(spot.kind)) {
                Reaction::Ignore => None,
                Reaction::Flee => Some(Doing::Fleeing { from: spot.id }),
                Reaction::Approach => Some(going(FRIENDLY_GAP)),
                Reaction::Attack => Some(going(ATTACK_GAP)),
            }
        })
}

/// Moves a mob by a step, if it can go. What it cost.
fn take_step(mob: &mut Mob, level: &Level, step: Option<Step>) -> Option<u32> {
    let to = step_on_level(level, mob.position, step?)?;
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
/// `seen` says whether any player can see a cell.
fn act(
    mob: &mut Mob,
    level: &Level,
    spots: &[Spot],
    maps: &mut Maps,
    tick: u64,
    seen: impl Fn((i64, i64)) -> bool,
) -> u32 {
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
            let step = maps.to(level, goal).towards(level, mob.position);
            take_step(mob, level, step).unwrap_or_else(|| {
                // No way there.
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
            let step = maps.to(level, threat.position).away(level, mob.position);
            take_step(mob, level, step).unwrap_or(THINKING_COST)
        }
        Doing::Settling { steps_left } => {
            mob.doing = match steps_left.checked_sub(1) {
                Some(steps_left) => Doing::Settling { steps_left },
                None => Doing::GoingHome { since: tick },
            };
            take_step(mob, level, wandering_step(mob, tick)).unwrap_or(THINKING_COST)
        }
        Doing::GoingHome { since } => {
            let late = tick.saturating_sub(since) > HOME_WITHIN;
            let step = maps.to(level, mob.home).towards(level, mob.position);
            let walked = take_step(mob, level, step);
            // Stuck or lost, it gets there when nobody can see.
            if (late || walked.is_none()) && !seen(mob.position) && !seen(mob.home) {
                mob.position = mob.home;
            }
            if mob.position == mob.home {
                mob.doing = Doing::Idle;
            }
            walked.unwrap_or(THINKING_COST)
        }
    }
}

/// Lets each level's mobs do what they are doing, as far as they have the energy.
pub(crate) fn run_mobs(clients: &mut [Client], world: &mut ServerWorld) {
    let tick = world.action_clock.ticks();
    let everyone = everyone_playing(clients);
    for (&place, population) in &mut world.populations.0 {
        let spots: Vec<Spot> = everyone
            .iter()
            .filter(|someone| someone.level == Some(place))
            .map(|someone| Spot {
                id: someone.id,
                kind: someone.look.0,
                position: someone.position,
            })
            .collect();
        let Population {
            mobs, level, due, ..
        } = population;

        // Anyone at rest or settling may notice someone.
        for (index, mob) in mobs.iter_mut().enumerate() {
            if let Some(doing) = notice(mob, level, &spots) {
                let at_rest = mob.doing == Doing::Idle;
                mob.doing = doing;
                if at_rest {
                    due.insert((mob.energy.ready_at(tick, mob.speed), index));
                }
            }
        }

        let mut maps = Maps::default();
        while let Some(&(when, index)) = due.first() {
            if when > tick {
                break;
            }
            due.remove(&(when, index));
            let mob = &mut mobs[index];
            let cost = act(mob, level, &spots, &mut maps, tick, |cell| {
                spots
                    .iter()
                    .any(|spot| line_is_clear(level, spot.position, cell))
            });
            mob.energy.spend(tick, mob.speed, cost);
            if mob.doing != Doing::Idle {
                // At least the next tick, so one turn can't run for ever here.
                due.insert((mob.energy.ready_at(tick + 1, mob.speed), index));
            }
        }
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
        let mut seen: Vec<EntityLook> = Vec::new();
        for someone in &everyone {
            let in_sight = someone.id != client.entity_id
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
                if visit.can_see(mob.position.0, mob.position.1) {
                    seen.push(EntityLook {
                        id: mob.id,
                        kind: mob.kind,
                        appearance: mob.appearance.clone(),
                        feeling: mob.kind.feeling_about(my_kind),
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
    client.seen_entities.retain(|id, _| {
        let stays = now_seen.contains(id);
        if !stays {
            client.link.send(&ServerMessage::EntityGone { id: *id });
        }
        stays
    });
    for look in seen {
        let at = (look.x, look.y);
        match client.seen_entities.insert(look.id, at) {
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
            let ends = ["first", "second"].map(|name| {
                let (end, link) = local_pair();
                connections.add_playing(link, crate::CharacterId::new(name).unwrap());
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
}
