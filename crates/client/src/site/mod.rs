//! Moving between maps: entering a world map poi's site, taking stairs
//! between its levels, and leaving again. See `docs/context/client/site.md`.
//!
//! The server decides all of it: clicking a `Portal` token (a world map
//! poi) sends the player travelling to it, and arriving asks the server
//! to go in. Inside, arriving (by a click) on stairs or on outside ground at
//! the very edge of the level asks the server to use it (and on the hatched
//! border round a poi in the wilderness, to leave), unless the player is
//! fighting: then the Down and Up keys take the stairs once it is over. The server answers
//! with the level the player is now on and its cells as they come into
//! sight, or with where they are back on the world map; `dungeon_map` draws
//! the level.

mod buttons;
mod hall;
mod wilderness;

use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage};
use shared::site::leads_out;
use shared::spot::Spot;

use crate::common::{
    ClickMode, CurrentDungeon, CurrentWorld, GameView, MapPosition, OtherLevels, OwnVitals, Portal,
    SmoothPosition, SpotPosition, WorldReturn,
};
use crate::connection::{FromServer, ToServer};
use crate::player::{Arrived, Player, PlayerSystems, TravelTo};
use crate::view3d::{ClickedToken, View3dSystems};

pub struct SitePlugin;

impl Plugin for SitePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Heading>()
            .add_systems(
                Update,
                (
                    follow_clicked_portal
                        .in_set(PlayerSystems::Input)
                        .after(View3dSystems::Click),
                    buttons::down_or_up.in_set(PlayerSystems::Input),
                ),
            )
            // After everything else this frame, so the view being left
            // never sees the new position (the view changes next frame).
            .add_systems(
                PostUpdate,
                (
                    change_map,
                    hall::follow_player,
                    wilderness::follow_point,
                    use_on_arrival,
                )
                    .chain(),
            );
    }
}

/// The portal the player is travelling to, to use on arrival.
#[derive(Resource, Default)]
struct Heading(Option<Entity>);

/// A click on a portal's token: travel to it, to use it there. In Teleport
/// Mode (dev tools) it jumps there and goes in at once, as the full map does.
fn follow_clicked_portal(
    clicked: Res<ClickedToken>,
    mode: Res<ClickMode>,
    portals: Query<(&MapPosition, &Portal)>,
    mut heading: ResMut<Heading>,
    mut travel: MessageWriter<TravelTo>,
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(entity) = clicked.0 else {
        return;
    };
    let Ok((position, portal)) = portals.get(entity) else {
        return;
    };
    let (x, y) = (position.x, position.y);
    if *mode == ClickMode::Teleport {
        heading.0 = None;
        to_server.write(ToServer(ClientMessage::DevTeleport { x, y }));
        to_server.write(ToServer(match portal {
            Portal::Poi(poi) => ClientMessage::EnterSite { chunk: poi.chunk },
            Portal::Gate(gate) => ClientMessage::EnterGate {
                entrance: gate.entrance,
            },
        }));
        return;
    }
    heading.0 = Some(entity);
    travel.write(TravelTo { x, y });
}

/// When a journey ends: on the portal it was heading for, ask to enter it;
/// in a site, on stairs or on outside ground at the edge of the level, ask
/// to use them, and on the border round a poi in the wilderness, ask to
/// leave for the world map, unless fighting (the server would refuse).
#[allow(clippy::too_many_arguments)]
fn use_on_arrival(
    mut arrivals: MessageReader<Arrived>,
    view: Res<State<GameView>>,
    dungeon: Res<CurrentDungeon>,
    portals: Query<(&MapPosition, &Portal)>,
    mut heading: ResMut<Heading>,
    vitals: Res<OwnVitals>,
    time: Res<Time>,
    mut to_server: MessageWriter<ToServer>,
) {
    let Some(&Arrived { x, y }) = arrivals.read().last() else {
        return;
    };
    let target = heading.0.take().and_then(|entity| portals.get(entity).ok());
    match target {
        Some((position, Portal::Poi(poi))) if (position.x, position.y) == (x, y) => {
            // After the steps there, so the server has the player at it.
            to_server.write(ToServer(ClientMessage::EnterSite { chunk: poi.chunk }));
        }
        Some((position, Portal::Gate(gate))) if (position.x, position.y) == (x, y) => {
            to_server.write(ToServer(ClientMessage::EnterGate {
                entrance: gate.entrance,
            }));
        }
        _ => {
            if *view.get() != GameView::Dungeon || vitals.fighting(time.elapsed_secs_f64()) {
                return;
            }
            let on_stairs = dungeon.stairs.iter().any(|s| (s.x, s.y) == (x, y));
            // The wilderness's edge is only where the server's window ends.
            let edge = !dungeon.is_wilderness() && leads_out(&dungeon.level, x, y);
            if on_stairs || edge {
                to_server.write(ToServer(ClientMessage::UseExit));
            } else if dungeon.borders.contains(&(x, y)) {
                // The hatched border round a poi: back to the world map.
                to_server.write(ToServer(ClientMessage::Up));
            }
        }
    }
}

