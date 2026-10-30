//! The client's connection to the server. See
//! `docs/context/client/connection.md`.
//!
//! Native clients start an embedded server for single player (on its own
//! thread, with its own Bevy world) and talk to it through an in-process
//! `Link`, using the same messages a network connection carries; the web
//! client connects to the server it was loaded from over a WebSocket
//! (`socket`). A lost connection is tried again every few seconds
//! (`ServerStatus` says how it is going). The embedded server saves in its own
//! data folder; on quitting, the client hangs up and waits for it to save.
//!
//! It sends the player's steps (`player::Stepped`); the server checks them,
//! sends what comes into sight and says where it has the player, and the
//! player is moved back if a step was refused (`prediction`). Other modules
//! send with `ToServer` and read the server's map messages as `FromServer`.

mod prediction;
mod socket;

use std::time::Duration;

use bevy::prelude::*;
use shared::protocol::{ClientLink, ClientMessage, PROTOCOL_VERSION, ServerMessage};
use shared::world::WORLD_MAP_VERSION;

use crate::common::{CurrentWorld, GameClock, GameView, MapPosition, ServerStatus, Weather};
use crate::player::{Player, PlayerSystems, Stepped};
use prediction::{Prediction, correction};
use socket::Target;

/// How long to wait before trying to connect again.
const RETRY_EVERY: Duration = Duration::from_secs(2);

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ToServer>()
            .add_message::<FromServer>()
            .init_resource::<Prediction>()
            .insert_resource(Connecting {
                target: Target::choose(),
                guest: socket::guest(),
                retry: None,
            })
            .add_systems(
                PreUpdate,
                (
                    connect.run_if(not(resource_exists::<ServerConnection>)),
                    receive.run_if(resource_exists::<ServerConnection>),
                )
                    .chain(),
            )
            .add_systems(Update, send_steps.after(PlayerSystems::Travel))
            // After this frame's steps, which were still on the last map.
            .add_systems(PostUpdate, count_map_changes)
            // Last, so everything written this frame goes, in order.
            .add_systems(
                Last,
                (send, hang_up_on_exit)
                    .chain()
                    .run_if(resource_exists::<ServerConnection>),
            );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Last, pump_socket.after(send));
    }
}

/// A message for the server, sent at the end of the frame.
#[derive(Message, Clone, Debug)]
pub struct ToServer(pub ClientMessage);

/// A message from the server about maps (`Welcome`, `EnterLevel`,
/// `LevelCells`, `OnWorldMap`, `Teleported`), for `site`. The rest are
/// handled here.
#[derive(Message, Clone, Debug)]
pub struct FromServer(pub ServerMessage);

/// The link to the server. Removed if the connection is lost (and tried
/// again), and on quitting.
#[derive(Resource)]
pub struct ServerConnection {
    link: ClientLink,
    /// The single-player server, if it is ours. Dropping it waits for it to
    /// save and stop, so it must come after `link` (fields are dropped in
    /// order): the server stops once the link is gone.
    #[cfg(not(target_arch = "wasm32"))]
    _embedded: Option<server::EmbeddedServer>,
}

/// How to connect, and when to try again.
#[derive(Resource)]
struct Connecting {
    target: Target,
    guest: u64,
    /// Counting down to the next try; `None` to try now.
    retry: Option<Timer>,
}

/// Connects (or tries again, every `RETRY_EVERY`), and says hello: the
/// link holds what is sent until the socket is open.
fn connect(world: &mut World) {
    if matches!(world.resource::<ServerStatus>(), ServerStatus::Refused(_)) {
        return;
    }
    let delta = world.resource::<Time>().delta();
    let mut connecting = world.resource_mut::<Connecting>();
    if let Some(retry) = &mut connecting.retry
        && !retry.tick(delta).is_finished()
    {
        return;
    }
    connecting.retry = Some(Timer::new(RETRY_EVERY, TimerMode::Once));
    let guest = connecting.guest;
    let connection = match &connecting.target {
        #[cfg(not(target_arch = "wasm32"))]
        Target::Embedded => {
            let (link, embedded) = server::start_embedded(
                shared::world::WorldSeed::new(server::DEFAULT_WORLD_SEED),
                Some(server::default_data_folder()),
            );
            ServerConnection {
                link,
                _embedded: Some(embedded),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        Target::Url(url) => {
            info!("Connecting to {url}");
            ServerConnection {
                link: socket::open(url),
                _embedded: None,
            }
        }
        #[cfg(target_arch = "wasm32")]
        Target::Url(url) => {
            info!("Connecting to {url}");
            let Some((link, socket)) = socket::BrowserSocket::open(url) else {
                warn!("Can't open a WebSocket to {url}");
                return;
            };
            world.insert_non_send(socket);
            ServerConnection { link }
        }
    };
    connection.link.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
        guest,
    });
    world.insert_resource(connection);
    // Steps start again from the server's word.
    world.insert_resource(Prediction::default());
}

