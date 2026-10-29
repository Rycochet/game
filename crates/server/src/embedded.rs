//! The single-player server, embedded in a native client.

use std::thread;

use shared::protocol::{ClientLink, local_pair};
use shared::world::WorldSeed;

use crate::{Connections, ServerOptions, headless_app};

/// Starts a server on its own thread, with its own Bevy world, and returns
/// the client's end of its one connection. The client's log is used, and the
/// thread ends with the process.
pub fn start_embedded(world_seed: WorldSeed) -> ClientLink {
    let (client, server) = local_pair();
    thread::Builder::new()
        .name("server".into())
        .spawn(move || {
            // Made on the thread itself: a Bevy `App` can't move between
            // threads.
            // Single player: the player's own server, so dev tools are fine.
            let mut app = headless_app(world_seed, ServerOptions { dev_tools: true });
            app.world_mut().resource_mut::<Connections>().add(server);
            app.run();
        })
        .expect("the server thread can start");
    client
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use shared::protocol::{ClientMessage, PROTOCOL_VERSION, ServerMessage};

    use super::*;
    use crate::ServerWorld;

    #[test]
    fn an_embedded_server_welcomes_its_client() {
        let seed = WorldSeed::new(99);
        let client = start_embedded(seed);
        assert!(client.send(&ClientMessage::Hello {
            protocol: PROTOCOL_VERSION
        }));
        let deadline = Instant::now() + Duration::from_secs(10);
        let answer = loop {
            if let Some(message) = client.receive().expect("still connected") {
                break message;
            }
            assert!(Instant::now() < deadline, "no answer from the server");
            thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(
            answer,
            ServerMessage::Welcome(ServerWorld::new(seed).info())
        );
    }
}