/// What the server says about maps: the player is on a new level (and its
/// cells), in the Hall of Worlds, or on the world map (out of a site, or moved by dev tools).
#[allow(clippy::too_many_arguments)]
fn change_map(
    mut from_server: MessageReader<FromServer>,
    mut dungeon: ResMut<CurrentDungeon>,
    mut others: ResMut<OtherLevels>,
    world: Option<Res<CurrentWorld>>,
    mut world_return: ResMut<WorldReturn>,
    mut next: ResMut<NextState<GameView>>,
    view: Res<State<GameView>>,
    player: Single<(&mut MapPosition, &mut SpotPosition, &mut SmoothPosition), With<Player>>,
) {
    let (mut player, mut spot, mut smooth) = player.into_inner();
    for FromServer(message) in from_server.read() {
        match message {
            // Joined (or joined again): where the player is on the world map,
            // and the way back to it if a level follows.
            &ServerMessage::Welcome { x, y, .. } => world_return.0 = MapPosition { x, y },
            ServerMessage::EnterLevel(info) => {
                if *view.get() == GameView::WorldMap {
                    world_return.0 = *player;
                }
                (*spot, *player, *smooth) = SpotPosition::place(info.arrival);
                others.entering(&mut dungeon, info);
                dungeon.enter(info.clone());
                next.set(GameView::Dungeon);
            }
            // Counted from `origin`, a multiple of the gates' spacing, so the
            // cells are the same.
            &ServerMessage::EnterHall { at, origin } => {
                if *view.get() == GameView::WorldMap {
                    world_return.0 = *player;
                }
                (*spot, *player, *smooth) = SpotPosition::place(at);
                let (x, y) = at.cell();
                *others = OtherLevels::default();
                hall::show_around(&mut dungeon, (x, y), origin);
                next.set(GameView::Dungeon);
            }
            ServerMessage::Buildings(buildings) => dungeon.add_buildings(buildings),
            ServerMessage::Fields(fields) => dungeon.fields.clone_from(fields),
            ServerMessage::Walls(walls) => dungeon.walls.clone_from(walls),
            ServerMessage::Borders(cells) => dungeon.borders = cells.iter().copied().collect(),
            ServerMessage::FlatGround(flats) => dungeon.add_flats(flats),
            ServerMessage::LevelCells(chunk) => dungeon.add(chunk),
            &ServerMessage::WildernessWindow { bounds } => dungeon.move_window(bounds),
            ServerMessage::InPoi { kind, name } => {
                dungeon.in_poi = *kind;
                dungeon.in_poi_name.clone_from(name);
            }
            ServerMessage::RememberedLevel(remembered) => others.add(&dungeon, remembered),
            &ServerMessage::OnWorldMap { x, y } => {
                *others = OtherLevels::default();
                // The world wraps east-west: stay in the same copy of the
                // planet the player went in from (or is on).
                let width = world.as_ref().map_or(i64::MAX, |w| w.surface.width());
                let from = match view.get() {
                    GameView::WorldMap => player.x,
                    _ => world_return.0.x,
                };
                let shift = (from - x + width / 2).div_euclid(width) * width;
                *player = MapPosition { x: x + shift, y };
                // (Moved on the world map: no need to set it up again.)
                if *view.get() != GameView::WorldMap {
                    next.set(GameView::WorldMap);
                }
            }
            &ServerMessage::Teleported { x, y } => {
                // Dev tools, on the map the player is on: on the world map,
                // on the copy of the planet they are on; on a level, in units.
                if *view.get() != GameView::WorldMap {
                    (*spot, *player, *smooth) = SpotPosition::place(Spot::new(x, y));
                    continue;
                }
                let shift = world.as_ref().map_or(0, |world| {
                    let width = world.surface.width();
                    (player.x - x + width / 2).div_euclid(width) * width
                });
                *player = MapPosition { x: x + shift, y };
            }
            _ => {}
        }
    }
}
