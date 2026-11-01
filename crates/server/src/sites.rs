//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to). What a player has seen is kept with their
//! character; each site keeps its generator version, when its levels were
//! last visited and players' changes (`storage::SiteSave`).

use std::collections::VecDeque;

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::movement::{Step, step_on_level};
use shared::poi::{ChunkPos, Poi, PoiKind};
use shared::protocol::{
    ENTER_REACH, LevelChunk, LevelInfo, RememberedLevel, SeenStairs, ServerMessage, level_chunk,
};
use shared::rng::Seed;
use shared::site::{Exit, Generator, Site, SiteLevel, StairsDirection, leads_out};

use crate::connections::Client;
use crate::storage::{CellMasks, SiteId, VisitSave, now};
use crate::world_map::{self, look_around};
use crate::{LEVEL_PRELOAD_STEPS, ServerWorld};

/// A player's stay on a site level.
pub(crate) struct Visit {
    /// The poi the site is behind.
    poi: Poi,
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
    /// The cells already looked out from for what to send, and what was seen
    /// from them, so each step only looks from the cells that are new.
    looked_from: Sight,
    loaded: Sight,
}

impl Visit {
    /// Where the player is on the level.
    pub(crate) fn position(&self) -> (i64, i64) {
        self.position
    }

    /// Whether the site is a town, whose layout players always know.
    fn is_town(&self) -> bool {
        self.site.kind().is_town()
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

/// Enters the site of the poi in `chunk`, if the player is at it.
pub(crate) fn enter_poi(client: &mut Client, world: &mut ServerWorld, chunk: ChunkPos) -> bool {
    let Some(poi) = world_map::known_poi(client, world, chunk) else {
        warn!("A client tried to enter {chunk:?}, with no poi it knows of");
        return true;
    };
    let (x, y) = client.world_position();
    // East-west, the nearer way round the planet.
    let dx = (poi.x - x).rem_euclid(world.width);
    let near = dx.min(world.width - dx) <= ENTER_REACH && (poi.y - y).abs() <= ENTER_REACH;
    if client.visit.is_some() || !near {
        warn!("A client tried to enter {chunk:?} from elsewhere");
        return true;
    }
    client.character.pois.entry(chunk).or_default().visited = true;
    client.unsaved = true;
    let id = SiteId::Poi(chunk);
    enter_site(client, world, poi, id, 0, Arrival::From(Exit::WorldMap))
}

/// Developer tools: enters a made-up site at any level, where the player
/// stands on the world map.
pub(crate) fn dev_enter(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: PoiKind,
    seed: u64,
    level: u32,
    ruin: u8,
) -> bool {
    leave(client, world);
    client.visit = None;
    let poi = dev_poi(client, kind, seed);
    let id = SiteId::Dev { kind, seed, ruin };
    enter_site(client, world, poi, id, level, Arrival::From(Exit::WorldMap))
}

/// The made-up poi of a dev tools site: where the player stands.
fn dev_poi(client: &Client, kind: PoiKind, seed: u64) -> Poi {
    let (x, y) = client.world_position();
    Poi {
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
    let poi = match saved.site {
        SiteId::Poi(chunk) => world.pois.in_chunk(chunk),
        SiteId::Dev { kind, seed, .. } => Some(dev_poi(client, kind, seed)),
    };
    let Some(poi) = poi else {
        warn!("A character was in {:?}, which isn't there", saved.site);
        client.unsaved = true;
        return true;
    };
    let at = Arrival::At(saved.position);
    enter_site(client, world, poi, saved.site, saved.level, at)
}

/// Generates the site (with the generator version it was first made with)
/// and puts the player on level `index` (or its last level, if there are
/// fewer).
fn enter_site(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    id: SiteId,
    index: u32,
    arrival: Arrival,
) -> bool {
    let version = world.site(id, poi.kind).version;
    let generator = match id {
        SiteId::Dev { ruin, .. } => Generator::for_poi(poi.kind).with_ruin(ruin),
        SiteId::Poi(_) => Generator::for_site(poi.kind, poi.seed),
    };
    let Some(site) = Site::with_generator(poi.kind, poi.seed, version, generator) else {
        error!("{id:?} needs generator version {version}, which this server can't run");
        return true;
    };
    let index = index.min(site.levels() - 1);
    let new = NewVisit { poi, id, site };
    enter_level(client, world, new, index, arrival, true)
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
            // A building's stairs are on the same cell of both floors, and it
            // may have several: arrive at the ones taken.
            let from = if visit.site.is_building() {
                Arrival::At((x, y))
            } else {
                Arrival::From(Exit::Level(visit.index))
            };
            let new = NewVisit {
                poi: visit.poi,
                id: visit.id,
                site: visit.site,
            };
            enter_level(client, world, new, index, from, false)
        }
        Exit::WorldMap => {
            let (x, y) = (visit.poi.x, visit.poi.y);
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
        let site = world.site_mut(visit.id, visit.poi.kind);
        site.levels.entry(visit.index).or_default().last_visit = now();
    }
}

/// Developer tools: changes a cell of the player's level (kept as a player's
/// change), and sends it again with what can be seen now.
pub(crate) fn set_cell(
    client: &mut Client,
    world: &mut ServerWorld,
    x: i64,
    y: i64,
    cell: Cell,
) -> bool {
    let Some(visit) = &mut client.visit else {
        return true;
    };
    if !visit.level.level.contains(x, y) || visit.position == (x, y) {
        return true;
    }
    visit.level.level.set(x, y, cell);
    let site = world.site_mut(visit.id, visit.poi.kind);
    let saved = site.levels.entry(visit.index).or_default();
    saved.changes.insert((x, y), cell);
    // Everything may look different from everywhere now.
    visit.looked_from = Sight::new(&visit.level.level);
    visit.loaded = Sight::new(&visit.level.level);
    let (chunk, bit) = level_chunk(x, y);
    if let Some(mask) = visit.sent.get_mut(&chunk) {
        *mask &= !(1 << bit);
    }
    update_sight(client)
}

/// The site a player is going to a level of.
struct NewVisit {
    poi: Poi,
    id: SiteId,
    site: Site,
}

/// Level `index` of a site with players' changes over it, and its stairs as
/// players see them.
fn level_of(world: &mut ServerWorld, new: &NewVisit, index: u32) -> (SiteLevel, Vec<SeenStairs>) {
    let site = &new.site;
    let mut level = site.level(index);
    let saved = world
        .site_mut(new.id, new.poi.kind)
        .levels
        .entry(index)
        .or_default();
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
            style: site.stairs_style(index, stairs),
            open_to_sky: match stairs.to {
                Exit::WorldMap => true,
                // Also only for looks, and only worth generating going up.
                Exit::Level(other) => {
                    stairs.direction == StairsDirection::Up
                        && site
                            .level(other)
                            .level
                            .cell(stairs.x, stairs.y)
                            .is_outside()
                }
            },
        })
        .collect();
    (level, stairs)
}

