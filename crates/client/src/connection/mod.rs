//! The client's connection to the server. See
//! `docs/context/client/connection.md`.
//!
//! Native clients start an embedded server for single player (on its own
//! thread, with its own Bevy world) and talk to it through an in-process
//! `Link`, using the same messages a network connection will. The web client
//! has no server to talk to until networking exists, so it stays
//! `GameView::Connecting`. The embedded server saves in its own data folder;
//! on quitting, the client hangs up and waits for it to save.
//!
//! It sends the player's steps (`player::Stepped`); the server checks them,
//! sends what comes into sight and says where it has the player, and the
//! player is moved back if a step was refused (`prediction`). Other modules
//! send with `ToServer` and read the server's map messages as `FromServer`.

mod prediction;

use bevy::prelude::*;
use shared::protocol::{ClientLink, ClientMessage, PROTOCOL_VERSION, ServerMessage};
use shared::world::WORLD_MAP_VERSION;

use crate::common::{CurrentWorld, GameView, MapPosition};
use crate::player::{Player, PlayerSystems, Stepped};
use prediction::{Prediction, correction};

pub struct ConnectionPlugin;

impl Plugin for ConnectionPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (link, embedded) = server::start_embedded(
                shared::world::WorldSeed::new(server::DEFAULT_WORLD_SEED),
                Some(server::default_data_folder()),
            );
            app.insert_resource(ServerConnection {
                link,
                _embedded: Some(embedded),
            });
        }

        app.add_message::<ToServer>()
            .add_message::<FromServer>()
            .init_resource::<Prediction>()
            .add_systems(
                Startup,
                say_hello.run_if(resource_exists::<ServerConnection>),
            )
            .add_systems(
                PreUpdate,
                receive.run_if(resource_exists::<ServerConnection>),
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
    }
}

/// A message for the server, sent at the end of the frame.
#[derive(Message, Clone, Debug)]
pub struct ToServer(pub ClientMessage);

/// A message from the server about maps (`EnterLevel`, `LevelCells`,
/// `OnWorldMap`, `Teleported`), for `site`. The rest are handled here.
#[derive(Message, Clone, Debug)]
pub struct FromServer(pub ServerMessage);

/// The link to the server. Removed if the connection is lost, and on
/// quitting.
// The web client never makes one yet.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Resource)]
pub struct ServerConnection {
    link: ClientLink,
    /// The single-player server, if it is ours. Dropping it waits for it to
    /// save and stop, so it must come after `link` (fields are dropped in
    /// order): the server stops once the link is gone.
    #[cfg(not(target_arch = "wasm32"))]
    _embedded: Option<server::EmbeddedServer>,
}

fn say_hello(connection: Res<ServerConnection>) {
    connection.link.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    });
}

/// Handles every message that has arrived since the last frame.
#[allow(clippy::too_many_arguments)]
fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut world: Option<ResMut<CurrentWorld>>,
    view: Res<State<GameView>>,
    mut next_view: ResMut<NextState<GameView>>,
    mut from_server: MessageWriter<FromServer>,
    mut prediction: ResMut<Prediction>,
    mut player: Query<&mut MapPosition, With<Player>>,
) {
    // A world joined now: its chunks follow straight away, before the
    // resource can be added.
    let mut joined: Option<CurrentWorld> = None;
    loop {
        match connection.link.receive() {
            Ok(Some(ServerMessage::Welcome { world: info, x, y })) => {
                if info.world_map_version != WORLD_MAP_VERSION {
                    error!(
                        "The world needs world map version {}; this client has {WORLD_MAP_VERSION}",
                        info.world_map_version
                    );
                    continue;
                }
                info!("Joined world {:?}", info.id);
                joined = Some(CurrentWorld::new(&info));
                // Where the player left off (`EnterLevel` follows if they
                // were in a site).
                for mut position in &mut player {
                    *position = MapPosition { x, y };
                }
                next_view.set(GameView::WorldMap);
            }
            Ok(Some(ServerMessage::Refused(reason))) => {
                error!("The server refused to connect: {reason}");
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
                error!("Lost the connection to the server ({error:?})");
                commands.remove_resource::<ServerConnection>();
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
    for &Stepped(step) in stepped.read() {
        to_server.write(ToServer(prediction.step(step)));
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
