//! The client's connection to the server. See
//! `docs/context/client/connection.md`.
//!
//! Native clients start an embedded server for single player (on its own
//! thread, with its own Bevy world) and talk to it through an in-process
//! `Link`, using the same messages a network connection will. The web client
//! has no server to talk to until networking exists, so it stays
//! `GameView::Connecting`.
//!
//! On the world map it reports where the player is (TEMPORARY, until
//! movement goes through the server), and the server sends the chunks
//! coming into sight.

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
        app.insert_resource(ServerConnection(server::start_embedded(
            shared::world::WorldSeed::new(server::DEFAULT_WORLD_SEED),
        )));

        app.add_systems(
            Startup,
            say_hello.run_if(resource_exists::<ServerConnection>),
        )
        .add_systems(
            PreUpdate,
            receive.run_if(resource_exists::<ServerConnection>),
        )
        .add_systems(
            Update,
            report_world_map_position
                .after(PlayerSystems::Travel)
                .run_if(resource_exists::<ServerConnection>)
                .run_if(in_state(GameView::WorldMap)),
        );
    }
}

/// The link to the server. Removed if the connection is lost.
// The web client never makes one yet.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Resource)]
pub struct ServerConnection(ClientLink);

fn say_hello(connection: Res<ServerConnection>) {
    connection.0.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    });
}

/// Handles every message that has arrived since the last frame.
fn receive(
    mut commands: Commands,
    connection: Res<ServerConnection>,
    mut world: Option<ResMut<CurrentWorld>>,
    mut next_view: ResMut<NextState<GameView>>,
) {
    loop {
        match connection.0.receive() {
            Ok(Some(ServerMessage::Welcome(info))) => {
                if info.world_map_version != WORLD_MAP_VERSION {
                    error!(
                        "The world needs world map version {}; this client has {WORLD_MAP_VERSION}",
                        info.world_map_version
                    );
                    continue;
                }
                info!("Joined world {:?}", info.id);
                commands.insert_resource(CurrentWorld::new(&info));
                next_view.set(GameView::WorldMap);
            }
            Ok(Some(ServerMessage::Refused(reason))) => {
                error!("The server refused to connect: {reason}");
            }
            Ok(Some(ServerMessage::WorldChunk(chunk))) => match world.as_mut() {
                Some(world) => world.features.insert(chunk.chunk, chunk.feature),
                None => warn!("A world map chunk arrived before the world"),
            },
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
    connection: Res<ServerConnection>,
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
        connection.0.send(&ClientMessage::WorldMapPosition {
            x: player.x,
            y: player.y,
        });
        *reported = Some(**player);
    }
}
