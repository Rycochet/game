//! Players in sites: entering them, moving about, taking stairs and leaving,
//! and sending each player the cells of their level as they come to see
//! them (or are about to). What a player has seen is kept with their
//! character; each site keeps its generator version, when its levels were
//! last visited and players' changes (`storage::SiteSave`).

use std::collections::VecDeque;

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::id::Id;
use shared::item::{Item, MadeAt, Quality};
use shared::item_form::give_form;
use shared::mob::Event;
use shared::movement::{Step, step_on_level};
use shared::poi::{ChunkPos, Poi, PoiKind};
use shared::protocol::{
    ENTER_REACH, LevelChunk, LevelInfo, RememberedLevel, SeenStairs, ServerMessage, level_chunk,
};
use shared::rng::Seed;
use shared::shop::{DOOR_REACH, Shop, catch_up, sells};
use shared::site::{Exit, Generator, MAX_RUIN, Site, SiteLevel, StairsDirection, leads_out};

use crate::connections::Client;
use crate::ids::new_id;
use crate::storage::{CellMasks, SiteId, VisitSave, now};
use crate::world_map::{self, look_around};
use crate::{LEVEL_PRELOAD_STEPS, ServerWorld};

/// How far in a town others can be made out, in cells.
const TOWN_SIGHT: u32 = 40;

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
    /// What can be seen now, by the game's rule: who else is in sight. Worked out
    /// from `view_from`, and only again once the player has moved.
    view: Sight,
    view_from: Option<(i64, i64)>,
}

impl Visit {
    /// Where the player is on the level.
    pub(crate) fn position(&self) -> (i64, i64) {
        self.position
    }

    /// The level's first stairs, for tests.
    #[cfg(test)]
    pub(crate) fn first_stairs(&self) -> (i64, i64) {
        let stairs = self.level.stairs[0];
        (stairs.x, stairs.y)
    }

