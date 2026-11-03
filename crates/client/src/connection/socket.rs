//! Where the client connects, and the WebSockets that carry a `ClientLink`'s
//! bytes there (one game message per WebSocket binary message).
//!
//! - Native: a server chosen in Connect, or named by `HOW_SERVER`
//!   (`wss://host:port/ws`, checked by `trust`, or plain `ws://`), through a
//!   thread running `tungstenite`; otherwise the embedded single-player server.
//! - Web: a server chosen in Connect, or the server the page came from
//!   (`wss://` on an `https` page), through the browser's `WebSocket`.
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

#[cfg(not(target_arch = "wasm32"))]
use super::trust::KeyFrom;
use crate::common::ChosenServer;

/// Where to connect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Target {
    /// Start a single-player server in this process.
    #[cfg(not(target_arch = "wasm32"))]
    Embedded,
    /// A server's WebSocket.
    Url {
        url: String,
        /// The fingerprint its certificate's key must have, if any.
        #[cfg(not(target_arch = "wasm32"))]
        key: Option<(String, KeyFrom)>,
    },
}

impl Target {
    /// Native: the server chosen in Connect; otherwise `HOW_SERVER` (with
    /// `HOW_SERVER_KEY`), until logging out; otherwise single player.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn choose(chosen: &ChosenServer, environment: bool) -> Self {
        if let Some(server) = &chosen.0
            && let Some(address) = server.parsed()
        {
            let key = server.key.clone().map(|key| (key, KeyFrom::Pasted));
            return Self::Url {
                url: address.url(),
                key,
            };
        }
        match std::env::var("HOW_SERVER") {
            Ok(url) if environment => Self::Url {
                url,
                key: std::env::var("HOW_SERVER_KEY")
                    .ok()
                    .map(|key| (key.trim().to_lowercase(), KeyFrom::Environment)),
            },
            _ => Self::Embedded,
        }
    }

    /// Web: the server chosen in Connect, otherwise the one the page came from.
    #[cfg(target_arch = "wasm32")]
    pub fn choose(chosen: &ChosenServer, _environment: bool) -> Self {
        if let Some(address) = chosen.0.as_ref().and_then(|server| server.parsed()) {
            return Self::Url { url: address.url() };
        }
        let location = web_sys::window().expect("a window").location();
        let secure = location.protocol().is_ok_and(|p| p == "https:");
        let host = location.host().unwrap_or_default();
        let scheme = if secure { "wss" } else { "ws" };
        Self::Url {
            url: format!("{scheme}://{host}{}", shared::protocol::SOCKET_PATH),
        }
    }
}
