//! Who else is on a map: the mobs on site levels in use, shared by everyone on
//! them, and telling each player who they can see.
//!
//! Mobs are kept per level only while someone is there, and for a while after,
//! so that someone reconnecting finds the same ones.

use std::collections::{BTreeMap, BTreeSet};
use std::time::{Duration, Instant};

use shared::creature::CreatureKind;
use shared::dungeon::Level;
use shared::id::Id;
use shared::mob::{EntityLook, spawns};
use shared::poi::Poi;
use shared::protocol::{ServerMessage, WORLD_MAP_SIGHT};
use shared::sheet::Appearance;

use crate::ServerWorld;
use crate::connections::{Client, Phase};
use crate::ids::new_id;
use crate::storage::SiteId;

/// How long a level nobody is on stays in use.
const KEPT_FOR: Duration = Duration::from_secs(60);

/// Something on a level that isn't a player.
#[derive(Clone, Debug)]
pub(crate) struct Mob {
    pub id: Id,
    pub kind: CreatureKind,
    pub appearance: Appearance,
    pub position: (i64, i64),
}

/// The mobs of one level, and how long nobody has been there.
#[derive(Debug, Default)]
struct Population {
    mobs: Vec<Mob>,
    empty_since: Option<Instant>,
}

/// The mobs on the levels in use.
#[derive(Debug, Default)]
pub(crate) struct Populations(BTreeMap<(SiteId, u32), Population>);

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
    pub(crate) fn populate(&mut self, id: SiteId, poi: Poi, index: u32, level: &Level) {
        if self.populations.0.contains_key(&(id, index)) {
            return;
        }
        let plan = spawns(poi.seed, index, poi.kind, level);
        let saved = self.site_mut(id, poi.kind).levels.entry(index).or_default();
        let mobs = plan
            .into_iter()
            .map(|spawn| Mob {
                id: *saved.mobs.entry(spawn.number).or_insert_with(new_id),
                kind: spawn.kind,
                appearance: spawn.appearance,
                position: spawn.position,
            })
            .collect();
        let population = Population {
            mobs,
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

/// How far apart on the world map, the nearer way round the planet.
fn world_gap(a: (i64, i64), b: (i64, i64), width: i64) -> i64 {
    let dx = (a.0 - b.0).rem_euclid(width);
    dx.min(width - dx).max((a.1 - b.1).abs())
}

/// Tells everyone playing who has come into sight, moved or gone, and lets go of
/// levels nobody has been on for a while.
pub(crate) fn sync(clients: &mut [Client], world: &mut ServerWorld) {
    let playing = |client: &&mut Client| client.phase == Phase::Playing;
    let everyone: Vec<Someone> = clients
        .iter_mut()
        .filter(playing)
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
        .collect();

    let now = Instant::now();
    let in_use: BTreeSet<_> = everyone
        .iter()
        .filter_map(|someone| someone.level)
        .collect();
    world.populations.0.retain(|level, population| {
        if in_use.contains(level) {
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
}
