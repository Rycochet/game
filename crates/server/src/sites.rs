//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to).

use std::collections::{BTreeMap, VecDeque};

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::feature::{ChunkPos, Feature, FeatureKind};
use shared::protocol::{
    ENTER_REACH, LEVEL_PRELOAD_STEPS, LevelChunk, LevelInfo, SeenStairs, ServerMessage, level_chunk,
};
use shared::rng::Seed;
use shared::site::{Exit, Site, SiteLevel, leads_out};

use crate::ServerWorld;
use crate::connections::Client;

/// Cells of a level by chunk: one bit per cell (see `level_chunk`).
pub(crate) type CellMasks = BTreeMap<(i64, i64), u64>;

/// A site level: the site's map seed and the level's index.
pub(crate) type LevelKey = (u64, u32);

/// A player's stay on a site level.
pub(crate) struct Visit {
    /// The feature the site is behind.
    feature: Feature,
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
    fn key(&self) -> LevelKey {
        (self.feature.seed.value(), self.index)
    }
}

/// Enters the site of the feature in `chunk`, if the player is at it.
pub(crate) fn enter_feature(client: &mut Client, world: &ServerWorld, chunk: ChunkPos) -> bool {
    let Some(feature) = world.features.in_chunk(chunk) else {
        warn!("A client tried to enter {chunk:?}, which has no feature");
        return true;
    };
    let near = client.world_position.is_some_and(|(x, y)| {
        // East-west, the nearer way round the planet.
        let dx = (feature.x - x).rem_euclid(world.width);
        dx.min(world.width - dx) <= ENTER_REACH && (feature.y - y).abs() <= ENTER_REACH
    });
    if client.visit.is_some() || !near {
        warn!("A client tried to enter {chunk:?} from elsewhere");
        return true;
    }
    let site = Site::new(feature.kind, feature.seed);
    enter_level(client, feature, site, 0, Exit::WorldMap)
}

/// Developer tools: enters a made-up site at any level, where the player
/// stands on the world map.
pub(crate) fn dev_enter(client: &mut Client, kind: FeatureKind, seed: u64, level: u32) -> bool {
    let (x, y) = client.world_position.unwrap_or((0, 0));
    let feature = Feature {
        kind,
        chunk: ChunkPos { x: 0, y: 0 },
        x,
        y,
        seed: Seed::new(seed),
    };
    let site = Site::new(kind, feature.seed);
    let index = level.min(site.levels() - 1);
    enter_level(client, feature, site, index, Exit::WorldMap)
}

/// The player says they are at `(x, y)` on their level: accepted if it is a
/// walkable cell, then sends what they now see or are about to.
pub(crate) fn moved_to(client: &mut Client, x: i64, y: i64) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    if !visit.level.level.is_walkable(x, y) {
        warn!("A client said it was on an unwalkable cell ({x}, {y})");
        return true;
    }
    visit.position = (x, y);
    update_sight(client)
}

/// Uses what the player stands on: stairs to another level, or ground on
/// the edge of the level back to the world map.
pub(crate) fn use_exit(client: &mut Client) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let (x, y) = visit.position;
    let stairs = visit.level.stairs.iter().find(|s| (s.x, s.y) == (x, y));
    let exit = match stairs {
        Some(stairs) => stairs.to,
        None if leads_out(&visit.level.level, x, y) => Exit::WorldMap,
        None => {
            client.visit = Some(visit);
            return true;
        }
    };
    match exit {
        Exit::Level(index) => {
            let from = Exit::Level(visit.index);
            enter_level(client, visit.feature, visit.site, index, from)
        }
        Exit::WorldMap => {
            let (x, y) = (visit.feature.x, visit.feature.y);
            client.world_position = Some((x, y));
            client.link.send(&ServerMessage::LeftSite { x, y })
        }
    }
}

/// Puts the player on level `index` of `site`, arriving from `from`: tells
/// the client about the level, sends what the player remembers of it, then
/// what they see.
fn enter_level(client: &mut Client, feature: Feature, site: Site, index: u32, from: Exit) -> bool {
    let level = site.level(index);
    let stairs = level
        .stairs
        .iter()
        .map(|&stairs| SeenStairs {
            x: stairs.x,
            y: stairs.y,
            direction: stairs.direction,
            descends: site.stairs_descend(index, &level.level, stairs),
        })
        .collect();
    let position = level.arrive(from);
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
        site,
        index,
        level,
        stairs,
        position,
        sent: CellMasks::new(),
    };
    if !client.link.send(&ServerMessage::EnterLevel(info)) {
        return false;
    }
    let remembered = client.memory.get(&visit.key()).cloned().unwrap_or_default();
    if !send_cells(client, &visit, &remembered, true) {
        return false;
    }
    visit.sent = remembered;
    client.visit = Some(visit);
    update_sight(client)
}

/// Works out what the player sees (remembered from now on) and what they
/// could see within `LEVEL_PRELOAD_STEPS`, and sends the cells of that the
/// client doesn't have yet.
fn update_sight(client: &mut Client) -> bool {
    let Some(visit) = &client.visit else {
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

    let memory = client.memory.entry(visit.key()).or_default();
    let mut new = CellMasks::new();
    let (min_x, min_y, width, height) = level.bounds();
    for cy in min_y..min_y + height {
        for cx in min_x..min_x + width {
            let (chunk, bit) = level_chunk(cx, cy);
            if seen.is_seen(cx, cy) {
                *memory.entry(chunk).or_default() |= 1 << bit;
            }
            let sent = visit.sent.get(&chunk).copied().unwrap_or(0);
            if loaded.is_seen(cx, cy) && sent & (1 << bit) == 0 {
                *new.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    if !send_cells(client, visit, &new, false) {
        return false;
    }
    let visit = client.visit.as_mut().expect("still there");
    for (chunk, mask) in new {
        *visit.sent.entry(chunk).or_default() |= mask;
    }
    true
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

    use super::*;
    use crate::connections::tests::TestClient;

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
        // Standing still sends nothing more.
        let visit = test.client.visit.as_ref().unwrap();
        let (x, y) = visit.position;
        assert!(cells(&test.send(&[ClientMessage::SitePosition { x, y }])).is_empty());
        // Coming back: what was seen arrives remembered, before anything new.
        let again = cells(&enter(&mut test, 0));
        assert!(again.iter().any(|&(_, _, _, remembered)| remembered));
        let remembered: Vec<_> = again.iter().filter(|c| c.3).collect();
        assert!(remembered.len() <= first.len());
    }

    #[test]
    fn unwalkable_positions_are_ignored() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let before = test.client.visit.as_ref().unwrap().position;
        let (min_x, min_y, _, _) = test.client.visit.as_ref().unwrap().level.level.bounds();
        // The level's corner is wall (or ground outside a building).
        test.send(&[ClientMessage::SitePosition {
            x: min_x - 5,
            y: min_y - 5,
        }]);
        assert_eq!(test.client.visit.as_ref().unwrap().position, before);
    }

    #[test]
    fn stairs_and_the_way_out_work() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        // A castle has floors above: take the first stairs.
        let visit = test.client.visit.as_ref().unwrap();
        let stairs = visit.level.stairs[0];
        test.send(&[ClientMessage::SitePosition {
            x: stairs.x,
            y: stairs.y,
        }]);
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
        test.send(&[ClientMessage::SitePosition { x: out.0, y: out.1 }]);
        let answers = test.send(&[ClientMessage::UseExit]);
        assert!(matches!(answers[..], [ServerMessage::LeftSite { .. }]));
        assert!(test.client.visit.is_none());
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
