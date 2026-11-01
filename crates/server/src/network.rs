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

use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

/// The largest message a client may send (they are all tiny).
const MAX_MESSAGE: usize = 64 * 1024;

/// How often an idle connection is pinged, so proxies (which drop idle
/// WebSockets, Cloudflare after 100 s) keep it open.
const PING_EVERY: Duration = Duration::from_secs(30);

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

/// What every WebSocket needs: where to hand its link, and the clock.
#[derive(Clone)]
struct Sockets {
    new_connections: Sender<ServerLink>,
    clock: SharedClock,
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

/// Carries one client's messages between its WebSocket and its link, until
/// either side goes. Pings are answered here instead.
async fn carry(mut socket: WebSocket, sockets: Sockets) {
    let (link, wire) = wired();
    if sockets.new_connections.send(link).is_err() {
        return;
    }
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
    let mut ping = tokio::time::interval(PING_EVERY);
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Binary(bytes))) => {
                    if let Some(sent) = ClientMessage::read_ping(&bytes) {
                        let pong = ServerMessage::pong_bytes(sent, sockets.clock.hours());
                        if socket.send(Message::Binary(pong.into())).await.is_err() {
                            break;
                        }
                    } else if to_link.send(bytes.to_vec()).is_err() {
                        break;
                    }
                }
                // Text isn't part of the game; pings and pongs are answered
                // by the library.
                Some(Ok(Message::Text(_) | Message::Ping(_) | Message::Pong(_))) => {}
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
            },
            outgoing = to_socket.recv() => match outgoing {
                Some(bytes) => {
                    if socket.send(Message::Binary(bytes.into())).await.is_err() {
                        break;
                    }
                }
                // The game has dropped this client.
                None => {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            },
            _ = ping.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        }
    }
    // Dropping `to_link` tells the game the client has gone.
}

#[cfg(test)]
mod tests {
    use super::*;

    use shared::protocol::PROTOCOL_VERSION;
    use tungstenite::Message as WsMessage;

    /// A web server with no game running: nothing ever reads its links.
    fn serve_without_a_game() -> (SocketAddr, Receiver<ServerLink>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (new_connections, accepted) = mpsc::channel();
        let clock = SharedClock::default();
        clock.set(6.75);
        let sockets = Sockets {
            new_connections,
            clock,
        };
        thread::spawn(move || serve(listener, None, sockets, Arc::default()));
        (address, accepted)
    }

    #[test]
    fn pings_are_answered_without_the_game_and_the_rest_reach_it() {
        let (address, accepted) = serve_without_a_game();
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
