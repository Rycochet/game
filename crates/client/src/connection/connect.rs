//! Connecting, trying again, and hanging up.

use super::ServerConnection;
#[cfg(not(target_arch = "wasm32"))]
use super::ping::LatestPong;
use super::prediction::Prediction;
use super::socket::Target;
use crate::common::{LoadingStep, ServerClock, ServerStatus};
use bevy::prelude::*;
use shared::protocol::{ClientMessage, GuestToken, PROTOCOL_VERSION};
use std::time::Duration;

/// How long to wait before trying to connect again.
pub(super) const RETRY_EVERY: Duration = Duration::from_secs(2);

/// How to connect, and when to try again.
#[derive(Resource)]
pub(super) struct Connecting {
    pub(super) target: Target,
    /// The token to log in as a guest with, once one is needed.
    pub(super) guest: Option<GuestToken>,
    /// Counting down to the next try; `None` to try now.
    pub(super) retry: Option<Timer>,
}

/// Connects (or tries again, every `RETRY_EVERY`), and says hello: the
/// link holds what is sent until the socket is open.
pub(super) fn connect(world: &mut World) {
    if matches!(world.resource::<ServerStatus>(), ServerStatus::Refused(_))
        || *world.resource::<LoadingStep>() == LoadingStep::Starting
    {
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
    let connection = match &connecting.target {
        #[cfg(not(target_arch = "wasm32"))]
        Target::Embedded => {
            let (link, embedded) = server::start_embedded(
                shared::world::WorldSeed::new(server::DEFAULT_WORLD_SEED),
                Some(server::default_data_folder()),
            );
            ServerConnection {
                link,
                pongs: LatestPong::default(),
                local: true,
                refusal: super::socket::Refusal::default(),
                _embedded: Some(embedded),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        Target::Url(url) => {
            info!("Connecting to {url}");
            let (link, pongs, refusal) = super::socket::open(url);
            ServerConnection {
                link,
                pongs,
                local: false,
                refusal,
                _embedded: None,
            }
        }
        #[cfg(target_arch = "wasm32")]
        Target::Url(url) => {
            info!("Connecting to {url}");
            let Some((link, pongs, socket)) = super::socket::BrowserSocket::open(url) else {
                warn!("Can't open a WebSocket to {url}");
                return;
            };
            world.insert_non_send(socket);
            ServerConnection {
                link,
                pongs,
                local: false,
            }
        }
    };
    world.resource_mut::<ServerClock>().local_server = connection.local;
    connection.link.send(&ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    });
    world.insert_resource(connection);
    // Steps start again from the server's word.
    world.insert_resource(Prediction::default());
}

/// Ends the connection and doesn't try again, saying why.
pub(super) fn give_up(commands: &mut Commands, status: &mut ServerStatus, reason: String) {
    warn!("Not connecting again: {reason}");
    *status = ServerStatus::Refused(reason);
    hang_up(commands);
}

/// Ends the connection.
pub(super) fn hang_up(commands: &mut Commands) {
    commands.remove_resource::<ServerConnection>();
    #[cfg(target_arch = "wasm32")]
    commands.queue(|world: &mut World| {
        world.remove_non_send::<super::socket::BrowserSocket>();
    });
}

/// Web: sends what the link has sent; drops the socket once it has closed,
/// so the link reports the connection lost.
#[cfg(target_arch = "wasm32")]
pub(super) fn pump_socket(world: &mut World) {
    let open = world
        .get_non_send::<super::socket::BrowserSocket>()
        .is_none_or(|socket| socket.pump());
    if !open {
        world.remove_non_send::<super::socket::BrowserSocket>();
    }
}
