//! Exploring the wilderness from the world map, arriving in it, and moving
//! the window along as the player walks.

use bevy::prelude::*;
use shared::dungeon::{Cell, Level, Sight};
use shared::protocol::{LevelInfo, LevelKind, ServerMessage, WORLD_MAP_PRELOAD, WORLD_MAP_SIGHT};
use shared::site::{BuildingLook, FieldLook};
use shared::spot::{CELL_SPAN, Spot};
use shared::wilderness::gates_of;

use super::arrival::arrival;
use super::found::{found_by_standing, send_borders};
use super::in_poi::{poi_here, tell_of, tell_poi};
use super::town::send_town;
use super::window::window;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::remembered::send_remembered;
use crate::sites::{Mirror, Place, SetIn, Visit, keep_within, keepers_in_window, update_sight};
use crate::storage::{CellMasks, SiteId};

/// How far round where the player would arrive a walkable cell is looked for.
const ARRIVAL_SEARCH: i64 = 48;

/// The player explores the wilderness where they stand on the world map
/// (outside a poi they are at: `arrival`): refused in water, a site or the
/// Hall.
pub(crate) fn explore(client: &mut Client, world: &mut ServerWorld) -> bool {
    if client.visit.is_some() || client.character.hall.is_some() {
        warn!("A client tried to explore from inside somewhere");
        return true;
    }
    let (px, py) = client.world_position();
    if world.ground.elevation(px, py, world.conditions()).is_none() {
        info!("Refused exploring from water at ({px}, {py})");
        return true;
    }
    let at = arrival(client, world);
    enter_wilderness(client, world, Spot::middle_of(at))
}

/// Puts the player in the wilderness at `at`, or the walkable cell nearest it.
pub(in crate::sites) fn enter_wilderness(
    client: &mut Client,
    world: &mut ServerWorld,
    at: Spot,
) -> bool {
    if let Err(error) = world.open_site(SiteId::Wilderness) {
        return client.refuse_for_saves(&error);
    }
    let (x, y) = at.cell();
    let wrapped = world.wilderness.wild.wrap(x);
    let at = at.offset((wrapped - x) * CELL_SPAN, 0);
    let x = wrapped;
    let mut made = window(world, (x, y));
    let looks = Looks::of(&mut made);
    let level = &made.level.level;
    let cell = nearest_walkable(level, (x, y));
    // Where they were, unless that has no ground now.
    let position = if cell == (x, y) {
        at
    } else {
        Spot::middle_of(cell)
    };
    let width = world.wilderness.wild.width();
    let mut visit = Visit {
        looked_from: Sight::new(level),
        loaded: Sight::new(level),
        view: Sight::new(level),
        view_from: None,
        place: Place::Wilderness {
            centre: made.centre,
            width,
            cutouts: made.cutouts,
            in_poi: poi_here(world, cell),
        },
        id: SiteId::Wilderness,
        index: 0,
        level: made.level,
        stairs: made.stairs,
        doors: made.doors,
        position,
        sent: CellMasks::new(),
        mirrors: made.mirrors,
        wild_area: 0,
    };
    let info = LevelInfo {
        kind: LevelKind::Wilderness,
        ruined: false,
        building: false,
        index: 0,
        levels: 1,
        basement: false,
        bounds: visit.level.level.bounds(),
        arrival: position,
        entering_site: true,
        place_name: None,
        ground: shared::relief::LevelGround::Wilderness,
    };
    client.changed_map();
    let in_poi = poi_here(world, cell);
    if !client.link.send(&ServerMessage::EnterLevel(info))
        || !tell_of(client, world, in_poi)
        || !send_looks(client, world, looks, (&visit.mirrors, set_ins(&visit)))
        || !send_remembered(client, &mut visit)
    {
        return false;
    }
    ask_roads(world, cell);
    client.character.world_position = world.wilderness.wild.point_of(cell.0, cell.1);
    client.character.hall = None;
    client.visit = Some(visit);
    client.unsaved = true;
    send_town(client) && update_sight(client) && found_by_standing(client)
}

