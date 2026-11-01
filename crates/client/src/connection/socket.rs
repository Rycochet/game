//! Where the client connects, and the WebSockets that carry a `ClientLink`'s
//! bytes there (one game message per WebSocket binary message).
//!
//! - Native: the embedded single-player server, or a server named by
//!   `HALL_OF_WORLDS_SERVER` (`ws://host:port/ws`; plain `ws://` only, no
//!   TLS yet), through a thread running `tungstenite`.
//! - Web: the server the page came from (`wss://` on an `https` page),
//!   through the browser's `WebSocket`.
//!
//! When a WebSocket closes, its link reports `Closed`, as an in-process link
//! does when the server goes. Both also send pings and time the answers
//! (`ping`), which never reach the link.

use shared::protocol::ClientLink;

use super::ping::LatestPong;
#[cfg(target_arch = "wasm32")]
use super::ping::PING_EVERY;
use super::ping::Pinger;

/// Where to connect.
pub(super) enum Target {
    /// Start a single-player server in this process.
    #[cfg(not(target_arch = "wasm32"))]
    Embedded,
    /// A server's WebSocket.
    Url(String),
}

impl Target {
    /// Native: `HALL_OF_WORLDS_SERVER` if set, otherwise single player.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn choose() -> Self {
        match std::env::var("HALL_OF_WORLDS_SERVER") {
            Ok(url) => Self::Url(url),
            Err(_) => Self::Embedded,
        }
    }

    /// Web: the server the page was loaded from.
    #[cfg(target_arch = "wasm32")]
    pub fn choose() -> Self {
        let location = web_sys::window().expect("a window").location();
        let secure = location.protocol().is_ok_and(|p| p == "https:");
        let host = location.host().unwrap_or_default();
        let scheme = if secure { "wss" } else { "ws" };
        Self::Url(format!(
            "{scheme}://{host}{}",
            shared::protocol::SOCKET_PATH
        ))
    }
}

