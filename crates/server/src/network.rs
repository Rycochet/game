//! Clients over the network: one port serving both the web client's files
//! and the game itself as WebSockets on `/ws`, so a single HTTPS port (or a
//! proxy such as Cloudflare in front of it) is all a server needs.
//!
//! That port is the secure one (HTTPS and `wss://`), and the plain port only
//! answers with an error saying so. Behind a proxy (`Security::BehindProxy`)
//! the plain port serves everything and the proxy looks after TLS.
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
            Security::BehindProxy => {
                info!("Behind a proxy: connections here aren't secured");
                None
            }
            Security::Tls { address, source } => {
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

/// Lets the connections that arrived join (as guests, named by their hello).
fn accept_connections(new: Res<NewConnections>, mut connections: ResMut<Connections>) {
    let accepted = new.0.lock().expect("never poisoned");
    for link in accepted.try_iter() {
        info!("A client connected");
        connections.add(link, None);
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
        info!("Serving the web client from {}", web.display());
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
        let refusing = Router::new().fallback(move || use_the_secure_port(port));
        let plain = axum::serve(listener, refusing);
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
        } = secure;
        listener.set_nonblocking(true).expect("can be non-blocking");
        let address = listener.local_addr().expect("a bound port has an address");
        let listener = tokio::net::TcpListener::from_std(listener).expect("a tokio listener");
        let acceptor = TlsAcceptor::from(Arc::new(certificates.server_config()));
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
mod tests {
    use super::*;

    use std::io::{Read, Write};
    use std::net::TcpStream;

    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::crypto::ring::default_provider;
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, SignatureScheme};
    use shared::protocol::PROTOCOL_VERSION;
    use tungstenite::Message as WsMessage;

    use crate::storage::tests::TestFolder;
    use crate::tls::CertificateSource;

    /// A web server with no game running: nothing ever reads its links.
    fn serve_without_a_game(
        ping_every: Duration,
        silent_after: Duration,
    ) -> (SocketAddr, Receiver<ServerLink>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (new_connections, accepted) = mpsc::channel();
        let clock = WorldClock::new(405.0);
        let sockets = Sockets {
            new_connections,
            clock,
            connections: Arc::default(),
            ping_every,
            silent_after,
        };
        thread::spawn(move || serve(listener, None, None, sockets, Arc::default()));
        (address, accepted)
    }

    /// A server with TLS on and no game running: its plain and secure
    /// addresses, its links, and the folder keeping its certificate.
    fn serve_securely() -> (SocketAddr, SocketAddr, Receiver<ServerLink>, TestFolder) {
        let folder = TestFolder::new();
        let source = CertificateSource::Generated {
            folder: folder.0.join("tls"),
            names: Vec::new(),
        };
        let secure = SecurePort {
            listener: TcpListener::bind("127.0.0.1:0").unwrap(),
            certificates: Arc::new(Certificates::load(&source).unwrap()),
        };
        let secure_address = secure.listener.local_addr().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let plain_address = listener.local_addr().unwrap();
        let (new_connections, accepted) = mpsc::channel();
        let sockets = Sockets {
            new_connections,
            clock: WorldClock::new(405.0),
            connections: Arc::default(),
            ping_every: PING_EVERY,
            silent_after: SILENT_AFTER,
        };
        thread::spawn(move || serve(listener, Some(secure), None, sockets, Arc::default()));
        (plain_address, secure_address, accepted, folder)
    }

    /// Accepts any certificate: these tests are about the server. The real
    /// checks are the client's (`connection::trust`).
    #[derive(Debug)]
    struct AcceptAnything;

    impl ServerCertVerifier for AcceptAnything {
        fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn verify_tls13_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Ok(HandshakeSignatureValid::assertion())
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            default_provider()
                .signature_verification_algorithms
                .supported_schemes()
        }
    }

    fn trusting_client() -> Arc<rustls::ClientConfig> {
        Arc::new(
            rustls::ClientConfig::builder_with_provider(Arc::new(default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(AcceptAnything))
                .with_no_client_auth(),
        )
    }

    /// Asks for a path over HTTP on any stream; the whole answer.
    fn ask(mut stream: impl Read + Write, path: &str) -> String {
        write!(
            stream,
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut answer = Vec::new();
        // A peer closing without TLS's goodbye is fine here.
        let _ = stream.read_to_end(&mut answer);
        String::from_utf8_lossy(&answer).into_owned()
    }

    #[test]
    fn the_plain_port_says_to_use_the_secure_one() {
        let (plain, secure, _accepted, _folder) = serve_securely();
        for path in ["/", SOCKET_PATH] {
            let answer = ask(TcpStream::connect(plain).unwrap(), path);
            assert!(answer.starts_with("HTTP/1.1 426"), "{answer}");
            assert!(
                answer.contains(&format!("port {}", secure.port())),
                "{answer}"
            );
        }
        let url = format!("ws://{plain}{SOCKET_PATH}");
        assert!(tungstenite::connect(url).is_err());
    }

    #[test]
    fn the_certificate_can_be_asked_for_and_is_the_one_in_use() {
        let (_, secure, _accepted, folder) = serve_securely();
        let name = ServerName::try_from("localhost").unwrap();
        let connection = rustls::ClientConnection::new(trusting_client(), name).unwrap();
        let mut stream = rustls::StreamOwned::new(connection, TcpStream::connect(secure).unwrap());

        let answer = ask(&mut stream, CERTIFICATE_PATH);

        assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
        assert!(answer.contains("BEGIN CERTIFICATE"), "{answer}");
        assert!(!answer.contains("PRIVATE KEY"), "{answer}");
        let on_disk = std::fs::read_to_string(folder.0.join("tls/certificate.pem")).unwrap();
        let from_disk = CertificateDer::from_pem_slice(on_disk.as_bytes()).unwrap();
        assert_eq!(stream.conn.peer_certificates().unwrap()[0], from_disk);
    }

    #[test]
    fn the_game_works_over_a_secure_websocket() {
        let (_, secure, accepted, _folder) = serve_securely();
        let url = format!("wss://localhost:{}{SOCKET_PATH}", secure.port());
        let stream = TcpStream::connect(secure).unwrap();
        let (mut socket, _) = tungstenite::client_tls_with_config(
            url,
            stream,
            None,
            Some(tungstenite::Connector::Rustls(trusting_client())),
        )
        .unwrap();
        assert!(accepted.recv_timeout(Duration::from_secs(5)).is_ok());

        socket
            .send(WsMessage::Binary(ClientMessage::ping_bytes(2.5).into()))
            .unwrap();
        let answer = loop {
            if let WsMessage::Binary(bytes) = socket.read().unwrap() {
                break bytes;
            }
        };
        assert_eq!(ServerMessage::read_pong(&answer).unwrap().0, 2.5);
    }

    #[test]
    fn a_silent_client_is_dropped() {
        let (address, accepted) =
            serve_without_a_game(Duration::from_millis(50), Duration::from_millis(300));
        // Never read from, so it never answers a ping.
        let (_socket, _) = tungstenite::connect(format!("ws://{address}{SOCKET_PATH}")).unwrap();
        let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        while link.receive() != Err(shared::protocol::LinkError::Closed) {
            assert!(Instant::now() < deadline, "the client wasn't dropped");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn a_client_answering_pings_stays() {
        let (address, accepted) =
            serve_without_a_game(Duration::from_millis(50), Duration::from_millis(300));
        let (mut socket, _) = tungstenite::connect(format!("ws://{address}{SOCKET_PATH}")).unwrap();
        if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_mut() {
            stream
                .set_read_timeout(Some(Duration::from_millis(10)))
                .unwrap();
        }
        let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();

        // Reading is all it takes: the library answers. Sending nothing else.
        let end = Instant::now() + Duration::from_secs(1);
        while Instant::now() < end {
            match socket.read() {
                Ok(_) => {}
                Err(tungstenite::Error::Io(error))
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(error) => panic!("the client was dropped: {error}"),
            }
        }
        assert!(link.receive().is_ok());
    }

    #[test]
    fn pings_are_answered_without_the_game_and_the_rest_reach_it() {
        let (address, accepted) = serve_without_a_game(PING_EVERY, SILENT_AFTER);
        let url = format!("ws://{address}{SOCKET_PATH}");
        let (mut socket, _) = tungstenite::connect(url).unwrap();

        socket
            .send(WsMessage::Binary(ClientMessage::ping_bytes(1.5).into()))
            .unwrap();
        // (The keep-alive ping comes first, on opening.)
        let answer = loop {
            if let WsMessage::Binary(bytes) = socket.read().unwrap() {
                break bytes;
            }
        };
        let (sent, clock) = ServerMessage::read_pong(&answer).unwrap();
        assert_eq!(sent, 1.5);
        assert!((405.0..465.0).contains(&clock), "{clock}");

        let hello = ClientMessage::Hello {
            protocol: PROTOCOL_VERSION,
            guest: 1,
        };
        let bytes = postcard_bytes(&hello);
        socket.send(WsMessage::Binary(bytes.into())).unwrap();
        let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            match link.receive().unwrap() {
                Some(message) => {
                    assert_eq!(message, hello, "the ping must not reach the game");
                    break;
                }
                None if std::time::Instant::now() > deadline => panic!("no hello arrived"),
                None => thread::sleep(Duration::from_millis(1)),
            }
        }
    }

    /// A message as the client's link would send it.
    fn postcard_bytes(message: &ClientMessage) -> Vec<u8> {
        let (link, wire) = wired::<ClientMessage, ServerMessage>();
        link.send(message);
        wire.take().unwrap().unwrap()
    }
}
