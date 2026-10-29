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
//! It reports where the player is (TEMPORARY, until movement goes through
//! the server), and the server sends what comes into sight. Other modules
//! send with `ToServer` and read the server's map messages as `FromServer`.

use bevy::prelude::*;
use shared::protocol::{
    ClientLink, ClientMessage, PROTOCOL_VERSION, ServerMessage, WORLD_MAP_REPORT_DISTANCE,
};
use shared::world::WORLD_MAP_VERSION;

use crate::common::{CurrentWorld, GameView, MapPosition};
use crate::player::{Player, PlayerSystems};

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
            .add_systems(
                Startup,
                say_hello.run_if(resource_exists::<ServerConnection>),
            )
            .add_systems(
                PreUpdate,
                receive.run_if(resource_exists::<ServerConnection>),
            )
            .add_systems(
                Update,
                (
                    report_world_map_position.run_if(in_state(GameView::WorldMap)),
                    report_site_position.run_if(in_state(GameView::Dungeon)),
                )
                    .after(PlayerSystems::Travel),
            )
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
/// `LeftSite`), for `site`. The rest are handled here.
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
fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut world: Option<ResMut<CurrentWorld>>,
    mut next_view: ResMut<NextState<GameView>>,
    mut from_server: MessageWriter<FromServer>,
    mut player: Query<&mut MapPosition, With<Player>>,
) {
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
                commands.insert_resource(CurrentWorld::new(&info));
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
            Ok(Some(ServerMessage::WorldChunk(chunk))) => match world.as_mut() {
                Some(world) => world.features.insert(chunk.chunk, chunk.feature),
                None => warn!("A world map chunk arrived before the world"),
            },
            Ok(Some(
                message @ (ServerMessage::EnterLevel(_)
                | ServerMessage::LevelCells(_)
                | ServerMessage::LeftSite { .. }),
            )) => {
                from_server.write(FromServer(message));
            }
            Ok(None) => return,
            Err(error) => {
                error!("Lost the connection to the server ({error:?})");
                commands.remove_resource::<ServerConnection>();
                return;
            }
        }
    }
}

/// Tells the server where the player is on the world map: on coming to it,
/// and whenever they are `WORLD_MAP_REPORT_DISTANCE` from the last report.
fn report_world_map_position(
    mut to_server: MessageWriter<ToServer>,
    player: Single<&MapPosition, With<Player>>,
    view: Res<State<GameView>>,
    mut reported: Local<Option<MapPosition>>,
) {
    // The view has changed since this last ran: back from a site, whose
    // positions are on another grid.
    if view.is_changed() {
        *reported = None;
    }
    let far = |last: &MapPosition| {
        (player.x - last.x).abs() >= WORLD_MAP_REPORT_DISTANCE
            || (player.y - last.y).abs() >= WORLD_MAP_REPORT_DISTANCE
    };
    if reported.as_ref().is_none_or(far) {
        to_server.write(ToServer(ClientMessage::WorldMapPosition {
            x: player.x,
            y: player.y,
        }));
        *reported = Some(**player);
    }
}

/// Tells the server where the player is in a site, whenever they move to
/// another cell.
fn report_site_position(
    mut to_server: MessageWriter<ToServer>,
    player: Single<Ref<MapPosition>, With<Player>>,
) {
    if player.is_changed() {
        to_server.write(ToServer(ClientMessage::SitePosition {
            x: player.x,
            y: player.y,
        }));
    }
}

/// Sends what other systems wrote this frame.
fn send(connection: Res<ServerConnection>, mut to_server: MessageReader<ToServer>) {
    for message in to_server.read() {
        connection.link.send(&message.0);
    }
}

/// Quitting: says exactly where the player is on the world map (it is only
/// reported every few steps), then hangs up, so the server knows the player
/// has gone (and an embedded one saves and stops; this waits for it).
fn hang_up_on_exit(
    mut commands: Commands,
    mut exit: MessageReader<AppExit>,
    connection: Res<ServerConnection>,
    view: Res<State<GameView>>,
    player: Single<&MapPosition, With<Player>>,
) {
    if exit.read().next().is_none() {
        return;
    }
    if *view.get() == GameView::WorldMap {
        connection.link.send(&ClientMessage::WorldMapPosition {
            x: player.x,
            y: player.y,
        });
    }
    commands.remove_resource::<ServerConnection>();
}
