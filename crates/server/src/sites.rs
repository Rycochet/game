//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to). What a player has seen is kept with their
//! character; each site keeps its generator version, when its levels were
//! last visited and players' changes (`storage::SiteSave`).

use std::collections::VecDeque;

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::feature::{ChunkPos, Feature, FeatureKind};
use shared::movement::{Step, step_on_level};
use shared::protocol::{
    ENTER_REACH, LevelChunk, LevelInfo, SeenStairs, ServerMessage, level_chunk,
};
use shared::rng::Seed;
use shared::site::{Exit, Site, SiteLevel, leads_out};

use crate::connections::Client;
use crate::storage::{CellMasks, SiteId, VisitSave, now};
use crate::world_map::look_around;
use crate::{LEVEL_PRELOAD_STEPS, ServerWorld};

/// A player's stay on a site level.
pub(crate) struct Visit {
    /// The feature the site is behind.
    feature: Feature,
    id: SiteId,
    site: Site,
    index: u32,
    level: SiteLevel,
    /// Its stairs as players see them.
    stairs: Vec<SeenStairs>,
    /// Where the player is.
    position: (i64, i64),
    /// The cells the client has been sent on this visit.
    sent: CellMasks,
}

impl Visit {
    /// Where the player is on the level.
    pub(crate) fn position(&self) -> (i64, i64) {
        self.position
    }

    /// What is saved with the character.
    pub(crate) fn save(&self) -> VisitSave {
        VisitSave {
            site: self.id,
            level: self.index,
            position: self.position,
        }
    }
}

/// Where on a level a player arrives.
enum Arrival {
    /// Coming from the world map or another level.
    From(Exit),
    /// Back where they were (a saved visit), if it is still walkable.
    At((i64, i64)),
}

/// Enters the site of the feature in `chunk`, if the player is at it.
pub(crate) fn enter_feature(client: &mut Client, world: &mut ServerWorld, chunk: ChunkPos) -> bool {
    let Some(feature) = world.features.in_chunk(chunk) else {
        warn!("A client tried to enter {chunk:?}, which has no feature");
        return true;
    };
    let (x, y) = client.world_position();
    // East-west, the nearer way round the planet.
    let dx = (feature.x - x).rem_euclid(world.width);
    let near = dx.min(world.width - dx) <= ENTER_REACH && (feature.y - y).abs() <= ENTER_REACH;
    if client.visit.is_some() || !near {
        warn!("A client tried to enter {chunk:?} from elsewhere");
        return true;
    }
    client.character.features.entry(chunk).or_default().visited = true;
    client.unsaved = true;
    let id = SiteId::Feature(chunk);
    enter_site(client, world, feature, id, 0, Arrival::From(Exit::WorldMap))
}

/// Developer tools: enters a made-up site at any level, where the player
/// stands on the world map.
pub(crate) fn dev_enter(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: FeatureKind,
    seed: u64,
    level: u32,
) -> bool {
    leave(client, world);
    client.visit = None;
    let feature = dev_feature(client, kind, seed);
    let id = SiteId::Dev { kind, seed };
    enter_site(
        client,
        world,
        feature,
        id,
        level,
        Arrival::From(Exit::WorldMap),
    )
}

/// The made-up feature of a dev tools site: where the player stands.
fn dev_feature(client: &Client, kind: FeatureKind, seed: u64) -> Feature {
    let (x, y) = client.world_position();
    Feature {
        kind,
        chunk: ChunkPos { x: 0, y: 0 },
        x,
        y,
        seed: Seed::new(seed),
    }
}

/// A client coming back: puts the player back on the level they were on
/// when they left, if it is still there.
pub(crate) fn resume(client: &mut Client, world: &mut ServerWorld, saved: VisitSave) -> bool {
    let feature = match saved.site {
        SiteId::Feature(chunk) => world.features.in_chunk(chunk),
        SiteId::Dev { kind, seed } => Some(dev_feature(client, kind, seed)),
    };
    let Some(feature) = feature else {
        warn!("A character was in {:?}, which isn't there", saved.site);
        client.unsaved = true;
        return true;
    };
    let at = Arrival::At(saved.position);
    enter_site(client, world, feature, saved.site, saved.level, at)
}

