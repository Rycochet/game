//! Where the client connects, and the WebSockets that carry a `ClientLink`'s
//! bytes there (one game message per WebSocket binary message).
//!
//! - Native: the embedded single-player server, or a server named by
//!   `HOW_SERVER` (`wss://host:port/ws`, checked by `trust`, or plain
//!   `ws://`), through a thread running `tungstenite`.
//! - Web: the server the page came from (`wss://` on an `https` page),
//!   through the browser's `WebSocket`.
//!
//! When a WebSocket closes, its link reports `Closed`, as an in-process link
//! does when the server goes. Both also send pings and time the answers
//! (`ping`), which never reach the link.

// Split into files (`cargo xtask lint` keeps files small).
#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
pub(super) use browser::BrowserSocket;
#[cfg(not(target_arch = "wasm32"))]
pub(super) use native::{Refusal, open};

/// Where to connect.
pub(super) enum Target {
    /// Start a single-player server in this process.
    #[cfg(not(target_arch = "wasm32"))]
    Embedded,
    /// A server's WebSocket.
    Url(String),
}

impl Target {
    /// Native: `HOW_SERVER` if set, otherwise single player.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn choose() -> Self {
        match std::env::var("HOW_SERVER") {
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
