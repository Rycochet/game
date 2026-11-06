//! Exploring the wilderness from the world map, arriving in it, and moving
//! the window along as the player walks.

use bevy::prelude::*;
use shared::dungeon::{Level, Sight};
use shared::protocol::{LevelInfo, LevelKind, ServerMessage};

use super::window::window;
use crate::ServerWorld;
use crate::connections::Client;
use crate::sites::remembered::send_remembered;
use crate::sites::{Place, Visit, update_sight};
use crate::storage::{CellMasks, SiteId};

/// How far round where the player would arrive a walkable cell is looked for.
const ARRIVAL_SEARCH: i64 = 48;

/// The player explores the wilderness where they stand on the world map:
/// refused in water, a site or the Hall.
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
    let at = world.wilderness.wild.centre_of(px, py);
    enter_wilderness(client, world, at)
}

/// Puts the player in the wilderness at `at`, or the walkable cell nearest it.
pub(in crate::sites) fn enter_wilderness(
    client: &mut Client,
    world: &mut ServerWorld,
    (x, y): (i64, i64),
) -> bool {
    if let Err(error) = world.open_site(SiteId::Wilderness) {
        return client.refuse_for_saves(&error);
    }
    let x = world.wilderness.wild.wrap(x);
    let made = window(world, (x, y));
    let level = &made.level.level;
    let position = nearest_walkable(level, (x, y));
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
        },
        id: SiteId::Wilderness,
        index: 0,
        level: made.level,
        stairs: made.stairs,
        position,
        sent: CellMasks::new(),
        mirrors: made.mirrors,
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
    };
    client.changed_map();
    if !client.link.send(&ServerMessage::EnterLevel(info)) || !send_remembered(client, &mut visit) {
        return false;
    }
    client.character.world_position = world.wilderness.wild.point_of(position.0, position.1);
    client.character.hall = None;
    client.visit = Some(visit);
    client.unsaved = true;
    update_sight(client)
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
    let (x, y) = visit.position;
    if !(0..width).contains(&x) {
        return enter_wilderness(client, world, (x, y));
    }
    let wild = &world.wilderness.wild;
    client.character.world_position = wild.point_of(x, y);
    if wild.block_of(x, y) == centre {
        return true;
    }
    let made = window(world, (x, y));
    let Some(visit) = &mut client.visit else {
        return true;
    };
    let level = &made.level.level;
    visit.looked_from = Sight::new(level);
    visit.loaded = Sight::new(level);
    visit.view = Sight::new(level);
    visit.view_from = None;
    visit.place = Place::Wilderness {
        centre: made.centre,
        width,
        cutouts: made.cutouts,
    };
    visit.level = made.level;
    visit.stairs = made.stairs;
    visit.mirrors = made.mirrors;
    let bounds = visit.level.level.bounds();
    if !client
        .link
        .send(&ServerMessage::WildernessWindow { bounds })
    {
        return false;
    }
    let mut visit = client.visit.take().expect("checked above");
    let sent = send_remembered(client, &mut visit);
    client.visit = Some(visit);
    sent && update_sight(client)
}

/// The walkable cell nearest `at` on `level` (by rings round it), or `at`.
fn nearest_walkable(level: &Level, (x, y): (i64, i64)) -> (i64, i64) {
    (0..=ARRIVAL_SEARCH)
        .flat_map(|ring| {
            (-ring..=ring).flat_map(move |dy| {
                (-ring..=ring)
                    .filter(move |dx| dx.abs() == ring || dy.abs() == ring)
                    .map(move |dx| (x + dx, y + dy))
            })
        })
        .find(|&(cx, cy)| level.is_walkable(cx, cy))
        .unwrap_or((x, y))
}
