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

use std::io;
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::serve::{Listener, ListenerExt};
use bevy::prelude::*;
use shared::protocol::{ClientMessage, SOCKET_PATH, ServerLink, ServerMessage, wired};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;
use tower_http::services::ServeDir;

use crate::ServerWorld;
use crate::clock::WorldClock;
use crate::connections::{self, Connections};
use crate::latency::{Latency, Pings};
use crate::lets_encrypt::is_domain_check;
use crate::tls::{Certificates, Security};

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

/// Lets the connections that arrived join (they log in from there).
fn accept_connections(new: Res<NewConnections>, mut connections: ResMut<Connections>) {
    let accepted = new.0.lock().expect("never poisoned");
    for link in accepted.try_iter() {
        info!("A client connected");
        connections.add(link);
    }
}

/// Asked to stop: everyone is saved and disconnected, then the server stops.
fn stop_when_asked(
    stop: Res<StopAsked>,
    mut connections: ResMut<Connections>,
    mut world: ResMut<ServerWorld>,
    mut exit: MessageWriter<AppExit>,
) {
    if stop.0.load(Ordering::Relaxed) {
        info!("Stopping: saving everyone");
        connections.disconnect_all(&mut world);
        world.storage.finish_writing();
        exit.write(AppExit::Success);
    }
}

/// Waits for Ctrl+C, or (on Unix) SIGTERM.
async fn stop_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("can watch for SIGTERM");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

/// The routes that serve the game: its WebSockets, the certificate if there
/// is one, and the web client's files.
fn game_routes(
    web: Option<PathBuf>,
    sockets: Sockets,
    certificates: Option<Arc<Certificates>>,
) -> Router {
    let mut router = Router::new()
        .route(SOCKET_PATH, get(upgrade))
        .with_state(sockets);
    if let Some(certificates) = certificates {
        router = router.route(
            CERTIFICATE_PATH,
            get(move || async move {
                (
                    [(header::CONTENT_TYPE, "application/x-pem-file")],
                    certificates.public_pem(),
                )
            }),
        );
    }
    if let Some(web) = web {
        router = router.fallback_service(ServeDir::new(web));
    }
    router
}

/// What the plain port answers when the secure one is in use, to everything.
async fn use_the_secure_port(secure_port: u16) -> impl IntoResponse {
    (
        StatusCode::UPGRADE_REQUIRED,
        [(header::UPGRADE, "TLS/1.3")],
        format!(
            "This server only takes secure connections: use https:// or wss:// on port {secure_port}.\n"
        ),
    )
}

/// Runs the web server (the game's WebSockets, and the web client's files)
/// until asked to stop.
fn serve(
    listener: TcpListener,
    secure: Option<SecurePort>,
    web: Option<PathBuf>,
    sockets: Sockets,
    stop_asked: Arc<AtomicBool>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a tokio runtime");
    runtime.block_on(async move {
        listener.set_nonblocking(true).expect("can be non-blocking");
        // Small messages go at once, rather than waiting to join a bigger one.
        let listener = tokio::net::TcpListener::from_std(listener)
            .expect("a tokio listener")
            .tap_io(|stream| {
                let _ = stream.set_nodelay(true);
            });
        tokio::spawn(async move {
            stop_signal().await;
            stop_asked.store(true, Ordering::Relaxed);
        });
        if let Some(web) = &web {
            info!("Serving the web client from {}", web.display());
        }
        let Some(secure) = secure else {
            if let Err(error) = axum::serve(listener, game_routes(web, sockets, None)).await {
                error!("The web server stopped: {error}");
            }
            return;
        };
        let port = secure
            .listener
            .local_addr()
            .expect("a bound port has an address")
            .port();
        let plain_routes = if secure.plain_too {
            game_routes(web.clone(), sockets.clone(), None)
        } else {
            Router::new().fallback(move || use_the_secure_port(port))
        };
        let plain = axum::serve(listener, plain_routes);
        let routes = game_routes(web, sockets, Some(secure.certificates.clone()));
        let secured = axum::serve(TlsListener::start(secure), routes);
        let (plain, secured) = tokio::join!(plain.into_future(), secured.into_future());
        if let Err(error) = plain.and(secured) {
            error!("The web server stopped: {error}");
        }
    });
}

/// Hands over connections once their TLS handshake is done, so a slow client
/// doesn't hold up anyone else.
struct TlsListener {
    ready: tokio::sync::mpsc::Receiver<(TlsStream<tokio::net::TcpStream>, SocketAddr)>,
    address: SocketAddr,
}

