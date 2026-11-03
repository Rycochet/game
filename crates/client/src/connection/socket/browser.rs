//! Web: a browser WebSocket and the link it carries.

use shared::protocol::ClientLink;

use super::super::ping::LatestPong;
use super::super::ping::PING_EVERY;
use super::super::ping::Pinger;

/// Web: a browser WebSocket and the link whose bytes it carries. Incoming
/// messages go straight to the link; outgoing ones wait in the link until
/// `pump` sends them (once the socket is open). Dropping it closes the
/// socket, and the link reports `Closed`. There are no threads here: pings go
/// from `pump`, and their answers are timed as soon as the browser hands them
/// over, between frames.
#[cfg(target_arch = "wasm32")]
pub(in crate::connection) struct BrowserSocket {
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
