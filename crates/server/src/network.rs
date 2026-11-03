//! Clients over the network: one port serving both the web client's files
//! and the game itself as WebSockets on `/ws`, so a single HTTPS port (or a
//! proxy such as Cloudflare in front of it) is all a server needs.
//!
//! That port is the secure one (HTTPS and `wss://`), and the plain port only
//! answers with an error saying so. Behind a proxy, the plain port serves
//! everything too, and with `Security::PlainOnly` it is the only port.
//!
//! The web server runs on its own thread (with a small `tokio` runtime, as
//! the web libraries need). Each WebSocket becomes a `ServerLink` handed to
//! the Bevy world through a channel (`NewConnections`), exactly like the
//! embedded server's in-process link: one game message per WebSocket binary
//! message.
//!
//! `Ping` is the exception: it is answered here, without waiting for the
//! game's next tick, so a round trip measures the network and not the tick.
//! The server also times its own WebSocket pings (`latency`), to log connections
//! having trouble and to drop those that have gone silent.

use bevy::prelude::*;
use shared::protocol::ServerLink;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::ServerWorld;
use crate::clock::WorldClock;
use crate::connections::{self};
use crate::tls::{Certificates, Security};

// Split into files (`cargo xtask lint` keeps files small).
mod routes;
mod socket;
mod stop;
mod tls_listener;

use routes::{accept_connections, serve};
use socket::upgrade;
use stop::{stop_signal, stop_when_asked};
use tls_listener::TlsListener;

/// The largest message a client may send (they are all tiny).
const MAX_MESSAGE: usize = 64 * 1024;

/// How often a connection is pinged, to time it. This also keeps it open
/// through proxies (which drop idle WebSockets, Cloudflare after 100 s).
const PING_EVERY: Duration = Duration::from_secs(1);

/// A ping unanswered for this long means the connection is in trouble.
const NO_ANSWER_AFTER: Duration = Duration::from_secs(5);

/// A connection with nothing heard for this long, or that can't take what is
/// sent, is dropped: the player leaves and is saved, and a client still there
/// connects again. A frozen game doesn't count, as its network code still answers.
const SILENT_AFTER: Duration = Duration::from_secs(30);

/// Where a certificate can be asked for (only on the secure port).
const CERTIFICATE_PATH: &str = "/certificate";

/// How often the admin's certificate files are checked for a replacement.
const CHECK_FILES_EVERY: Duration = Duration::from_secs(60 * 60);

/// How long a client gets to finish setting up a secure connection.
const HANDSHAKE_LIMIT: Duration = Duration::from_secs(10);

/// Listens for game clients (and serves the web client from `web`, if given):
/// on `address` behind a proxy, otherwise securely on the address in `security`,
/// with `address` only answering that it isn't secure.
pub struct NetworkPlugin {
    pub address: SocketAddr,
    pub web: Option<PathBuf>,
    pub security: Security,
}

/// The secure port, and the certificate it uses.
struct SecurePort {
    listener: TcpListener,
    certificates: Arc<Certificates>,
    /// Whether to get certificates from Let's Encrypt, if they are from there
    /// (tests answer its checks without asking it for anything).
    ask_lets_encrypt: bool,
    /// Whether the plain port serves the game as well.
    plain_too: bool,
}

/// Connections the web server has accepted, waiting to join the game.
#[derive(Resource)]
struct NewConnections(Mutex<Receiver<ServerLink>>);

/// Set when the server has been asked to stop (Ctrl+C, or `docker stop`'s
/// SIGTERM).
#[derive(Resource, Clone, Default)]
struct StopAsked(Arc<AtomicBool>);

/// What every WebSocket needs: where to hand its link, the clock, the number
/// for the next connection (to tell them apart in the log), and how often to
/// ping and when to give up (shorter in tests).
#[derive(Clone)]
struct Sockets {
    new_connections: Sender<ServerLink>,
    clock: WorldClock,
    connections: Arc<AtomicU64>,
    ping_every: Duration,
    silent_after: Duration,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Bound here, so a port in use stops the server straight away.
        let listener = TcpListener::bind(self.address)
            .unwrap_or_else(|error| panic!("Can't listen on {}: {error}", self.address));
        let secure = match &self.security {
            Security::PlainOnly => {
                info!("SSL is disabled: connections here aren't secured");
                None
            }
            Security::Tls {
                address,
                source,
                plain_too,
            } => {
                if *plain_too {
                    info!(
                        "Behind a proxy: connections on {} aren't secured",
                        self.address
                    );
                }
                let certificates =
                    Certificates::load(source).unwrap_or_else(|error| panic!("{error}"));
                let listener = TcpListener::bind(address)
                    .unwrap_or_else(|error| panic!("Can't listen on {address}: {error}"));
                info!(
                    "Listening securely on {address}: key fingerprint {}",
                    certificates.fingerprint()
                );
                Some(SecurePort {
                    listener,
                    certificates: Arc::new(certificates),
                    ask_lets_encrypt: true,
                    plain_too: *plain_too,
                })
            }
        };
        let (new_connections, accepted) = mpsc::channel();
        let web = self.web.clone();
        let stop = StopAsked::default();
        let stop_asked = stop.0.clone();
        let clock = app.world().resource::<ServerWorld>().clock.clone();
        let sockets = Sockets {
            new_connections,
            clock,
            connections: Arc::default(),
            ping_every: PING_EVERY,
            silent_after: SILENT_AFTER,
        };
        thread::Builder::new()
            .name("network".into())
            .spawn(move || serve(listener, secure, web, sockets, stop_asked))
            .expect("the network thread can start");
        info!("Listening on {}", self.address);
        app.insert_resource(NewConnections(Mutex::new(accepted)))
            .insert_resource(stop)
            .add_systems(
                Update,
                (
                    accept_connections.before(connections::handle_messages),
                    stop_when_asked.after(connections::handle_messages),
                ),
            );
    }
}

#[cfg(test)]
mod tests;
