//! Clients over the network: one port serving both the web client's files
//! and the game itself as WebSockets on `/ws`, so a single HTTPS port (or a
//! proxy such as Cloudflare in front of it) is all a server needs.
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
use axum::response::Response;
use axum::routing::get;
use axum::serve::ListenerExt;
use bevy::prelude::*;
use shared::protocol::{ClientMessage, SOCKET_PATH, ServerLink, ServerMessage, wired};
use tower_http::services::ServeDir;

use crate::ServerWorld;
use crate::connections::{self, Connections};
use crate::latency::{Latency, Pings};

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

/// Listens on `address` for game clients (and serves the web client from
/// `web`, if given).
pub struct NetworkPlugin {
    pub address: SocketAddr,
    pub web: Option<PathBuf>,
}

/// Connections the web server has accepted, waiting to join the game.
#[derive(Resource)]
struct NewConnections(Mutex<Receiver<ServerLink>>);

/// Set when the server has been asked to stop (Ctrl+C, or `docker stop`'s
/// SIGTERM).
#[derive(Resource, Clone, Default)]
struct StopAsked(Arc<AtomicBool>);

/// The world's clock as of the last tick, for answering pings on the network
/// thread: the bits of a float, as there is no atomic float. Being up to a tick
/// old doesn't matter, as clients ease towards it.
#[derive(Resource, Clone, Default)]
struct SharedClock(Arc<AtomicU64>);

impl SharedClock {
    fn set(&self, hours: f64) {
        self.0.store(hours.to_bits(), Ordering::Relaxed);
    }

    fn hours(&self) -> f64 {
        f64::from_bits(self.0.load(Ordering::Relaxed))
    }
}

/// What every WebSocket needs: where to hand its link, the clock, the number
/// for the next connection (to tell them apart in the log), and how often to
/// ping and when to give up (shorter in tests).
#[derive(Clone)]
struct Sockets {
    new_connections: Sender<ServerLink>,
    clock: SharedClock,
    connections: Arc<AtomicU64>,
    ping_every: Duration,
    silent_after: Duration,
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        // Bound here, so a port in use stops the server straight away.
        let listener = TcpListener::bind(self.address)
            .unwrap_or_else(|error| panic!("Can't listen on {}: {error}", self.address));
        let (new_connections, accepted) = mpsc::channel();
        let web = self.web.clone();
        let stop = StopAsked::default();
        let stop_asked = stop.0.clone();
        let clock = SharedClock::default();
        let sockets = Sockets {
            new_connections,
            clock: clock.clone(),
            connections: Arc::default(),
            ping_every: PING_EVERY,
            silent_after: SILENT_AFTER,
        };
        thread::Builder::new()
            .name("network".into())
            .spawn(move || serve(listener, web, sockets, stop_asked))
            .expect("the network thread can start");
        info!("Listening on {}", self.address);
        app.insert_resource(NewConnections(Mutex::new(accepted)))
            .insert_resource(stop)
            .insert_resource(clock)
            .add_systems(
                Update,
                (
                    accept_connections.before(connections::handle_messages),
                    stop_when_asked.after(connections::handle_messages),
                    share_clock.after(connections::run_clock),
                ),
            );
    }
}

/// Every tick: lets the network thread see the clock.
fn share_clock(world: Res<ServerWorld>, clock: Res<SharedClock>) {
    clock.set(world.clock);
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

/// Runs the web server (the game's WebSockets, and the web client's files)
/// until asked to stop.
fn serve(
    listener: TcpListener,
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
        let mut router = Router::new()
            .route(SOCKET_PATH, get(upgrade))
            .with_state(sockets);
        if let Some(web) = web {
            info!("Serving the web client from {}", web.display());
            router = router.fallback_service(ServeDir::new(web));
        }
        tokio::spawn(async move {
            stop_signal().await;
            stop_asked.store(true, Ordering::Relaxed);
        });
        if let Err(error) = axum::serve(listener, router).await {
            error!("The web server stopped: {error}");
        }
    });
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
                            let pong = ServerMessage::pong_bytes(sent, sockets.clock.hours());
                            if !send(&mut socket, Message::Binary(pong.into()), limit).await {
                                break;
                            }
                        } else if to_link.send(bytes.to_vec()).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(payload))) => {
                        if let Some(round_trip) = pings.answered(&payload, last_heard)
                            && let Some(troubled) = latency.measured(round_trip)
                        {
                            log_trouble(number, &latency, troubled);
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

    use shared::protocol::PROTOCOL_VERSION;
    use tungstenite::Message as WsMessage;

    /// A web server with no game running: nothing ever reads its links.
    fn serve_without_a_game(
        ping_every: Duration,
        silent_after: Duration,
    ) -> (SocketAddr, Receiver<ServerLink>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (new_connections, accepted) = mpsc::channel();
        let clock = SharedClock::default();
        clock.set(6.75);
        let sockets = Sockets {
            new_connections,
            clock,
            connections: Arc::default(),
            ping_every,
            silent_after,
        };
        thread::spawn(move || serve(listener, None, sockets, Arc::default()));
        (address, accepted)
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
        assert_eq!(ServerMessage::read_pong(&answer), Some((1.5, 6.75)));

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
