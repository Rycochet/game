//! Handling every message that has arrived from the server.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::protocol::{OnlineState, ServerMessage};
use shared::spot::Spot;
use shared::world::MINUTES_PER_HOUR;
use shared::world::WORLD_MAP_VERSION;

use super::ping::TimedPong;
use super::prediction::{Prediction, correction};
use super::{FromServer, ServerConnection, give_up, hang_up};
use crate::common::{
    CurrentWorld, GameClock, GameView, Hosting, MapPosition, ServerClock, ServerStatus,
    SmoothPosition, SpotPosition, WorldChanged,
};
use crate::player::Player;

/// Handles every message that has arrived since the last frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut status: ResMut<ServerStatus>,
    mut world: Option<ResMut<CurrentWorld>>,
    view: Res<State<GameView>>,
    mut next_view: ResMut<NextState<GameView>>,
    // Together, as Bevy limits how many things a system can ask for.
    (mut from_server, mut world_changed): (MessageWriter<FromServer>, MessageWriter<WorldChanged>),
    mut prediction: ResMut<Prediction>,
    mut clock: ResMut<GameClock>,
    mut debug_tools: ResMut<crate::common::DebugTools>,
    mut server_clock: ResMut<ServerClock>,
    time: Res<Time<Real>>,
    mut hosting: ResMut<Hosting>,
    mut player: Query<(&mut MapPosition, &mut SpotPosition, &mut SmoothPosition), With<Player>>,
) {
    // A world joined now: its chunks follow straight away, before the
    // resource can be added.
    let mut joined: Option<CurrentWorld> = None;
    loop {
        match connection.link.receive() {
            Ok(Some(ServerMessage::Hour(hour))) => {
                clock.set_if_neq(GameClock { hour });
            }
            Ok(Some(ServerMessage::WorldClock(world_time))) => {
                let seconds = server_clock.seconds;
                server_clock.set(seconds, world_time);
            }
            Ok(Some(ServerMessage::Paused {
                paused,
                clock: seconds,
            })) => {
                server_clock.paused = paused;
                let world_time = server_clock.time;
                server_clock.set(seconds, world_time);
                // An answer from before this could wrongly count the pause as
                // time passed.
                connection.pongs.take();
            }
            // Only from an embedded server: a socket takes them first.
            Ok(Some(ServerMessage::Pong { sent, clock })) => {
                connection.pongs.set(TimedPong {
                    round_trip: time.elapsed_secs_f64() - sent,
                    clock,
                    received: Instant::now(),
                });
            }
            Ok(Some(ServerMessage::Welcome {
                name,
                world: info,
                time: world_time,
                server_clock: seconds,
                x,
                y,
                debug_tools: tools,
                can_pause,
                stacks,
                role,
                powers,
                shown,
            })) => {
                let minutes = world_time.minutes(seconds);
                *debug_tools = crate::common::DebugTools::of(tools.iter().copied());
                server_clock.can_pause = can_pause;
                server_clock.paused = false;
                clock.set_if_neq(GameClock {
                    hour: (minutes / MINUTES_PER_HOUR) as u32,
                });
                server_clock.set(seconds, world_time);
                if info.world_map_version != WORLD_MAP_VERSION {
                    error!(
                        "The world needs world map version {}; this client has {WORLD_MAP_VERSION}",
                        info.world_map_version
                    );
                    continue;
                }
                info!("Joined world {:?}", info.id);
                // Moved to another while playing: the maps forget the last one.
                if world.as_ref().is_some_and(|world| world.id != info.id) {
                    world_changed.write(WorldChanged);
                }
                *status = ServerStatus::Connected;
                joined = Some(CurrentWorld::new(&info));
                // Where the player left off (`EnterLevel` follows if they
                // were in a site).
                for (mut position, _, _) in &mut player {
                    *position = MapPosition { x, y };
                }
                // (Connected again on the world map: nothing to set up.)
                if *view.get() != GameView::WorldMap {
                    next_view.set(GameView::WorldMap);
                }
                from_server.write(FromServer(ServerMessage::Welcome {
                    name,
                    world: info,
                    time: world_time,
                    server_clock: seconds,
                    x,
                    y,
                    debug_tools: tools,
                    can_pause,
                    stacks,
                    role,
                    powers,
                    shown,
                }));
            }
            Ok(Some(ServerMessage::Online(state))) => {
                // Nobody pauses a world others are playing in.
                server_clock.can_pause = !matches!(state, OnlineState::Online { .. });
                hosting.0 = state;
            }
            Ok(Some(ServerMessage::Lockdown(on))) => {
                commands.insert_resource(crate::common::Lockdown(Some(on)));
            }
            // On purpose: not worth trying again.
            Ok(Some(ServerMessage::GoingOffline(name))) => {
                let reason =
                    format!("{name} has gone offline: its host closed it to other players.");
                give_up(&mut commands, &mut status, reason);
                break;
            }
            // Logging in, an account's characters, and roles: read by `log_in`,
            // Settings and the Accounts panel.
            Ok(Some(
                message @ (ServerMessage::LoginOptions { .. }
                | ServerMessage::Characters { .. }
                | ServerMessage::Role(_)
                | ServerMessage::OpenLogin { .. }
                | ServerMessage::LoggedIn { .. }
                | ServerMessage::LoginFailed(_)),
            )) => {
                from_server.write(FromServer(message));
            }
            Ok(Some(ServerMessage::Refused(reason))) => {
                error!("The server refused to connect: {reason}");
                *status = ServerStatus::Refused(format!("The server refused: {reason}"));
            }
            Ok(Some(ServerMessage::ForgetWorldMap)) => {
                if let Some(world) = joined.as_mut().or(world.as_deref_mut()) {
                    world.pois.forget_all();
                }
            }
            Ok(Some(ServerMessage::WorldChunk(chunk))) => {
                match joined.as_mut().or(world.as_deref_mut()) {
                    Some(world) => world
                        .pois
                        .insert(chunk.chunk, chunk.poi, chunk.name.clone()),
                    None => warn!("A world map chunk arrived before the world"),
                }
            }
            Ok(Some(ServerMessage::Road(road))) => match joined.as_mut().or(world.as_deref_mut()) {
                Some(world) => world.roads.insert(road),
                None => warn!("A road arrived before the world"),
            },
            Ok(Some(ServerMessage::Position { map, step, x, y })) => {
                // On a level or in the Hall, in units.
                let Some(predicted) = prediction.confirmed(map, step, (x, y)) else {
                    continue;
                };
                // The world map wraps; site levels don't.
                let width = match view.get() {
                    GameView::WorldMap => world.as_ref().map(|w| w.surface.width()),
                    _ => None,
                };
                let on_level = prediction.on_level();
                for (mut position, mut spot, mut smooth) in &mut player {
                    let shown = match on_level {
                        true => (spot.0.x, spot.0.y),
                        false => (position.x, position.y),
                    };
                    let expected = prediction.expected(shown);
                    let Some((x, y)) = correction(expected, predicted, width) else {
                        continue;
                    };
                    info!("The server has the player at ({x}, {y}): a move was refused");
                    if on_level {
                        (*spot, *position, *smooth) = SpotPosition::place(Spot::new(x, y));
                    } else {
                        *position = MapPosition { x, y };
                    }
                }
            }
            Ok(Some(
                message @ (ServerMessage::EnterLevel(_)
                | ServerMessage::Difficulty { .. }
                | ServerMessage::WildernessWindow { .. }
                | ServerMessage::InPoi { .. }
                | ServerMessage::EnterHall { .. }
                | ServerMessage::Buildings(_)
                | ServerMessage::Fields(_)
                | ServerMessage::Borders(_)
                | ServerMessage::FlatGround(_)
                | ServerMessage::PoiFound { .. }
                | ServerMessage::LevelCells(_)
                | ServerMessage::RememberedLevel(_)
                | ServerMessage::OnWorldMap { .. }
                | ServerMessage::Teleported { .. }
                | ServerMessage::Inventory(_)
                | ServerMessage::NotWorn { .. }
                | ServerMessage::TradeStock { .. }
                | ServerMessage::ItemsLookedUp { .. }
                | ServerMessage::GroundItems { .. }
                | ServerMessage::GroundSeen { .. }
                | ServerMessage::GroundGone { .. }
                | ServerMessage::Keepers(_)
                | ServerMessage::Sheet(_)
                | ServerMessage::Vitals { .. }
                | ServerMessage::VisitedGates(_)
                | ServerMessage::ActionBar(_)
                | ServerMessage::Sneaking(_)
                | ServerMessage::DoorRefused { .. }
                | ServerMessage::AbilityStarted { .. }
                | ServerMessage::AbilityLanded { .. }
                | ServerMessage::Killed { .. }
                | ServerMessage::Harmed { .. }
                | ServerMessage::Lightning { .. }
                | ServerMessage::DevMobInfo { .. }
                | ServerMessage::DevAlignment(_)
                | ServerMessage::DevSkillsOf { .. }
                | ServerMessage::EntitySeen(_)
                | ServerMessage::Moves(_)
                | ServerMessage::Turned(_)
                | ServerMessage::Quest(_)
                | ServerMessage::EntityGone { .. }),
            )) => {
                from_server.write(FromServer(message));
            }
            Ok(None) => break,
            Err(error) => {
                if status.is_connected() {
                    warn!("Lost the connection to the server ({error:?})");
                }
                if !matches!(*status, ServerStatus::Refused(_)) {
                    *status = ServerStatus::Connecting;
                }
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(reason) = connection.refusal.take() {
                    warn!("Not connecting again: {reason}");
                    *status = ServerStatus::Refused(reason);
                }
                hang_up(&mut commands);
                break;
            }
        }
    }
    if let Some(world) = joined {
        commands.insert_resource(world);
    }
}
