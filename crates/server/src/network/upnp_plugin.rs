//! `HOW_UPNP` on a standalone server: asks the router to forward the secure
//! port (UPnP) for as long as the server runs, logging how it went. Never
//! behind a reverse proxy, whose own port is the one players reach.
//!
//! With Let's Encrypt the router's port 443 is forwarded, as Let's Encrypt
//! checks the domain there; otherwise the secure port's own number.

use bevy::prelude::*;
use std::net::SocketAddr;

use super::upnp::RouterWork;
use crate::tls::{CertificateSource, Security};

/// Forwards the router's `external` port to `local` while the server runs.
pub struct UpnpPlugin {
    pub external: u16,
    pub local: SocketAddr,
}

/// What `HOW_UPNP` comes to with the other settings.
pub enum UpnpChoice {
    /// Not asked for.
    Off,
    /// Asked for, but the server is behind a reverse proxy: ignored, with a
    /// warning.
    BehindProxy,
    Forward(UpnpPlugin),
}

/// Keeps the forwarding; dropped (removing it) when the server stops.
#[derive(Resource)]
struct Forwarding(#[allow(dead_code)] RouterWork);

impl UpnpPlugin {
    /// What `HOW_UPNP` asks for, given how the server is secured.
    pub fn from_environment(security: &Security) -> UpnpChoice {
        choose(crate::upnp_from_environment().unwrap_or(false), security)
    }
}

pub(super) fn choose(upnp: bool, security: &Security) -> UpnpChoice {
    if !upnp {
        return UpnpChoice::Off;
    }
    match security {
        Security::PlainOnly
        | Security::Tls {
            plain_too: true, ..
        } => UpnpChoice::BehindProxy,
        Security::Tls {
            address, source, ..
        } => {
            let external = match source {
                CertificateSource::LetsEncrypt { .. } => 443,
                _ => address.port(),
            };
            UpnpChoice::Forward(UpnpPlugin {
                external,
                local: *address,
            })
        }
    }
}

impl Plugin for UpnpPlugin {
    fn build(&self, app: &mut App) {
        info!(
            "Asking the router to forward port {} to {} (HOW_UPNP)",
            self.external, self.local
        );
        app.insert_resource(Forwarding(RouterWork::start(self.external, self.local)));
    }
}

#[cfg(test)]
mod tests;