/// Generates the site (with the generator version it was first made with)
/// and puts the player on level `index` (or its last level, if there are
/// fewer).
fn enter_site(
    client: &mut Client,
    world: &mut ServerWorld,
    feature: Feature,
    id: SiteId,
    index: u32,
    arrival: Arrival,
) -> bool {
    let version = world.site(id, feature.kind).version;
    let Some(site) = Site::with_version(feature.kind, feature.seed, version) else {
        error!("{id:?} needs generator version {version}, which this server can't run");
        return true;
    };
    let index = index.min(site.levels() - 1);
    enter_level(client, world, feature, id, site, index, arrival)
}

/// The player takes a step on their level: taken if it leads onto a
/// walkable cell (otherwise `Position` puts the client right), then sends
/// what they now see or are about to.
pub(crate) fn stepped(client: &mut Client, step: Step) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let Some(position) = step_on_level(&visit.level.level, visit.position, step) else {
        let (x, y) = step.from(visit.position);
        info!("Refused a step onto an unwalkable cell ({x}, {y})");
        return true;
    };
    visit.position = position;
    client.unsaved = true;
    update_sight(client)
}

/// Developer tools: puts the player on `(x, y)` on their level, if it is
/// walkable (through walls, unlike a step).
pub(crate) fn teleport(client: &mut Client, x: i64, y: i64) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    if !visit.level.level.is_walkable(x, y) {
        info!("Refused a teleport onto an unwalkable cell ({x}, {y})");
        return true;
    }
    visit.position = (x, y);
    client.unsaved = true;
    client.changed_map();
    client.link.send(&ServerMessage::Teleported { x, y }) && update_sight(client)
}

/// Uses what the player stands on: stairs to another level, or ground on
/// the edge of the level back to the world map.
pub(crate) fn use_exit(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let (x, y) = visit.position;
    let stairs = visit.level.stairs.iter().find(|s| (s.x, s.y) == (x, y));
    let exit = match stairs {
        Some(stairs) => stairs.to,
        None if leads_out(&visit.level.level, x, y) => Exit::WorldMap,
        None => return true,
    };
    leave(client, world);
    let visit = client.visit.take().expect("checked above");
    client.unsaved = true;
    match exit {
        Exit::Level(index) => {
            let from = Arrival::From(Exit::Level(visit.index));
            enter_level(
                client,
                world,
                visit.feature,
                visit.id,
                visit.site,
                index,
                from,
            )
        }
        Exit::WorldMap => {
            let (x, y) = (visit.feature.x, visit.feature.y);
            client.character.world_position = (x, y);
            client.changed_map();
            client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
        }
    }
}

/// The player is leaving their level (or the server): it was last visited
/// now. The visit itself is left alone (saved with the character).
pub(crate) fn leave(client: &Client, world: &mut ServerWorld) {
    if let Some(visit) = &client.visit {
        let site = world.site_mut(visit.id, visit.feature.kind);
        site.levels.entry(visit.index).or_default().last_visit = now();
    }
}

/// Puts the player on level `index` of `site`: generates it, with players'
/// changes over it, tells the client about it, sends what the player
/// remembers of it, then what they see.
fn enter_level(
    client: &mut Client,
    world: &mut ServerWorld,
    feature: Feature,
    id: SiteId,
    site: Site,
    index: u32,
    arrival: Arrival,
) -> bool {
    let mut level = site.level(index);
    let saved = world
        .site_mut(id, feature.kind)
        .levels
        .entry(index)
        .or_default();
    saved.last_visit = now();
    for (&(x, y), &cell) in &saved.changes {
        level.level.set(x, y, cell);
    }
    let stairs = level
        .stairs
        .iter()
        .map(|&stairs| SeenStairs {
            x: stairs.x,
            y: stairs.y,
            direction: stairs.direction,
            // Only for looks: the other level is generated without changes.
            descends: site.stairs_descend(index, &level.level, stairs),
        })
        .collect();
    let position = match arrival {
        Arrival::At((x, y)) if level.level.is_walkable(x, y) => (x, y),
        Arrival::At(_) => level.arrive(Exit::WorldMap),
        Arrival::From(from) => level.arrive(from),
    };
    let info = LevelInfo {
        kind: site.kind(),
        building: site.is_building(),
        index,
        levels: site.levels(),
        bounds: level.level.bounds(),
        arrival: position,
    };
    let mut visit = Visit {
        feature,
        id,
        site,
        index,
        level,
        stairs,
        position,
        sent: CellMasks::new(),
    };
    client.changed_map();
    if !client.link.send(&ServerMessage::EnterLevel(info)) {
        return false;
    }
    let remembered = memory(client, &visit).clone();
    if !send_cells(client, &visit, &remembered, true) {
        return false;
    }
    visit.sent = remembered;
    client.visit = Some(visit);
    client.unsaved = true;
    update_sight(client)
}