/// Sends what the character remembers of the site's levels other than
/// `index`, for the maps.
fn send_other_levels(client: &Client, world: &mut ServerWorld, new: &NewVisit, index: u32) -> bool {
    let Some(memory) = client.character.memory.get(&new.id) else {
        return true;
    };
    for (&other, remembered) in memory {
        // (A level may be gone, if the generator has changed.)
        let known = remembered.values().any(|&mask| mask != 0);
        if other == index || other >= new.site.levels() || !known {
            continue;
        }
        let (level, stairs) = level_of(world, new, other);
        let level = &level.level;
        let is_remembered = |x, y| {
            let (chunk, bit) = level_chunk(x, y);
            remembered.get(&chunk).is_some_and(|m| m & (1 << bit) != 0)
        };
        let neighbours = beside(level, is_remembered, remembered);
        let mut chunks = level_chunks(level, &stairs, remembered, true);
        chunks.extend(level_chunks(level, &stairs, &neighbours, false));
        let message = ServerMessage::RememberedLevel(RememberedLevel {
            index: other,
            bounds: level.bounds(),
            chunks,
        });
        if !client.link.send(&message) {
            return false;
        }
    }
    true
}

/// Puts the player on level `index` of a site: generates it, tells the client
/// about it, sends what the player remembers of it, then what they see. On
/// coming into the site, also what they remember of its other levels.
fn enter_level(
    client: &mut Client,
    world: &mut ServerWorld,
    new: NewVisit,
    index: u32,
    arrival: Arrival,
    entering_site: bool,
) -> bool {
    let (level, stairs) = level_of(world, &new, index);
    let saved = world.site_mut(new.id, new.poi.kind).levels.entry(index);
    saved.or_default().last_visit = now();
    let site = &new.site;
    let position = match arrival {
        Arrival::At((x, y)) if level.level.is_walkable(x, y) => (x, y),
        Arrival::At(_) => level.arrive(Exit::WorldMap),
        Arrival::From(from) => level.arrive(from),
    };
    let info = LevelInfo {
        kind: site.kind(),
        ruined: site.is_ruined(),
        building: site.is_building(),
        index,
        levels: site.levels(),
        basement: site.has_basement(),
        bounds: level.level.bounds(),
        arrival: position,
        entering_site,
    };
    let looked_from = Sight::new(&level.level);
    let loaded = Sight::new(&level.level);
    client.changed_map();
    if !client.link.send(&ServerMessage::EnterLevel(info)) {
        return false;
    }
    if entering_site && !send_other_levels(client, world, &new, index) {
        return false;
    }
    let NewVisit { poi, id, site } = new;
    let mut visit = Visit {
        looked_from,
        loaded,
        poi,
        id,
        site,
        index,
        level,
        stairs,
        position,
        sent: CellMasks::new(),
    };
    if visit.is_town() {
        let looks = visit.level.buildings.iter().map(|b| b.look.clone());
        let message = ServerMessage::Buildings(looks.collect());
        if !client.link.send(&message) {
            return false;
        }
    }
    if !visit.level.fields.is_empty() {
        let message = ServerMessage::Fields(visit.level.fields.clone());
        if !client.link.send(&message) {
            return false;
        }
    }
    // A town's layout is known from the start, and so isn't kept as memory.
    let remembered = if visit.is_town() {
        all_cells(&visit.level.level)
    } else {
        memory(client, &visit).clone()
    };
    if !send_cells(client, &visit, &remembered, true) {
        return false;
    }
    // Their neighbours too, not as remembered: they shape what is.
    let level = &visit.level.level;
    let is_remembered = |x, y| {
        let (chunk, bit) = level_chunk(x, y);
        remembered.get(&chunk).is_some_and(|m| m & (1 << bit) != 0)
    };
    let neighbours = beside(level, is_remembered, &remembered);
    if !send_cells(client, &visit, &neighbours, false) {
        return false;
    }
    visit.sent = remembered;
    add_masks(&mut visit.sent, neighbours);
    client.visit = Some(visit);
    client.unsaved = true;
    update_sight(client)
}