impl TlsListener {
    /// Starts accepting on the secure port, and checking for replaced
    /// certificate files.
    fn start(secure: SecurePort) -> Self {
        let SecurePort {
            listener,
            certificates,
            ask_lets_encrypt,
            ..
        } = secure;
        listener.set_nonblocking(true).expect("can be non-blocking");
        let address = listener.local_addr().expect("a bound port has an address");
        let listener = tokio::net::TcpListener::from_std(listener).expect("a tokio listener");
        let acceptor = TlsAcceptor::from(Arc::new(certificates.server_config()));
        if let Some(issuing) = certificates.issuing().filter(|_| ask_lets_encrypt) {
            tokio::spawn(issuing.run());
        }
        let (finished, ready) = tokio::sync::mpsc::channel(16);
        tokio::spawn(async move {
            loop {
                let (stream, peer) = match listener.accept().await {
                    Ok(accepted) => accepted,
                    // Out of file handles, say: try again shortly.
                    Err(error) => {
                        warn!("Can't accept a connection: {error}");
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                };
                let _ = stream.set_nodelay(true);
                let acceptor = acceptor.clone();
                let finished = finished.clone();
                tokio::spawn(async move {
                    match tokio::time::timeout(HANDSHAKE_LIMIT, acceptor.accept(stream)).await {
                        // Dropped: the handshake was all a domain check wanted.
                        Ok(Ok(stream)) if is_domain_check(stream.get_ref().1.alpn_protocol()) => {}
                        Ok(Ok(stream)) => {
                            let _ = finished.send((stream, peer)).await;
                        }
                        Ok(Err(error)) => debug!("A secure connection failed: {error}"),
                        Err(_) => debug!("A secure connection took too long to set up"),
                    }
                });
            }
        });
        if certificates.watches_files() {
            tokio::spawn(async move {
                let mut check = tokio::time::interval(CHECK_FILES_EVERY);
                loop {
                    check.tick().await;
                    certificates.reload_if_changed();
                }
            });
        }
        Self { ready, address }
    }
}

impl Listener for TlsListener {
    type Io = TlsStream<tokio::net::TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        match self.ready.recv().await {
            Some(accepted) => accepted,
            // The accepting task never ends while this runs.
            None => std::future::pending().await,
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        Ok(self.address)
    }
}

/// A request to open a WebSocket.
async fn upgrade(upgrade: WebSocketUpgrade, State(sockets): State<Sockets>) -> Response {
    upgrade
        .max_message_size(MAX_MESSAGE)
        .on_upgrade(move |socket| carry(socket, sockets))
}

/// Sends a message; `false` if it failed or the client isn't taking it.
async fn send(socket: &mut WebSocket, message: Message, limit: Duration) -> bool {
    matches!(
        tokio::time::timeout(limit, socket.send(message)).await,
        Ok(Ok(()))
    )
}

/// Logs a connection going into trouble or coming out of it.
fn log_trouble(number: u64, latency: &Latency, troubled: bool) {
    let round_trip = latency.smoothed_ms().unwrap_or_default();
    let jitter = latency.jitter_ms();
    if troubled {
        warn!(
            "Connection {number} is in trouble: round trips {round_trip:.0} ms, varying by {jitter:.0} ms"
        );
    } else {
        info!(
            "Connection {number} is over its trouble: round trips {round_trip:.0} ms, varying by {jitter:.0} ms"
        );
    }
}

/// Carries one client's messages between its WebSocket and its link, until
/// either side goes or the client falls silent. Pings are answered here
/// instead, and this times its own.
async fn carry(mut socket: WebSocket, sockets: Sockets) {
    let (link, wire) = wired();
    if sockets.new_connections.send(link).is_err() {
        return;
    }
    let number = sockets.connections.fetch_add(1, Ordering::Relaxed) + 1;
    let limit = sockets.silent_after;
    let mut pings = Pings::new();
    let mut latency = Latency::default();
    let mut last_heard = Instant::now();
    let (to_link, from_link) = wire.into_parts();
    // `from_link` only has a waiting `recv`: a thread waits on it and passes
    // what the game sends to this task. It ends when the game drops the link.
    let (outgoing, mut to_socket) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    thread::spawn(move || {
        while let Ok(bytes) = from_link.recv() {
            if outgoing.send(bytes).is_err() {
                break;
            }
        }
    });
    let mut ping = tokio::time::interval(sockets.ping_every);
    // After a hold-up, carry on rather than sending the missed ones in a burst.
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                last_heard = Instant::now();
                match incoming {
                    Some(Ok(Message::Binary(bytes))) => {
                        if let Some(sent) = ClientMessage::read_ping(&bytes) {
                            let pong = ServerMessage::pong_bytes(sent, sockets.clock.minutes());
                            if !send(&mut socket, Message::Binary(pong.into()), limit).await {
                                break;
                            }
                        } else if to_link.send(bytes.to_vec()).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(payload))) => {
                        if let Some(round_trip) = pings.answered(&payload, last_heard) {
                            debug!("Connection {number} ping: {:.1} ms", round_trip.as_secs_f64() * 1000.0);
                            if let Some(troubled) = latency.measured(round_trip) {
                                log_trouble(number, &latency, troubled);
                            }
                        }
                    }
                    // Text isn't part of the game; pings are answered by the
                    // library.
                    Some(Ok(Message::Text(_) | Message::Ping(_))) => {}
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                }
            }
            outgoing = to_socket.recv() => match outgoing {
                Some(bytes) => {
                    if !send(&mut socket, Message::Binary(bytes.into()), limit).await {
                        warn!("Connection {number} isn't taking what is sent: dropping it");
                        break;
                    }
                }
                // The game has dropped this client.
                None => {
                    send(&mut socket, Message::Close(None), limit).await;
                    break;
                }
            },
            _ = ping.tick() => {
                let now = Instant::now();
                if now.duration_since(last_heard) >= limit {
                    warn!("Connection {number} has gone silent: dropping it");
                    break;
                }
                if pings
                    .waiting_since()
                    .is_some_and(|sent| now.duration_since(sent) >= NO_ANSWER_AFTER)
                    && let Some(troubled) = latency.no_answer()
                {
                    log_trouble(number, &latency, troubled);
                }
                let payload = pings.next(now).to_vec();
                if !send(&mut socket, Message::Ping(payload.into()), limit).await {
                    break;
                }
            }
        }
    }
    if let Some(round_trip) = latency.smoothed_ms() {
        info!(
            "Connection {number} ended: round trips {round_trip:.0} ms, varying by {:.0} ms",
            latency.jitter_ms()
        );
    }
    // Dropping `to_link` tells the game the client has gone.
}

#[cfg(test)]
mod tests;