/// After a step in the wilderness: keeps the window round the player, and
/// their place on the world map with them. Over the planet's seam, they are
/// put where it is round the planet, as on a new map.
pub(crate) fn follow(client: &mut Client, world: &mut ServerWorld) -> bool {
    let Some(visit) = &client.visit else {
        return true;
    };
    let Place::Wilderness { centre, width, .. } = visit.place else {
        return true;
    };
    let (x, y) = visit.position();
    if !(0..width).contains(&x) {
        return enter_wilderness(client, world, visit.spot());
    }
    let wild = &world.wilderness.wild;
    client.character.world_position = wild.point_of(x, y);
    let same_block = wild.block_of(x, y) == centre;
    if visit.level.level.cell(x, y) == Cell::WorldGate {
        return through_gate(client, world, (x, y));
    }
    if !tell_poi(client, world) || !found_by_standing(client) {
        return false;
    }
    if same_block {
        return send_town(client);
    }
    let mut made = window(world, (x, y));
    let looks = Looks::of(&mut made);
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let level = &made.level.level;
    visit.looked_from = Sight::new(level);
    visit.loaded = Sight::new(level);
    visit.view = Sight::new(level);
    visit.view_from = None;
    let in_poi = match visit.place {
        Place::Wilderness { in_poi, .. } => in_poi,
        Place::Site { .. } => None,
    };
    visit.place = Place::Wilderness {
        centre: made.centre,
        width,
        cutouts: made.cutouts,
        in_poi,
    };
    visit.level = made.level;
    visit.stairs = made.stairs;
    visit.doors = made.doors;
    visit.mirrors = made.mirrors;
    let bounds = visit.level.level.bounds();
    // The client lets go of what is outside the window now.
    keep_within(&mut visit.sent, bounds);
    let mirrors = visit.mirrors.clone();
    let cutouts = set_ins(visit).to_vec();
    if !client
        .link
        .send(&ServerMessage::WildernessWindow { bounds })
        || !send_looks(client, world, looks, (&mirrors, &cutouts))
    {
        return false;
    }
    ask_roads(world, (x, y));
    let mut visit = client.visit.take().expect("checked above");
    let sent = send_remembered(client, &mut visit);
    client.visit = Some(visit);
    sent && send_town(client) && update_sight(client)
}

/// The player has stepped onto a World Gate's cell: through it into the Hall
/// of Worlds, as from the world map (coming back arrives on the world map).
/// A gate leading nowhere (a world that isn't one of the Hall's) does nothing.
fn through_gate(client: &mut Client, world: &mut ServerWorld, (x, y): (i64, i64)) -> bool {
    let wild = &world.wilderness.wild;
    let at = (wild.wrap(x), y);
    let gates = gates_of(wild, &world.pois);
    let Some(gate) = gates.iter().find(|gate| gate.cell == at) else {
        return true;
    };
    if world.gate(gate.entrance).is_none() {
        return true;
    }
    super::super::leave(client, world);
    client.visit = None;
    client.unsaved = true;
    crate::gates::enter_gate(client, world, gate.entrance)
}

/// The window's towns' buildings, who keeps their shops, farms' fields and
/// the borders round pois the player knows, before its cells (the client
/// marks the buildings' cells as theirs): always, so ones left behind go.
fn send_looks(
    client: &Client,
    world: &mut ServerWorld,
    made: Looks,
    (mirrors, cutouts): (&[Mirror], &[SetIn]),
) -> bool {
    let keepers = keepers_in_window(world, mirrors);
    client.link.send(&ServerMessage::Buildings(made.buildings))
        && client.link.send(&ServerMessage::Keepers(keepers))
        && client.link.send(&ServerMessage::Fields(made.fields))
        && send_borders(client, cutouts)
}

/// The pois set into a wilderness visit's window.
fn set_ins(visit: &Visit) -> &[SetIn] {
    match &visit.place {
        Place::Wilderness { cutouts, .. } => cutouts,
        Place::Site { .. } => &[],
    }
}

/// What is sent with each window besides its cells.
struct Looks {
    buildings: Vec<BuildingLook>,
    fields: Vec<FieldLook>,
}

impl Looks {
    fn of(made: &mut super::window::Window) -> Self {
        Looks {
            buildings: std::mem::take(&mut made.buildings),
            fields: std::mem::take(&mut made.fields),
        }
    }
}

/// Asks for the roads round the player, as the world map does round those on
/// it; the chunks they pass are made again with them as they come.
fn ask_roads(world: &ServerWorld, (x, y): (i64, i64)) {
    let (px, py) = world.wilderness.wild.point_of(x, y);
    let reach = WORLD_MAP_SIGHT + WORLD_MAP_PRELOAD;
    world.ask_roads((px - reach, py - reach), (px + reach, py + reach));
}

/// The walkable cell nearest `at` on `level` (by rings round it), or `at`;
/// never a World Gate's, which would take them straight through.
fn nearest_walkable(level: &Level, (x, y): (i64, i64)) -> (i64, i64) {
    (0..=ARRIVAL_SEARCH)
        .flat_map(|ring| {
            (-ring..=ring).flat_map(move |dy| {
                (-ring..=ring)
                    .filter(move |dx| dx.abs() == ring || dy.abs() == ring)
                    .map(move |dx| (x + dx, y + dy))
            })
        })
        .find(|&(cx, cy)| level.is_walkable(cx, cy) && level.cell(cx, cy) != Cell::WorldGate)
        .unwrap_or((x, y))
}