/// Sends the level the player is on again, as if arriving where they stand.
fn arrive_again(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let Visit {
        poi,
        id,
        site,
        index,
        position,
        ..
    } = visit;
    let new = NewVisit { poi, id, site };
    enter_level(client, world, new, index, Arrival::At(position), false)
}

/// Developer tools: the character knows the whole of the level they are on. A
/// secret door stays a wall, as it does when seen.
pub(crate) fn reveal_level(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    // A town is known already.
    if !visit.is_town() {
        *memory(client, &visit) = all_cells(&visit.level.level);
        client.visit = Some(visit);
        return arrive_again(client, world);
    }
    client.visit = Some(visit);
    true
}

/// Developer tools: the character forgets the level they are on.
pub(crate) fn forget_level(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    // A town would only be sent whole again.
    if !visit.is_town() {
        memory(client, &visit).clear();
        client.visit = Some(visit);
        return arrive_again(client, world);
    }
    client.visit = Some(visit);
    true
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
    let Some(mut visit) = client.visit.take() else {
        return true;
    };
    if visit.is_town() {
        // The client has it all, except any cell that has just changed.
        let mut new = all_cells(&visit.level.level);
        for (chunk, sent) in &visit.sent {
            if let Some(mask) = new.get_mut(chunk) {
                *mask &= !sent;
            }
        }
        new.retain(|_, mask| *mask != 0);
        let sent = send_cells(client, &visit, &new, true);
        let visit = client.visit.insert(visit);
        add_masks(&mut visit.sent, new);
        return sent;
    }
    let (looked_from, loaded) = (&mut visit.looked_from, &mut visit.loaded);
    let level = &visit.level.level;
    let (x, y) = visit.position;
    // What the client draws: glimpses, the more generous sight.
    let mut seen = Sight::new(level);
    seen.add_glimpses(level, x, y, None);
    for (nx, ny) in nearby_cells(level, (x, y), LEVEL_PRELOAD_STEPS) {
        if !looked_from.is_seen(nx, ny) {
            looked_from.insert(nx, ny);
            loaded.add_glimpses(level, nx, ny, None);
        }
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
            if visit.loaded.is_seen(cx, cy) && sent & (1 << bit) == 0 {
                *new.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    client.unsaved |= learnt;
    // Their neighbours too: a wall's shape depends on what is beside it.
    let extra = beside(level, |cx, cy| visit.loaded.is_seen(cx, cy), &visit.sent);
    add_masks(&mut new, extra);
    let sent = send_cells(client, &visit, &new, false);
    let visit = client.visit.insert(visit);
    add_masks(&mut visit.sent, new);
    sent
}

/// Every cell of a level.
fn all_cells(level: &Level) -> CellMasks {
    let mut masks = CellMasks::new();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let (chunk, bit) = level_chunk(x, y);
            *masks.entry(chunk).or_default() |= 1 << bit;
        }
    }
    masks
}

