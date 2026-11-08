//! The single-player server, embedded in a native client.

use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use shared::difficulty::Difficulties;
use shared::protocol::{ClientLink, local_pair};
use shared::world::WorldSeed;

use crate::debug_api::DebugApiPlugin;
use crate::{Connections, ServerOptions, headless_app};

/// A running embedded server. Dropping it waits for the server to finish:
/// drop the client's link first, so it sees its client leave, saves and
/// stops.
pub struct EmbeddedServer(Option<JoinHandle<()>>);

impl Drop for EmbeddedServer {
    fn drop(&mut self) {
        if let Some(thread) = self.0.take() {
            // A panic on the server thread was already logged.
            let _ = thread.join();
        }
    }
}

/// Starts a server on its own thread, with its own Bevy world, saving in
/// `data` (if any) and allowing these difficulties (Server Options), and
/// returns the client's end of its one connection. The client's log is used.
/// The server stops once that link is dropped.
pub fn start_embedded(
    world_seed: WorldSeed,
    data: Option<PathBuf>,
    difficulties: Difficulties,
) -> (ClientLink, EmbeddedServer) {
    let (client, server) = local_pair();
    let thread = thread::Builder::new()
        .name("server".into())
        .spawn(move || {
            // Single player: the player's own server, so dev tools are fine,
            // and it has nothing to do once the game has gone.
            let fixed = crate::difficulties_from_environment();
            let options = ServerOptions {
                dev_tools: true,
                exit_when_alone: true,
                pausable: true,
                guests: false,
                // Server Options' tickboxes, unless `HOW_DIFFICULTIES` says.
                difficulties: fixed.unwrap_or(difficulties),
                difficulties_fixed: fixed.is_some(),
                ..Default::default()
            };
            // Made on the thread itself: a Bevy `App` can't move between
            // threads.
            let mut app = headless_app(world_seed, options, data.clone());
            // Its player may open it to others.
            app.add_plugins(crate::OnlinePlugin { data });
            // The client's own debug API takes the usual port.
            app.add_plugins(DebugApiPlugin { port_offset: 1 });
            app.world_mut()
                .resource_mut::<Connections>()
                .add_single_player(server);
            app.run();
        })
        .expect("the server thread can start");
    (client, EmbeddedServer(Some(thread)))
}

#[cfg(test)]
mod tests;