    /// A walkable cell the player can't see, for tests.
    #[cfg(test)]
    pub(crate) fn walkable_out_of_sight(&mut self) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| self.level.level.is_walkable(x, y) && !self.can_see(x, y))
            .expect("a site has more than one room")
    }

    /// A cell with room for this many steps east of it, for tests.
    #[cfg(test)]
    pub(crate) fn room_to_walk_east(&self, steps: i64) -> (i64, i64) {
        const SEARCH: i64 = 200;
        (-SEARCH..SEARCH)
            .flat_map(|y| (-SEARCH..SEARCH).map(move |x| (x, y)))
            .find(|&(x, y)| (0..=steps).all(|dx| self.level.level.is_walkable(x + dx, y)))
            .expect("a site has a room")
    }

    /// Where a step from here lands, if it can be taken.
    pub(crate) fn landing(&self, step: Step) -> Option<(i64, i64)> {
        step_on_level(&self.level.level, self.position, step)
    }

    /// Which level of the site it is.
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Which site it is, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn site_id(&self) -> SiteId {
        self.id
    }

    /// The level's cells as the server has them, for the debug API.
    #[cfg(feature = "debug-api")]
    pub(crate) fn cells(&self) -> &shared::dungeon::Level {
        &self.level.level
    }

    /// Whether something at this cell can be seen from where the player is.
    pub(crate) fn can_see(&mut self, x: i64, y: i64) -> bool {
        if self.view_from != Some(self.position) {
            let (from_x, from_y) = self.position;
            let level = &self.level.level;
            self.view = Sight::new(level);
            // Towns are big, and work out no sight otherwise.
            let reach = self.is_town().then_some(TOWN_SIGHT);
            self.view.add_view(level, from_x, from_y, reach);
            self.view_from = Some(self.position);
        }
        self.view.is_seen(x, y)
    }

    /// Which site it is.
    pub(crate) fn id(&self) -> SiteId {
        self.id
    }

    /// What kind of poi it is behind.
    pub(crate) fn kind(&self) -> PoiKind {
        self.poi.kind
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

/// Developer tools: a made-up site as if new, whatever anyone has done there or
/// remembers of it, and the player in it. It is on no map: leaving it comes back
/// to where they stand now.
pub(crate) fn generate(
    client: &mut Client,
    world: &mut ServerWorld,
    kind: PoiKind,
    seed: u64,
    ruin: u8,
) -> bool {
    let ruin = if Generator::can_be_ruined(kind) {
        ruin.min(MAX_RUIN)
    } else {
        0
    };
    leave(client, world);
    client.visit = None;
    let id = SiteId::Dev { kind, seed, ruin };
    world.forget_site(id);
    client.character.memory.remove(&id);
    client.character.bags.remove(&id);
    client.character.site_versions.remove(&id);
    dev_enter(client, world, kind, seed, 0, ruin)
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
    let id = saved.site;
    let poi = match id {
        SiteId::Poi(chunk) => world.pois.in_chunk(chunk),
        SiteId::Dev { kind, seed, .. } => Some(dev_poi(client, kind, seed)),
    };
    let Some(poi) = poi else {
        warn!("A character was in {id:?}, which isn't there");
        client.unsaved = true;
        return world_map::look_around(client, world);
    };
    if let Err(error) = world.open_site(id, poi.kind) {
        return client.refuse_for_saves(&error);
    }
    // Their map is the version they knew (someone new to it knows the one saved
    // with the site).
    let saved_version = world.site(id, poi.kind).version;
    let knew = client.character.site_versions.get(&id).copied();
    let knew = knew.unwrap_or(saved_version);
    let site = make_site(world, &poi, id);
    if site.is_none() || world.site(id, poi.kind).version != knew {
        info!("A character was in {id:?}, which has changed: putting them out");
        return leave_for_poi(client, world, poi, id);
    }
    let at = Arrival::At(saved.position);
    enter_site(client, world, poi, id, saved.level, at)
}

/// Developer tools: the server forgets the site the player is on, and they
/// arrive on it again as first made, on the same level and cell if they can.
pub(crate) fn forget_site(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    let Visit {
        poi,
        id,
        index,
        position,
        ..
    } = visit;
    world.forget_site(id);
    client.character.memory.remove(&id);
    client.character.bags.remove(&id);
    client.character.site_versions.remove(&id);
    enter_site(client, world, poi, id, index, Arrival::At(position))
}

/// Puts the player out of the site they are on, which is no longer what they
/// are looking at, at its poi. They can go in again.
pub(crate) fn put_out(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = client.visit.take() else {
        return true;
    };
    leave_for_poi(client, world, visit.poi, visit.id)
}

/// The character forgets a site that has changed, and is on the world map at
/// its poi.
fn leave_for_poi(client: &mut Client, world: &mut ServerWorld, poi: Poi, id: SiteId) -> bool {
    client.character.memory.remove(&id);
    client.character.bags.remove(&id);
    client.character.site_versions.remove(&id);
    client.character.world_position = (poi.x, poi.y);
    client.unsaved = true;
    client.changed_map();
    let (x, y) = (poi.x, poi.y);
    client.link.send(&ServerMessage::OnWorldMap { x, y }) && look_around(client, world)
}

/// Generates a site with the generator version it was first made with. If this
/// server can't run that one any more, the site is forgotten and made anew.
fn make_site(world: &mut ServerWorld, poi: &Poi, id: SiteId) -> Option<Site> {
    let generator = match id {
        SiteId::Dev { ruin, .. } => Generator::for_poi(poi.kind).with_ruin(ruin),
        SiteId::Poi(_) => Generator::for_site(poi.kind, poi.seed),
    };
    let version = world.site(id, poi.kind).version;
    let site = Site::with_generator(poi.kind, poi.seed, version, generator);
    if site.is_some() {
        return site;
    }
    warn!("{id:?} needs generator version {version}, which this server can't run: remaking it");
    world.forget_site(id);
    let version = world.site(id, poi.kind).version;
    Site::with_generator(poi.kind, poi.seed, version, generator)
}

/// Generates the site and puts the player on level `index` (or its last level,
/// if there are fewer). What they remember of it goes if its map has changed.
fn enter_site(
    client: &mut Client,
    world: &mut ServerWorld,
    poi: Poi,
    id: SiteId,
    index: u32,
    arrival: Arrival,
) -> bool {
    if let Err(error) = world.open_site(id, poi.kind) {
        return client.refuse_for_saves(&error);
    }
    let Some(site) = make_site(world, &poi, id) else {
        error!("{id:?} can't be made by this server");
        return true;
    };
    let version = world.site(id, poi.kind).version;
    let known = client.character.site_versions.insert(id, version);
    if known.is_some_and(|known| known != version) {
        client.character.memory.remove(&id);
        client.character.bags.remove(&id);
    }
    client.unsaved = true;
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

/// Who keeps each of a town level's shops, by building. Each has one for good, made
/// the first time it is needed.
fn keepers_of(world: &mut ServerWorld, visit: &NewKeepers) -> Vec<(u16, Id)> {
    let saved = world
        .site_mut(visit.id, visit.kind)
        .levels
        .entry(visit.index);
    let saved = saved.or_default();
    let shops = visit.shops.iter().map(|&building| {
        let save = saved.buildings.entry(building).or_default();
        let keeper = *save.keeper.get_or_insert_with(new_id);
        let old_stock = std::mem::take(&mut save.stock);
        let record = saved.records.entry(keeper).or_default();
        record.carried.extend(old_stock);
        (building, keeper)
    });
    shops.collect()
}

/// A town level, for `keepers_of`.
struct NewKeepers {
    id: SiteId,
    kind: PoiKind,
    index: u32,
    shops: Vec<u16>,
}

/// Which building on the player's level this is the keeper of, if any.
pub(crate) fn keeper_building(client: &Client, world: &mut ServerWorld, with: Id) -> Option<u16> {
    let visit = client.visit.as_ref()?;
    let level = world
        .site(visit.id, visit.poi.kind)
        .levels
        .get(&visit.index)?;
    let mut buildings = level.buildings.iter();
    buildings
        .find(|(_, save)| save.keeper == Some(with))
        .map(|(&building, _)| building)
}

/// Gives a shop's keeper something, from beside its door: it joins what the shop has,
/// which comes back, and the keeper remembers it. Selling, until there is payment.
pub(crate) fn give_to_keeper(
    client: &mut Client,
    world: &mut ServerWorld,
    item: Id,
    with: Id,
) -> Result<Vec<Item>, &'static str> {
    let character = client.character_id.ok_or("nobody to remember")?;
    let building = keeper_building(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    let town = visit.level.buildings.iter().find(|b| b.look.id == building);
    let door = town.ok_or("no such building here")?.look.door;
    let (x, y) = visit.position;
    if (x - door.0).abs().max((y - door.1).abs()) > DOOR_REACH {
        return Err("too far away");
    }
    let (id, kind, index) = (visit.id, visit.poi.kind, visit.index);
    let item = client
        .inventory
        .hand_over(item)
        .ok_or("can't hand that over")?;
    let level = world.site_mut(id, kind).levels.entry(index).or_default();
    let record = level.records.entry(with).or_default();
    record.memory.remember(character, Event::Gave(item.kind));
    record.carried.push(item);
    Ok(record.carried.clone())
}

/// What a keeper on the player's level has.
pub(crate) fn keeper_stock(
    client: &Client,
    world: &mut ServerWorld,
    with: Id,
) -> Option<Vec<Item>> {
    keeper_building(client, world, with)?;
    let visit = client.visit.as_ref()?;
    let level = world
        .site(visit.id, visit.poi.kind)
        .levels
        .get(&visit.index)?;
    Some(level.records.get(&with)?.carried.clone())
}

/// Takes something from a shop's keeper, from beside its door, if it is still there
/// and can go where it is asked to. Free, until there is payment.
pub(crate) fn take_from_keeper(
    client: &mut Client,
    world: &mut ServerWorld,
    with: Id,
    item: Id,
    (to, x, y, rotation): (shared::item::Holder, u8, u8, shared::item::Rotation),
) -> Result<(), &'static str> {
    let building = keeper_building(client, world, with).ok_or("no such keeper here")?;
    let visit = client.visit.as_ref().ok_or("not in a site")?;
    let town = visit.level.buildings.iter().find(|b| b.look.id == building);
    let door = town.ok_or("no such building here")?.look.door;
    let (at_x, at_y) = visit.position;
    if (at_x - door.0).abs().max((at_y - door.1).abs()) > DOOR_REACH {
        return Err("too far away");
    }
    let (id, kind, index) = (visit.id, visit.poi.kind, visit.index);
    let level = world.site_mut(id, kind).levels.entry(index).or_default();
    let carried = &mut level.records.entry(with).or_default().carried;
    let place = carried.iter().position(|one| one.id == item);
    let place = place.ok_or("they don't have that")?;
    let taken = carried[place].clone();
    client
        .inventory
        .add(taken, to, x, y, rotation)
        .map_err(|_| "it can't go there")?;
    carried.remove(place);
    Ok(())
}

/// Opens trading with a shop's keeper for a player beside its door: its stock is
/// caught up and sent. Anything else is refused, with no answer. `false` if the
/// client has gone.
pub(crate) fn open_trade(client: &mut Client, world: &mut ServerWorld, with: Id) -> bool {
    let Some(visit) = &client.visit else {
        warn!("Refused opening a trade from outside any site");
        return true;
    };
    let Some(building) = keeper_building(client, world, with) else {
        warn!("Refused opening a trade with someone who isn't here: {with:?}");
        return true;
    };
    let Some(town) = visit.level.buildings.iter().find(|b| b.look.id == building) else {
        warn!("Refused opening a building that isn't here: {building}");
        return true;
    };
    let (x, y) = visit.position;
    let door = town.look.door;
    if sells(town.look.kind).is_empty() || (x - door.0).abs().max((y - door.1).abs()) > DOOR_REACH {
        warn!("Refused opening {:?} from {x},{y}", town.look.kind);
        return true;
    }
    let of_kind = visit.level.buildings.iter();
    let of_kind = of_kind.filter(|b| b.look.kind == town.look.kind).count() as u32;
    let (kind, quality, trades) = (town.look.kind, town.quality, town.trades);
    let seed = visit.poi.seed.derive("shop");
    let seed = Seed::new(seed.at(&[visit.index as i64, building as i64]));
    let (id, poi_kind, index) = (visit.id, visit.poi.kind, visit.index);
    let now = world.clock.minutes();
    let made = world.made_now();
    let level = world
        .site_mut(id, poi_kind)
        .levels
        .entry(index)
        .or_default();
    let keeping = level.records.entry(with).or_default();
    let saved = level.buildings.entry(building).or_default();
    let shop = Shop {
        kind,
        quality,
        trades: trades + saved.trades,
        of_kind,
    };
    let stock: Vec<Quality> = keeping
        .carried
        .iter()
        .map(|item| item.quality.unwrap_or_default())
        .collect();
    // The clock can be moved back (dev tools).
    let last_visit = saved.last_interaction.map(|at| at.max(0.0) as u64);
    let restocked = catch_up(
        seed,
        &shop,
        &stock,
        saved.last_made,
        last_visit,
        made.minute,
    );
    let mut items: Vec<Item> = restocked
        .keep
        .iter()
        .map(|&index| keeping.carried[index].clone())
        .collect();
    items.extend(restocked.new.iter().map(|new| {
        let mut item = Item {
            quality: new.quality,
            made_at: Some(MadeAt {
                minute: new.minute,
                ..made
            }),
            ..Item::new(new_id(), new.kind)
        };
        give_form(&mut item, None, &mut new_id);
        item.roll_secrets();
        item
    }));
    keeping.carried.clone_from(&items);
    saved.last_made = restocked.last_made;
    saved.last_interaction = Some(now);
    let items = client.as_seen(items);
    client.link.send(&ServerMessage::TradeStock { with, items })
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
            let (visit_id, from_level) = (visit.id, visit.index);
            let new = NewVisit {
                poi: visit.poi,
                id: visit.id,
                site: visit.site,
            };
            let arrived = enter_level(client, world, new, index, from, false);
            if let Some(there) = client.visit.as_ref().map(Visit::position) {
                let site = visit_id;
                let taken = crate::entities::StairsTaken {
                    who: client.entity_id(),
                    site,
                    from_level,
                    stairs: (x, y),
                    to_level: index,
                    arrives: there,
                };
                if !client.god_mode {
                    world.saw_stairs_taken(taken);
                }
            }
            arrived
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
    visit.view_from = None;
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
        let truesight = client.truesight;
        let mut chunks = level_chunks(level, &stairs, remembered, true, truesight);
        chunks.extend(level_chunks(level, &stairs, &neighbours, false, truesight));
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
    world.move_mob_aside((new.id, index), position);
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
    let view = Sight::new(&level.level);
    world.populate(new.id, new.poi, index, &level);
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
        view,
        view_from: None,
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
        let shops = visit.level.buildings.iter();
        let shops = shops.filter(|b| !sells(b.look.kind).is_empty());
        let keepers = NewKeepers {
            id: visit.id,
            kind: visit.poi.kind,
            index,
            shops: shops.map(|b| b.look.id).collect(),
        };
        let message = ServerMessage::Keepers(keepers_of(world, &keepers));
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

/// Developer tools: truesight has just changed. If it changes how this level looks,
/// it is sent again. Only then, as that closes whatever is open on it.
pub(crate) fn truesight_changed(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let level = &visit.level.level;
    let (min_x, min_y, width, height) = level.bounds();
    let hides_something = (min_y..min_y + height)
        .any(|y| (min_x..min_x + width).any(|x| level.cell(x, y) == Cell::SecretDoor));
    !hides_something || arrive_again(client, world)
}

/// Developer tools: the character knows the whole of the level they are on. A
/// secret door stays a wall, as it does when seen, unless they have truesight.
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
    let truesight = client.truesight;
    level_chunks(
        &visit.level.level,
        &visit.stairs,
        masks,
        remembered,
        truesight,
    )
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
    truesight: bool,
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
                // Hidden until found, but not from truesight.
                Cell::SecretDoor if !truesight => Cell::Wall,
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
mod tests;