/// The cells next to any cell for which `is_in` holds, on `level`, that
/// `except` doesn't include.
fn beside(level: &Level, is_in: impl Fn(i64, i64) -> bool, except: &CellMasks) -> CellMasks {
    let mut found = CellMasks::new();
    let (min_x, min_y, width, height) = level.bounds();
    for y in min_y..min_y + height {
        for x in min_x..min_x + width {
            let (chunk, bit) = level_chunk(x, y);
            let done = except.get(&chunk).is_some_and(|m| m & (1 << bit) != 0);
            let next_to = [(0, -1), (1, 0), (0, 1), (-1, 0)]
                .iter()
                .any(|(dx, dy)| is_in(x + dx, y + dy));
            if !done && !is_in(x, y) && next_to {
                *found.entry(chunk).or_default() |= 1 << bit;
            }
        }
    }
    found
}

/// Adds the cells in `more` to `masks`.
fn add_masks(masks: &mut CellMasks, more: CellMasks) {
    for (chunk, mask) in more {
        *masks.entry(chunk).or_default() |= mask;
    }
}

/// The walkable cells the player could reach within `steps` steps (in any
/// of 8 directions, a jump over liquid counting as one), including where they
/// stand.
fn nearby_cells(level: &Level, from: (i64, i64), steps: i64) -> Vec<(i64, i64)> {
    let mut found = vec![from];
    let mut queue = VecDeque::from([(from, 0)]);
    while let Some((at, distance)) = queue.pop_front() {
        if distance == steps {
            continue;
        }
        for dy in -1..=1 {
            for dx in -1..=1 {
                let Some(step) = Step::new(dx, dy) else {
                    continue;
                };
                if let Some(next) = step_on_level(level, at, step)
                    && !found.contains(&next)
                {
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
    level_chunks(&visit.level.level, &visit.stairs, masks, remembered)
        .into_iter()
        .all(|chunk| client.link.send(&ServerMessage::LevelCells(chunk)))
}

/// The cells in `masks` as the player would see them, a chunk at a time, with
/// any stairs on them.
fn level_chunks(
    level: &Level,
    stairs: &[SeenStairs],
    masks: &CellMasks,
    remembered: bool,
) -> Vec<LevelChunk> {
    let mut chunks = Vec::with_capacity(masks.len());
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
        let stairs = stairs
            .iter()
            .filter(|s| positions.contains(&(s.x, s.y)))
            .copied()
            .collect();
        chunks.push(LevelChunk {
            cells,
            stairs,
            ..message
        });
    }
    chunks
}

#[cfg(test)]
mod tests {
    use shared::protocol::{ClientMessage, ServerMessage};

    use shared::world::WorldSeed;

    use super::*;
    use crate::connections::tests::TestClient;
    use crate::storage::Storage;
    use crate::storage::tests::TestFolder;

    /// A town arrives whole and remembered, with its buildings, and nothing
    /// of it is kept in the character's memory.
    #[test]
    fn towns_are_known_at_once() {
        let mut test = TestClient::new();
        let answers = test.send(&[ClientMessage::DevEnterSite {
            kind: PoiKind::Village,
            seed: 4,
            level: 0,
            ruin: 0,
        }]);
        let site = Site::new(PoiKind::Village, Seed::new(4)).level(0);
        let looks = answers.iter().find_map(|message| match message {
            ServerMessage::Buildings(looks) => Some(looks),
            _ => None,
        });
        assert_eq!(looks.unwrap().len(), site.buildings.len());
        let mut cells = 0;
        for answer in &answers {
            if let ServerMessage::LevelCells(chunk) = answer {
                assert!(chunk.remembered);
                cells += chunk.cells.len() as i64;
            }
        }
        let (_, _, width, height) = site.level.bounds();
        assert_eq!(cells, width * height);
        let memory = test.client.character.memory.values();
        assert!(
            memory
                .flat_map(|levels| levels.values())
                .all(|m| m.is_empty())
        );
    }

    /// A castle with more than one floor.
    fn castle_seed() -> u64 {
        (1..)
            .find(|&seed| Site::new(PoiKind::Castle, Seed::new(seed)).levels() > 1)
            .unwrap()
    }

    /// Enters a test site (a castle with stairs) and returns the answers.
    fn enter(test: &mut TestClient, level: u32) -> Vec<ServerMessage> {
        test.send(&[ClientMessage::DevEnterSite {
            kind: PoiKind::Castle,
            seed: castle_seed(),
            level,
            ruin: 0,
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
    fn revealing_sends_the_whole_level_remembered_and_keeps_the_place() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let start = test.client.visit.as_ref().unwrap().position;
        let answers = test.send(&[ClientMessage::DevRevealMap]);
        let ServerMessage::EnterLevel(info) = &answers[0] else {
            panic!("{:?}", answers[0]);
        };
        assert_eq!(info.arrival, start);
        assert!(!info.entering_site);
        let (_, _, width, height) = info.bounds;
        let sent = cells(&answers);
        let remembered = sent.iter().filter(|cell| cell.3).count();
        assert_eq!(remembered, (width * height) as usize);
        assert!(sent.iter().all(|&(_, _, cell, _)| cell != Cell::SecretDoor));
    }

    #[test]
    fn forgetting_a_level_leaves_only_what_is_in_sight() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        test.send(&[ClientMessage::DevRevealMap]);
        let answers = test.send(&[ClientMessage::DevForgetMap]);
        assert!(matches!(answers[0], ServerMessage::EnterLevel(_)));
        let sent = cells(&answers);
        assert!(sent.iter().all(|cell| !cell.3), "nothing remembered");
        let ServerMessage::EnterLevel(info) = &answers[0] else {
            unreachable!();
        };
        let (_, _, width, height) = info.bounds;
        assert!(sent.len() < (width * height) as usize);
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

    #[test]
    fn coming_into_a_site_tells_what_is_remembered_of_its_other_levels() {
        let mut test = TestClient::new();
        let remembered = |answers: &[ServerMessage]| -> Vec<u32> {
            let levels = answers.iter().filter_map(|message| match message {
                ServerMessage::RememberedLevel(level) => Some(level.index),
                _ => None,
            });
            levels.collect()
        };
        // Nothing yet, and nothing of the level the player is going to.
        let answers = enter(&mut test, 0);
        assert_eq!(remembered(&answers), [] as [u32; 0]);
        let answers = enter(&mut test, 0);
        assert_eq!(remembered(&answers), [] as [u32; 0]);

        enter(&mut test, 1);
        let answers = enter(&mut test, 0);
        let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
            panic!("{answers:?}");
        };
        assert!(info.entering_site);
        assert_eq!(remembered(&answers), [1]);
        let level = answers.iter().find_map(|message| match message {
            ServerMessage::RememberedLevel(level) => Some(level),
            _ => None,
        });
        let chunks = &level.expect("found above").chunks;
        assert!(chunks.iter().any(|chunk| chunk.remembered));

        // Taking stairs tells nothing new: the client keeps what it has.
        let visit = test.client.visit.as_ref().unwrap();
        let stairs = visit.level.stairs[0];
        walk(&mut test, (stairs.x, stairs.y));
        let answers = test.send(&[ClientMessage::UseExit]);
        let Some(ServerMessage::EnterLevel(info)) = answers.first() else {
            panic!("{answers:?}");
        };
        assert!(!info.entering_site);
        assert_eq!(remembered(&answers), [] as [u32; 0]);
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
            kind: PoiKind::Castle,
            seed: castle_seed(),
            ruin: 0,
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
            kind: PoiKind::Castle,
            seed: castle_seed(),
            ruin: 0,
        };
        let level = test.world.site_mut(id, PoiKind::Castle).levels.entry(0);
        level.or_default().changes.insert((x + 1, y), Cell::Water);
        enter(&mut test, 0);
        let level = &test.client.visit.as_ref().unwrap().level.level;
        assert_eq!(level.cell(x + 1, y), Cell::Water);
    }

    #[test]
    fn dev_tools_change_cells_and_keep_the_change() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let (x, y) = test.client.visit.as_ref().unwrap().position;
        let (dx, dy) = walkable_neighbour(&test);
        let (nx, ny) = (x + dx, y + dy);
        let answers = test.send(&[ClientMessage::DevSetCell {
            x: nx,
            y: ny,
            cell: Cell::Window,
        }]);
        assert!(
            cells(&answers)
                .iter()
                .any(|&sent| (sent.0, sent.1, sent.2) == (nx, ny, Cell::Window))
        );
        // Not where the player stands.
        test.send(&[ClientMessage::DevSetCell {
            x,
            y,
            cell: Cell::Wall,
        }]);
        // Windows can't be stepped on; and the change is there next time.
        enter(&mut test, 0);
        let level = &test.client.visit.as_ref().unwrap().level.level;
        assert_eq!(level.cell(nx, ny), Cell::Window);
        assert!(level.is_walkable(x, y));
        assert!(!level.is_walkable(nx, ny));
    }

    #[test]
    fn walls_arrive_with_what_shapes_them() {
        let mut test = TestClient::new();
        let sent = cells(&enter(&mut test, 0));
        let is_sent = |x, y| sent.iter().any(|&(cx, cy, _, _)| (cx, cy) == (x, y));
        let visit = test.client.visit.as_ref().unwrap();
        let level = &visit.level.level;
        let mut sight = Sight::new(level);
        sight.add_glimpses(level, visit.position.0, visit.position.1, None);
        let seen_wall =
            |&&(x, y, cell, _): &&(i64, i64, Cell, bool)| cell.is_wall() && sight.is_seen(x, y);
        let walls: Vec<_> = sent.iter().filter(seen_wall).collect();
        assert!(!walls.is_empty());
        for &&(x, y, _, _) in &walls {
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                assert!(!level.contains(nx, ny) || is_sent(nx, ny), "{nx}, {ny}");
            }
        }
    }

    #[test]
    fn sites_keep_the_generator_version_they_were_made_with() {
        let mut test = TestClient::new();
        let id = SiteId::Dev {
            kind: PoiKind::Castle,
            seed: castle_seed(),
            ruin: 0,
        };
        // A site made by a generator version this server can't run.
        test.world.site_mut(id, PoiKind::Castle).version = 99;
        let answers = enter(&mut test, 0);
        assert!(answers.is_empty(), "{answers:?}");
        assert!(test.client.visit.is_none());
    }

    #[test]
    fn only_stairs_up_into_the_open_are_open_to_the_sky() {
        let mut test = TestClient::new();
        test.send(&[ClientMessage::DevEnterSite {
            kind: PoiKind::Grave,
            seed: 1,
            level: 0,
            ruin: 0,
        }]);
        let visit = test.client.visit.as_ref().unwrap();
        let out = visit
            .level
            .stairs
            .iter()
            .position(|stairs| stairs.to == Exit::WorldMap)
            .unwrap();
        assert!(visit.stairs[out].open_to_sky);

        // A castle's stairs lead to more castle.
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let visit = test.client.visit.as_ref().unwrap();
        assert!(visit.stairs.iter().all(|stairs| !stairs.open_to_sky));
    }

    #[test]
    fn levels_record_when_they_were_last_visited() {
        let mut test = TestClient::new();
        enter(&mut test, 0);
        let id = SiteId::Dev {
            kind: PoiKind::Castle,
            seed: castle_seed(),
            ruin: 0,
        };
        let mut world = test.leave();
        let last = world.site(id, PoiKind::Castle).levels[&0].last_visit;
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