/// Web: sends what the link has sent; drops the socket once it has closed,
/// so the link reports the connection lost.
#[cfg(target_arch = "wasm32")]
fn pump_socket(world: &mut World) {
    let open = world
        .get_non_send::<socket::BrowserSocket>()
        .is_none_or(|socket| socket.pump());
    if !open {
        world.remove_non_send::<socket::BrowserSocket>();
    }
}

/// Handles every message that has arrived since the last frame.
#[allow(clippy::too_many_arguments)]
fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut status: ResMut<ServerStatus>,
    mut world: Option<ResMut<CurrentWorld>>,
    view: Res<State<GameView>>,
    mut next_view: ResMut<NextState<GameView>>,
    mut from_server: MessageWriter<FromServer>,
    mut prediction: ResMut<Prediction>,
    mut clock: ResMut<GameClock>,
    mut weather: ResMut<Weather>,
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
            Ok(Some(ServerMessage::SnowLine(snow_line))) => {
                weather.set_if_neq(Weather {
                    snow_line: snow_line as f64,
                });
            }
            Ok(Some(ServerMessage::Welcome {
                world: info,
                hour,
                snow_line,
                x,
                y,
            })) => {
                clock.set_if_neq(GameClock { hour });
                weather.set_if_neq(Weather {
                    snow_line: snow_line as f64,
                });
                if info.world_map_version != WORLD_MAP_VERSION {
                    error!(
                        "The world needs world map version {}; this client has {WORLD_MAP_VERSION}",
                        info.world_map_version
                    );
                    continue;
                }
                info!("Joined world {:?}", info.id);
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
                    world: info,
                    hour,
                    snow_line,
                    x,
                    y,
                }));
            }
            Ok(Some(ServerMessage::Refused(reason))) => {
                error!("The server refused to connect: {reason}");
                *status = ServerStatus::Refused(reason);
            }
            Ok(Some(ServerMessage::WorldChunk(chunk))) => {
                match joined.as_mut().or(world.as_deref_mut()) {
                    Some(world) => world.features.insert(chunk.chunk, chunk.feature),
                    None => warn!("A world map chunk arrived before the world"),
                }
            }
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
                | ServerMessage::LevelCells(_)
                | ServerMessage::OnWorldMap { .. }
                | ServerMessage::Teleported { .. }),
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
                commands.remove_resource::<ServerConnection>();
                #[cfg(target_arch = "wasm32")]
                commands.queue(|world: &mut World| {
                    world.remove_non_send::<socket::BrowserSocket>();
                });
                break;
            }
        }
    }
    if let Some(world) = joined {
        commands.insert_resource(world);
    }
}

/// Sends the steps the player took this frame, keeping them until the
/// server confirms them.
fn send_steps(
    mut stepped: MessageReader<Stepped>,
    mut prediction: ResMut<Prediction>,
    mut to_server: MessageWriter<ToServer>,
) {
    for &Stepped { step, moved } in stepped.read() {
        to_server.write(ToServer(prediction.step(step, moved)));
    }
}

/// The server has put the player on another map (applied by `site` this
/// frame): steps from now on are on it.
fn count_map_changes(
    mut from_server: MessageReader<FromServer>,
    mut prediction: ResMut<Prediction>,
) {
    for FromServer(message) in from_server.read() {
        if matches!(
            message,
            ServerMessage::EnterLevel(_)
                | ServerMessage::OnWorldMap { .. }
                | ServerMessage::Teleported { .. }
        ) {
            prediction.changed_map();
        }
    }
}

/// Sends what other systems wrote this frame.
fn send(connection: Res<ServerConnection>, mut to_server: MessageReader<ToServer>) {
    for message in to_server.read() {
        connection.link.send(&message.0);
    }
}

/// Quitting: hangs up, so the server knows the player has gone (and an
/// embedded one saves and stops; this waits for it).
fn hang_up_on_exit(mut commands: Commands, mut exit: MessageReader<AppExit>) {
    if exit.read().next().is_some() {
        commands.remove_resource::<ServerConnection>();
    }
}