/// What the character remembers of the visit's level.
fn memory<'a>(client: &'a mut Client, visit: &Visit) -> &'a mut CellMasks {
    let site = client.character.memory.entry(visit.id).or_default();
    site.entry(visit.index).or_default()
}

/// Works out what the player sees (remembered from now on) and what they
/// could see within `LEVEL_PRELOAD_STEPS`, and sends the cells of that the
/// client doesn't have yet.
fn update_sight(client: &mut Client) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let level = &visit.level.level;
    let (x, y) = visit.position;
    // What the client draws: glimpses, the more generous sight.
    let mut seen = Sight::new(level);
    seen.add_glimpses(level, x, y, None);
    let mut loaded = Sight::new(level);
    for (nx, ny) in nearby_cells(level, (x, y), LEVEL_PRELOAD_STEPS) {
        loaded.add_glimpses(level, nx, ny, None);
    }

    let memory = memory(client, &visit);
    let mut learnt = false;
    let mut new = CellMasks::new();
    let (min_x, min_y, width, height) = level.bounds();
    for cy in min_y..min_y + height {
        for cx in min_x..min_x + width {
            let (chunk, bit) = level_chunk(cx, cy);
            if seen.is_seen(cx, cy) {
                let mask = memory.entry(chunk).or_default();
                learnt |= *mask & (1 << bit) == 0;
                *mask |= 1 << bit;
            }
            let sent = visit.sent.get(&chunk).copied().unwrap_or(0);
            if loaded.is_seen(cx, cy) && sent & (1 << bit) == 0 {
                *new.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    client.unsaved |= learnt;
    let sent = send_cells(client, &visit, &new, false);
    let visit = client.visit.insert(visit);
    for (chunk, mask) in new {
        *visit.sent.entry(chunk).or_default() |= mask;
    }
    sent
}

/// The walkable cells the player could reach within `steps` steps (in any
/// of 8 directions), including where they stand.
fn nearby_cells(level: &Level, from: (i64, i64), steps: i64) -> Vec<(i64, i64)> {
    let mut found = vec![from];
    let mut queue = VecDeque::from([(from, 0)]);
    while let Some(((x, y), distance)) = queue.pop_front() {
        if distance == steps {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let next = (x + dx, y + dy);
                if level.is_walkable(next.0, next.1) && !found.contains(&next) {
                    found.push(next);
                    queue.push_back((next, distance + 1));
                }
            }
        }
    }
    found
}

/// Sends the cells in `masks` as the player would see them, a chunk at a
/// time, with any stairs on them.
fn send_cells(client: &Client, visit: &Visit, masks: &CellMasks, remembered: bool) -> bool {
    let level = &visit.level.level;
    for (&chunk, &mask) in masks {
        let message = LevelChunk {
            chunk,
            mask,
            cells: Vec::new(),
            stairs: Vec::new(),
            remembered,
        };
        // The positions come from the mask; fill in what is there.
        let positions: Vec<(i64, i64)> = message.positions().collect();
        let cells = positions
            .iter()
            .map(|&(x, y)| match level.cell(x, y) {
                // Hidden until found.
                Cell::SecretDoor => Cell::Wall,
                cell => cell,
            })
            .collect();
        let stairs = visit
            .stairs
            .iter()
            .filter(|s| positions.contains(&(s.x, s.y)))
            .copied()
            .collect();
        let message = LevelChunk {
            cells,
            stairs,
            ..message
        };
        if !client.link.send(&ServerMessage::LevelCells(message)) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use shared::protocol::{ClientMessage, ServerMessage};

    use shared::world::WorldSeed;

    use super::*;
    use crate::connections::tests::TestClient;
    use crate::storage::Storage;
    use crate::storage::tests::TestFolder;

    /// A castle with more than one floor.
    fn castle_seed() -> u64 {
        (1..)
            .find(|&seed| Site::new(FeatureKind::Castle, Seed::new(seed)).levels() > 1)
            .unwrap()
    }

    /// Enters a test site (a castle with stairs) and returns the answers.
    fn enter(test: &mut TestClient, level: u32) -> Vec<ServerMessage> {
        test.send(&[ClientMessage::DevEnterSite {
            kind: FeatureKind::Castle,
            seed: castle_seed(),
            level,
        }])
    }

    /// The steps of a shortest walk from `from` to `to` on `level`.
    fn path(level: &Level, from: (i64, i64), to: (i64, i64)) -> Vec<(i64, i64)> {
        let mut came_from = std::collections::BTreeMap::from([(from, from)]);
        let mut queue = VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            if at == to {
                break;
            }
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let Some(step) = Step::new(dx, dy) else {
                        continue;
                    };
                    if let Some(next) = step_on_level(level, at, step)
                        && !came_from.contains_key(&next)
                    {
                        came_from.insert(next, at);
                        queue.push_back(next);
                    }
                }
            }
        }
        let mut steps = Vec::new();
        let mut at = to;
        while at != from {
            let back = came_from[&at];
            steps.push((at.0 - back.0, at.1 - back.1));
            at = back;
        }
        steps.reverse();
        steps
    }

    /// Walks the player to `to` on their level.
    fn walk(test: &mut TestClient, to: (i64, i64)) -> Vec<ServerMessage> {
        let visit = test.client.visit.as_ref().unwrap();
        let steps = path(&visit.level.level, visit.position, to);
        test.steps(&steps)
    }

    /// A walkable cell next to the player, and the step there.
    fn walkable_neighbour(test: &TestClient) -> (i64, i64) {
        let visit = test.client.visit.as_ref().unwrap();
        [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ]
        .into_iter()
        .find(|&(dx, dy)| {
            let step = Step::new(dx, dy).unwrap();
            step_on_level(&visit.level.level, visit.position, step).is_some()
        })
        .expect("room to move")
    }

    fn cells(answers: &[ServerMessage]) -> Vec<(i64, i64, Cell, bool)> {
        answers
            .iter()
            .filter_map(|answer| match answer {
                ServerMessage::LevelCells(chunk) => Some(chunk),
                _ => None,
            })
            .flat_map(|chunk| {
                chunk
                    .cells()
                    .map(|(x, y, cell)| (x, y, cell, chunk.remembered))
            })
            .collect()
    }

    #[test]
    fn entering_sends_the_level_then_what_is_in_sight() {
        let mut test = TestClient::new();
        let answers = enter(&mut test, 0);
        let ServerMessage::EnterLevel(info) = &answers[0] else {
            panic!("{:?}", answers[0]);
        };
        let sent = cells(&answers);
        assert!(!sent.is_empty());
        // The cell the player arrives on, and nothing remembered yet.
        let (ax, ay) = info.arrival;
        assert!(sent.iter().any(|&(x, y, _, _)| (x, y) == (ax, ay)));
        assert!(sent.iter().all(|&(_, _, _, remembered)| !remembered));
        // Never the whole level, and never a secret door.
        let (_, _, width, height) = info.bounds;
        assert!(sent.len() < (width * height) as usize);
        assert!(sent.iter().all(|&(_, _, cell, _)| cell != Cell::SecretDoor));
    }

    #[test]
    fn cells_are_sent_once_and_remembered_next_time() {
        let mut test = TestClient::new();
        let first = cells(&enter(&mut test, 0));
        // A step there and back: back where everything was sent already.
        let (dx, dy) = walkable_neighbour(&test);
        test.steps(&[(dx, dy)]);
        assert!(cells(&test.steps(&[(-dx, -dy)])).is_empty());
        // Coming back: what was seen arrives remembered, before anything new.
        let again = cells(&enter(&mut test, 0));
        assert!(again.iter().any(|&(_, _, _, remembered)| remembered));
        let remembered: Vec<_> = again.iter().filter(|c| c.3).collect();
        assert!(remembered.len() <= first.len());
    }

    #[test]
    fn dev_teleports_go_through_walls_but_not_into_them() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let level = test.client.visit.as_ref().unwrap().level.level.clone();
        let start = test.client.visit.as_ref().unwrap().position;
        // Somewhere walkable, more than a step away.
        let (min_x, min_y, width, height) = level.bounds();
        let far = (min_y..min_y + height)
            .flat_map(|y| (min_x..min_x + width).map(move |x| (x, y)))
            .find(|&(x, y)| level.is_walkable(x, y) && (x - start.0).abs() > 3)
            .unwrap();
        let answers = test.send(&[ClientMessage::DevTeleport { x: far.0, y: far.1 }]);
        assert_eq!(answers[0], ServerMessage::Teleported { x: far.0, y: far.1 });
        assert_eq!(test.client.visit.as_ref().unwrap().position, far);
        // Not into a wall.
        let (wx, wy) = (min_x - 1, min_y - 1);
        assert!(
            test.send(&[ClientMessage::DevTeleport { x: wx, y: wy }])
                .is_empty()
        );
        // Back to the start leaves the site.
        let answers = test.send(&[ClientMessage::DevBackToStart]);
        assert!(matches!(answers[0], ServerMessage::OnWorldMap { .. }));
        assert!(test.client.visit.is_none());
    }

    #[test]
    fn steps_into_walls_are_refused() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let visit = test.client.visit.as_ref().unwrap();
        let (x, y) = visit.position;
        let level = visit.level.level.clone();
        let (dx, dy) = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (1, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
        ]
        .into_iter()
        .find(|&(dx, dy)| !level.is_walkable(x + dx, y + dy))
        .expect("a wall nearby");
        let answers = test.steps(&[(dx, dy)]);
        assert_eq!(test.client.visit.as_ref().unwrap().position, (x, y));
        // The client is told where the player still is.
        assert!(answers.contains(&ServerMessage::Position {
            map: test.client_map(),
            step: 1,
            x,
            y
        }));
    }

    #[test]
    fn stairs_and_the_way_out_work() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        // A castle has floors above: take the first stairs.
        let visit = test.client.visit.as_ref().unwrap();
        let stairs = visit.level.stairs[0];
        walk(&mut test, (stairs.x, stairs.y));
        assert_eq!(
            test.client.visit.as_ref().unwrap().position,
            (stairs.x, stairs.y)
        );
        let answers = test.send(&[ClientMessage::UseExit]);
        let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
            panic!("{answers:?}");
        };
        assert_eq!(Exit::Level(info.index), stairs.to);
        // Back down, then out through the entrance's ground.
        test.send(&[ClientMessage::UseExit]);
        assert_eq!(test.client.visit.as_ref().unwrap().index, 0);
        let entrance = test.client.visit.as_ref().unwrap().level.entrance.unwrap();
        let level = test.client.visit.as_ref().unwrap().level.level.clone();
        // Walk south from the entrance to the edge of the level.
        let mut out = entrance;
        while !leads_out(&level, out.0, out.1) && level.is_walkable(out.0, out.1 + 1) {
            out.1 += 1;
        }
        walk(&mut test, out);
        let answers = test.send(&[ClientMessage::UseExit]);
        assert!(matches!(
            answers[..],
            [ServerMessage::OnWorldMap { .. }, ..]
        ));
        assert!(test.client.visit.is_none());
        // Steps taken in the site before leaving don't count out here.
        let stale = ClientMessage::Step {
            map: test.client_map() - 1,
            number: 99,
            step: Step::new(1, 0).unwrap(),
        };
        let at = test.client.world_position();
        test.send(&[stale]);
        assert_eq!(test.client.world_position(), at);
    }

    /// A world saved in `folder`.
    fn saved_world(folder: &TestFolder) -> ServerWorld {
        let seed = WorldSeed::new(1);
        ServerWorld::open(seed, Storage::in_folder(&folder.0, seed.id())).unwrap()
    }

    #[test]
    fn coming_back_resumes_the_visit_with_its_memory() {
        let folder = TestFolder::new();
        let mut test = TestClient::of(saved_world(&folder));
        let first = cells(&enter(&mut test, 1));
        let position = test.client.visit.as_ref().unwrap().position;
        test.leave();

        // Another connection, even another server, on the same data.
        let mut test = TestClient::of(saved_world(&folder));
        let answers = test.send(&[]);
        assert!(answers.is_empty(), "the welcome was read: {answers:?}");
        let visit = test.client.visit.as_ref().expect("back in the site");
        assert_eq!((visit.index, visit.position), (1, position));
        // What was seen came back remembered.
        let memory = &test.client.character.memory;
        let id = SiteId::Dev {
            kind: FeatureKind::Castle,
            seed: castle_seed(),
        };
        let seen: u32 = memory[&id][&1].values().map(|m| m.count_ones()).sum();
        assert!(seen > 0 && seen as usize <= first.len());
    }

    #[test]
    fn welcome_sends_the_level_after_the_world() {
        let folder = TestFolder::new();
        let mut test = TestClient::of(saved_world(&folder));
        test.send(&[ClientMessage::DevTeleport { x: 70, y: -3 }]);
        enter(&mut test, 0);
        let test = TestClient::of(test.leave());
        assert!(matches!(
            test.welcome[..],
            [
                ServerMessage::Welcome { x: 70, y: -3, .. },
                ServerMessage::EnterLevel(_),
                ..
            ]
        ));
    }

    #[test]
    fn changes_are_applied_over_the_generated_level() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let (x, y) = test.client.visit.as_ref().unwrap().position;
        let id = SiteId::Dev {
            kind: FeatureKind::Castle,
            seed: castle_seed(),
        };
        let level = test.world.site_mut(id, FeatureKind::Castle).levels.entry(0);
        level.or_default().changes.insert((x + 1, y), Cell::Water);
        enter(&mut test, 0);
        let level = &test.client.visit.as_ref().unwrap().level.level;
        assert_eq!(level.cell(x + 1, y), Cell::Water);
    }

    #[test]
    fn sites_keep_the_generator_version_they_were_made_with() {
        let mut test = TestClient::new();
        let id = SiteId::Dev {
            kind: FeatureKind::Castle,
            seed: castle_seed(),
        };
        // A site made by a generator version this server can't run.
        test.world.site_mut(id, FeatureKind::Castle).version = 99;
        let answers = enter(&mut test, 0);
        assert!(answers.is_empty(), "{answers:?}");
        assert!(test.client.visit.is_none());
    }

    #[test]
    fn levels_record_when_they_were_last_visited() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let id = SiteId::Dev {
            kind: FeatureKind::Castle,
            seed: castle_seed(),
        };
        let mut world = test.leave();
        let last = world.site(id, FeatureKind::Castle).levels[&0].last_visit;
        assert!(last > 0);
    }

    #[test]
    fn nearby_cells_stop_at_walls() {
        let level = Level::test_room();
        let near = nearby_cells(&level, (-3, 0), 2);
        // The wall at x = -2 (y = -1..=1) is in the way east: only round
        // its ends, two steps away.
        assert!(near.iter().all(|&(x, _)| x <= -2));
        assert!(near.contains(&(-2, -2)) && !near.contains(&(-1, 0)));
        assert!(near.iter().all(|&(x, y)| level.is_walkable(x, y)));
        assert!(near.contains(&(-5, 0)) && near.contains(&(-3, 2)));
        assert!(!near.contains(&(-6, 0)));
    }
}
