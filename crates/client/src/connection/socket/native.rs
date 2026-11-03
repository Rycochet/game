//! Native: a link carried over a WebSocket, and why a connection is refused.

use shared::protocol::ClientLink;

use super::super::ping::LatestPong;
use super::super::ping::Pinger;

/// Native: a link carried over a WebSocket by a thread of its own, and where
/// the answers to its pings go. The thread ends (and the link reports `Closed`)
/// when either side goes. It doesn't depend on the game's frames, so the
/// connection and its timing carry on while the window is frozen.
#[cfg(not(target_arch = "wasm32"))]
pub(in crate::connection) fn open(url: &str) -> (ClientLink, LatestPong, Refusal) {
    use std::net::TcpStream;
    use std::thread;
    use std::time::{Duration, Instant};

    use tungstenite::client::IntoClientRequest;
    use tungstenite::handshake::HandshakeError;
    use tungstenite::http::StatusCode;
    use tungstenite::stream::MaybeTlsStream;
    use tungstenite::{Message, client_tls_with_config};

    use super::super::ping::PING_EVERY;

    // How long a read waits before checking for something to send.
    const POLL: Duration = Duration::from_millis(2);

    let (link, wire) = shared::protocol::wired();
    let (to_link, from_link) = wire.into_parts();
    let url = url.to_owned();
    let latest = LatestPong::default();
    let pinger = Pinger::new(latest.clone());
    let refusal = Refusal::default();
    let refused = refusal.clone();
    thread::Builder::new()
        .name("socket".into())
        .spawn(move || {
            let request = match url.as_str().into_client_request() {
                Ok(request) => request,
                Err(error) => {
                    refused.set(format!("{url} is not a server's address: {error}"));
                    return;
                }
            };
            let host = request.uri().host().unwrap_or_default();
            let host = host.trim_matches(['[', ']']).to_owned();
            let secure = request.uri().scheme_str() == Some("wss");
            let port = request
                .uri()
                .port_u16()
                .unwrap_or(if secure { 443 } else { 80 });
            let connector = if secure {
                match super::super::trust::connector(&host, port, refused.clone()) {
                    Ok(connector) => Some(connector),
                    Err(error) => {
                        refused.set(error);
                        return;
                    }
                }
            } else {
                bevy::log::warn!("{url} isn't secure: use wss:// unless a VPN protects it");
                None
            };
            let stream = match TcpStream::connect((host.as_str(), port)) {
                Ok(stream) => stream,
                Err(error) => {
                    bevy::log::info!("Can't connect to {url}: {error}");
                    return;
                }
            };
            let _ = stream.set_nodelay(true);
            let mut socket = match client_tls_with_config(request, stream, None, connector) {
                Ok((socket, _)) => socket,
                Err(HandshakeError::Failure(tungstenite::Error::Http(response)))
                    if response.status() == StatusCode::UPGRADE_REQUIRED =>
                {
                    let body = response.body().as_deref().unwrap_or_default();
                    refused.set(String::from_utf8_lossy(body).trim().to_owned());
                    return;
                }
                Err(error) => {
                    bevy::log::info!("Can't connect to {url}: {error}");
                    return;
                }
            };
            // (Only once it is set up: setting up waits for the server.)
            let tcp = match socket.get_ref() {
                MaybeTlsStream::Plain(stream) => Some(stream),
                MaybeTlsStream::Rustls(stream) => Some(&stream.sock),
                _ => None,
            };
            if let Some(tcp) = tcp {
                let _ = tcp.set_read_timeout(Some(POLL));
            }
            let mut next_ping = Instant::now();
            loop {
                // Everything the game has sent, then whatever has arrived.
                loop {
                    match from_link.try_recv() {
                        Ok(bytes) => {
                            if socket.send(Message::Binary(bytes.into())).is_err() {
                                return;
                            }
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => break,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            let _ = socket.close(None);
                            return;
                        }
                    }
                }
                if Instant::now() >= next_ping {
                    next_ping = Instant::now() + PING_EVERY;
                    if socket.send(Message::Binary(pinger.ping().into())).is_err() {
                        return;
                    }
                }
                match socket.read() {
                    Ok(Message::Binary(bytes)) => {
                        if !pinger.took_pong(&bytes) && to_link.send(bytes.to_vec()).is_err() {
                            return;
                        }
                    }
                    Ok(Message::Close(_)) => return,
                    Ok(_) => {}
                    Err(tungstenite::Error::Io(error))
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => return,
                }
            }
        })
        .expect("the socket thread can start");
    (link, latest, refusal)
}

/// Why a connection must not be tried again, if it mustn't: left here by the
/// socket's thread, for when the link reports it closed.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Default)]
pub(in crate::connection) struct Refusal(
    pub(super) std::sync::Arc<std::sync::Mutex<Option<String>>>,
);

#[cfg(not(target_arch = "wasm32"))]
impl Refusal {
    pub fn set(&self, reason: String) {
        *self.0.lock().expect("never poisoned") = Some(reason);
    }

    pub fn take(&self) -> Option<String> {
        self.0.lock().expect("never poisoned").take()
    }
}
