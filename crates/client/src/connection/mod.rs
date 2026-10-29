//! The client's connection to the server. See
//! `docs/context/client/connection.md`.
//!
//! Native clients start an embedded server for single player (on its own
//! thread, with its own Bevy world) and talk to it through an in-process
//! `Link`, using the same messages a network connection will. The web client
//! has no server to talk to until networking exists, so it stays
//! `GameView::Connecting`.

use bevy::prelude::*;
use shared::protocol::{ClientLink, ClientMessage, PROTOCOL_VERSION, ServerMessage};
use shared::world::WORLD_MAP_VERSION;

use crate::common::{CurrentWorld, GameView};

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
            Ok(None) => return,
            Err(error) => {
                error!("Lost the connection to the server ({error:?})");
                commands.remove_resource::<ServerConnection>();
                return;
            }
        }
    }
}
