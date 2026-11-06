//! Handling every message that has arrived from the server.

use bevy::platform::time::Instant;
use bevy::prelude::*;
use shared::protocol::{OnlineState, ServerMessage};
use shared::world::MINUTES_PER_HOUR;
use shared::world::WORLD_MAP_VERSION;

use super::ping::TimedPong;
use super::prediction::{Prediction, correction};
use super::{Connecting, FromServer, ServerConnection, give_up, hang_up, log_in};
use crate::common::{
    CurrentWorld, GameClock, GameView, Hosting, MakingCharacter, MapPosition, ServerClock,
    ServerStatus, WorldChanged,
};
use crate::player::Player;

/// Handles every message that has arrived since the last frame.
#[allow(clippy::too_many_arguments)]
pub(super) fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut connecting: ResMut<Connecting>,
    mut status: ResMut<ServerStatus>,
    mut world: Option<ResMut<CurrentWorld>>,
    view: Res<State<GameView>>,
    mut next_view: ResMut<NextState<GameView>>,
    // Together, as Bevy limits how many things a system can ask for.
    (mut from_server, mut world_changed): (MessageWriter<FromServer>, MessageWriter<WorldChanged>),
    mut prediction: ResMut<Prediction>,
    mut clock: ResMut<GameClock>,
    mut server_dev_tools: ResMut<crate::common::ServerDevTools>,
    mut server_clock: ResMut<ServerClock>,
    time: Res<Time<Real>>,
    mut hosting: ResMut<Hosting>,
    mut player: Query<&mut MapPosition, With<Player>>,
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
            Ok(Some(ServerMessage::LoginOptions { methods })) => {
                if let Err(reason) = log_in(&connection, &mut connecting, &methods) {
                    give_up(&mut commands, &mut status, reason);
                    break;
                }
            }
            // A new character, to be made with `character_create`'s screen.
            Ok(Some(ServerMessage::Characters { list, can_create }))
                if list.is_empty() && can_create =>
            {
                commands.insert_resource(MakingCharacter(true));
            }
            // Nothing logs in to an account with characters yet: choosing one is
            // to come, before making one.
            Ok(Some(ServerMessage::Characters { .. })) => {
                let reason = "This version can't choose a character on this server".to_owned();
                give_up(&mut commands, &mut status, reason);
                break;
            }
            Ok(Some(ServerMessage::Welcome {
                name,
                world: info,
                time: world_time,
                server_clock: seconds,
                x,
                y,
                dev_tools,
                can_pause,
                stacks,
            })) => {
                let minutes = world_time.minutes(seconds);
                server_dev_tools.0 = dev_tools;
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
                for mut position in &mut player {
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
                    dev_tools,
                    can_pause,
                    stacks,
                }));
            }
            Ok(Some(ServerMessage::Online(state))) => {
                // Nobody pauses a world others are playing in.
                server_clock.can_pause = !matches!(state, OnlineState::Online { .. });
                hosting.0 = state;
            }
            // On purpose: not worth trying again.
            Ok(Some(ServerMessage::GoingOffline(name))) => {
                let reason =
                    format!("{name} has gone offline: its host closed it to other players.");
                give_up(&mut commands, &mut status, reason);
                break;
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
                    Some(world) => world.pois.insert(chunk.chunk, chunk.poi),
                    None => warn!("A world map chunk arrived before the world"),
                }
            }
            Ok(Some(ServerMessage::Road(road))) => match joined.as_mut().or(world.as_deref_mut()) {
                Some(world) => world.roads.insert(road),
                None => warn!("A road arrived before the world"),
            },
            Ok(Some(ServerMessage::Position { map, step, x, y })) => {
                let Some(predicted) = prediction.confirmed(map, step, (x, y)) else {
                    continue;
                };
                // The world map wraps; site levels don't.
                let width = match view.get() {
                    GameView::WorldMap => world.as_ref().map(|w| w.surface.width()),
                    _ => None,
                };
                for mut position in &mut player {
                    let shown = (position.x, position.y);
                    if let Some((x, y)) = correction(shown, predicted, width) {
                        info!("The server has the player at ({x}, {y}): a step was refused");
                        *position = MapPosition { x, y };
                    }
                }
            }
            Ok(Some(
                message @ (ServerMessage::EnterLevel(_)
                | ServerMessage::WildernessWindow { .. }
                | ServerMessage::InPoi(_)
                | ServerMessage::EnterHall { .. }
                | ServerMessage::Buildings(_)
                | ServerMessage::Fields(_)
                | ServerMessage::LevelCells(_)
                | ServerMessage::RememberedLevel(_)
                | ServerMessage::OnWorldMap { .. }
                | ServerMessage::Teleported { .. }
                | ServerMessage::Inventory(_)
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
                | ServerMessage::EntitySeen(_)
                | ServerMessage::EntityMoved { .. }
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