/// Native: a link carried over a WebSocket by a thread of its own, and where
/// the answers to its pings go. The thread ends (and the link reports `Closed`)
/// when either side goes. It doesn't depend on the game's frames, so the
/// connection and its timing carry on while the window is frozen.
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn open(url: &str) -> (ClientLink, LatestPong) {
    use std::thread;
    use std::time::{Duration, Instant};

    use tungstenite::stream::MaybeTlsStream;
    use tungstenite::{Message, connect};

    use super::ping::PING_EVERY;

    // How long a read waits before checking for something to send.
    const POLL: Duration = Duration::from_millis(2);

    let (link, wire) = shared::protocol::wired();
    let (to_link, from_link) = wire.into_parts();
    let url = url.to_owned();
    let latest = LatestPong::default();
    let pinger = Pinger::new(latest.clone());
    thread::Builder::new()
        .name("socket".into())
        .spawn(move || {
            let mut socket = match connect(&url) {
                Ok((socket, _)) => socket,
                Err(error) => {
                    bevy::log::info!("Can't connect to {url}: {error}");
                    return;
                }
            };
            if let MaybeTlsStream::Plain(stream) = socket.get_mut() {
                let _ = stream.set_read_timeout(Some(POLL));
                let _ = stream.set_nodelay(true);
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
    (link, latest)
}

/// Web: a browser WebSocket and the link whose bytes it carries. Incoming
/// messages go straight to the link; outgoing ones wait in the link until
/// `pump` sends them (once the socket is open). Dropping it closes the
/// socket, and the link reports `Closed`. There are no threads here: pings go
/// from `pump`, and their answers are timed as soon as the browser hands them
/// over, between frames.
#[cfg(target_arch = "wasm32")]
pub(super) struct BrowserSocket {
    socket: web_sys::WebSocket,
    from_link: std::sync::mpsc::Receiver<Vec<u8>>,
    closed: std::rc::Rc<std::cell::Cell<bool>>,
    pinger: Pinger,
    next_ping: std::cell::Cell<bevy::platform::time::Instant>,
    // Kept alive as long as the socket uses them; dropping them drops the
    // sender to the link.
    _on_message: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>,
    _on_close: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}

#[cfg(target_arch = "wasm32")]
impl BrowserSocket {
    pub fn open(url: &str) -> Option<(ClientLink, LatestPong, Self)> {
        use std::cell::Cell;
        use std::rc::Rc;

        use wasm_bindgen::JsCast;
        use wasm_bindgen::closure::Closure;

        let socket = web_sys::WebSocket::new(url).ok()?;
        // Arrives as an `ArrayBuffer`, read straight away: messages stay in
        // order (a `Blob` would be read later).
        socket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let (link, wire) = shared::protocol::wired();
        let (to_link, from_link) = wire.into_parts();
        let latest = LatestPong::default();
        let pinger = Pinger::new(latest.clone());
        let on_message = {
            let pinger = pinger.clone();
            Closure::<dyn FnMut(web_sys::MessageEvent)>::new(move |event: web_sys::MessageEvent| {
                if let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                    if !pinger.took_pong(&bytes) {
                        let _ = to_link.send(bytes);
                    }
                }
            })
        };
        let closed = Rc::new(Cell::new(false));
        let on_close = {
            let closed = closed.clone();
            Closure::<dyn FnMut(web_sys::Event)>::new(move |_| closed.set(true))
        };
        socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
        // An error is followed by a close.
        Some((
            link,
            latest,
            Self {
                socket,
                from_link,
                closed,
                pinger,
                next_ping: Cell::new(bevy::platform::time::Instant::now()),
                _on_message: on_message,
                _on_close: on_close,
            },
        ))
    }

    /// Sends what the link has sent, and a ping when one is due, once the
    /// socket is open. `false` once the socket has closed.
    pub fn pump(&self) -> bool {
        if self.closed.get() {
            return false;
        }
        if self.socket.ready_state() == web_sys::WebSocket::OPEN {
            for bytes in self.from_link.try_iter() {
                if self.socket.send_with_u8_array(&bytes).is_err() {
                    return false;
                }
            }
            let now = bevy::platform::time::Instant::now();
            if now >= self.next_ping.get() {
                self.next_ping.set(now + PING_EVERY);
                if self.socket.send_with_u8_array(&self.pinger.ping()).is_err() {
                    return false;
                }
            }
        }
        true
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for BrowserSocket {
    fn drop(&mut self) {
        self.socket.set_onmessage(None);
        self.socket.set_onclose(None);
        let _ = self.socket.close();
    }
}

/// The guest number this client plays as on servers without accounts
/// (`ClientMessage::Hello`): kept in the browser's local storage, so the
/// same browser gets the same character back.
#[cfg(target_arch = "wasm32")]
pub(super) fn guest() -> u64 {
    const KEY: &str = "hall-of-worlds-guest";
    let window = web_sys::window().expect("a window");
    let storage = window.local_storage().ok().flatten();
    let saved = storage
        .as_ref()
        .and_then(|storage| storage.get_item(KEY).ok().flatten())
        .and_then(|text| u64::from_str_radix(&text, 16).ok());
    if let Some(guest) = saved {
        return guest;
    }
    let mut bytes = [0u8; 8];
    if let Ok(crypto) = window.crypto() {
        let _ = crypto.get_random_values_with_u8_array(&mut bytes);
    }
    let guest = u64::from_le_bytes(bytes);
    if let Some(storage) = storage {
        let _ = storage.set_item(KEY, &format!("{guest:016x}"));
    }
    guest
}

/// Native: a new guest number each run (only for testing against a server;
/// single player ignores it).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn guest() -> u64 {
    use std::hash::{BuildHasher, RandomState};
    // `RandomState` starts from the system's randomness.
    RandomState::new().hash_one(std::time::SystemTime::now())
}
