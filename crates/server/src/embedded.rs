//! The single-player server, embedded in a native client.

use std::path::PathBuf;
use std::thread::{self, JoinHandle};

use shared::protocol::{ClientLink, local_pair};
use shared::world::WorldSeed;

use crate::{CharacterId, Connections, ServerOptions, headless_app};

/// The single player's character, who has no account (until there are
/// characters to choose from here).
const PLAYER: &str = "player";

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
/// `data` (if any), and returns the client's end of its one connection. The
/// client's log is used. The server stops once that link is dropped.
pub fn start_embedded(
    world_seed: WorldSeed,
    data: Option<PathBuf>,
) -> (ClientLink, EmbeddedServer) {
    let (client, server) = local_pair();
    let thread = thread::Builder::new()
        .name("server".into())
        .spawn(move || {
            // Single player: the player's own server, so dev tools are fine,
            // and it has nothing to do once the game has gone.
            let options = ServerOptions {
                dev_tools: true,
                exit_when_alone: true,
                guests: false,
            };
            // Made on the thread itself: a Bevy `App` can't move between
            // threads.
            let mut app = headless_app(world_seed, options, data);
            let character = CharacterId::new(PLAYER).expect("a fine id");
            app.world_mut()
                .resource_mut::<Connections>()
                .add_playing(server, character);
            app.run();
        })
        .expect("the server thread can start");
    (client, EmbeddedServer(Some(thread)))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerMessage};

    use super::*;
    use crate::ServerWorld;
    use crate::storage::tests::TestFolder;

    #[test]
    fn an_embedded_server_welcomes_its_client_and_saves_when_it_leaves() {
        let folder = TestFolder::new();
        let seed = WorldSeed::new(99);
        let (client, server) = start_embedded(seed, Some(folder.0.clone()));
        assert!(client.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
        }));
        assert!(client.send(&ClientMessage::DevTeleport { x: 5, y: 6 }));
        let deadline = Instant::now() + Duration::from_secs(10);
        let next_answer = || loop {
            if let Some(message) = client.receive().expect("still connected") {
                break message;
            }
            assert!(Instant::now() < deadline, "no answer from the server");
            thread::sleep(Duration::from_millis(10));
        };
        let world = ServerWorld::new(seed).info();
        // The clock has been running since the server started.
        let ServerMessage::Welcome { clock, .. } = next_answer() else {
            panic!("no welcome");
        };
        assert!((0.0..30.0).contains(&clock), "{clock}");
        let _ = world;
        // Not leaving before the teleport is dealt with: a server still
        // welcoming a client that has gone doesn't read what it sent next.
        let teleported = ServerMessage::Teleported { x: 5, y: 6 };
        while next_answer() != teleported {}

        // Leaving: the server saves and stops, so dropping it returns.
        drop(client);
        drop(server);
        let saved = folder.0.join("characters/player.ron");
        let character = std::fs::read_to_string(saved).unwrap();
        assert!(character.contains("world_position: (5, 6)"), "{character}");
    }
}
