//! Messages to and from the server, and the link to it.

use bevy::prelude::*;
use shared::protocol::{ClientLink, ClientMessage, ServerMessage};

use super::ping::LatestPong;

/// A message for the server, sent at the end of the frame.
#[derive(Message, Clone, Debug)]
pub struct ToServer(pub ClientMessage);

/// A message from the server about maps (`Welcome`, `EnterLevel`, `Buildings`,
/// `LevelCells`, `RememberedLevel`, `OnWorldMap`, `Teleported`), for `site`. The rest are
/// handled here.
#[derive(Message, Clone, Debug)]
pub struct FromServer(pub ServerMessage);

/// The link to the server. Removed if the connection is lost (and tried
/// again), and on quitting.
#[derive(Resource)]
pub struct ServerConnection {
    pub(super) link: ClientLink,
    /// The latest answer to a ping.
    pub(super) pongs: LatestPong,
    /// Whether the server is in this process, with no network to time.
    pub(super) local: bool,
    /// The single-player server, if it is ours. Dropping it waits for it to
    /// save and stop, so it must come after `link` (fields are dropped in
    /// order): the server stops once the link is gone.
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) _embedded: Option<server::EmbeddedServer>,
    /// Why the socket gave up for good, if it did (a certificate that fails
    /// its checks, say).
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) refusal: super::socket::Refusal,
}
