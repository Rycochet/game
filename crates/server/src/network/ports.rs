//! Binding a server's ports and starting the thread that serves them: once for
//! a standalone server, and whenever an embedded one goes online.

use bevy::prelude::*;
use shared::protocol::ServerLink;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};

use super::routes::{Stop, serve};
use super::web_files::WebClient;
use super::{Heartbeat, PING_EVERY, SILENT_AFTER, SecurePort, Sockets};
use crate::clock::ServerClock;
use crate::tls::{Certificates, Security};

/// Ports being served.
pub(crate) struct OpenPorts {
    /// Connections accepted, waiting to join the game.
    pub(crate) accepted: Receiver<ServerLink>,
    /// The certificate in use (PEM) and its key's fingerprint, if secure.
    pub(crate) certificate: Option<(String, String)>,
    pub(super) thread: JoinHandle<()>,
}

/// Binds the ports and serves them on a thread of their own until `stop`: the
/// plain one (`plain`, if any) and, with TLS, the secure one in `security`.
/// An `Err` says why it can't, before anything is served.
pub(crate) fn open_ports(
    clock: ServerClock,
    heartbeat: Heartbeat,
    plain: Option<SocketAddr>,
    security: &Security,
    web: Option<WebClient>,
    stop: Stop,
) -> Result<OpenPorts, String> {
    let bind = |address: SocketAddr| {
        TcpListener::bind(address).map_err(|error| format!("Can't listen on {address}: {error}"))
    };
    let plain = plain.map(bind).transpose()?;
    let secure = match security {
        Security::PlainOnly => {
            info!("SSL is disabled: connections here aren't secured");
            None
        }
        Security::Tls {
            address,
            source,
            plain_too,
        } => {
            if *plain_too && let Some(plain) = &plain {
                let at = plain.local_addr().map_err(|error| error.to_string())?;
                info!("Behind a proxy: connections on {at} aren't secured");
            }
            let certificates = Certificates::load(source)?;
            let listener = bind(*address)?;
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
    let certificate = secure.as_ref().map(|secure| {
        (
            secure.certificates.public_pem(),
            secure.certificates.fingerprint(),
        )
    });
    if let Some(plain) = &plain
        && let Ok(at) = plain.local_addr()
    {
        info!("Listening on {at}");
    }
    let (new_connections, accepted) = mpsc::channel();
    let sockets = Sockets {
        new_connections,
        clock,
        connections: Arc::default(),
        heartbeat,
        ping_every: PING_EVERY,
        silent_after: SILENT_AFTER,
    };
    let thread = thread::Builder::new()
        .name("network".into())
        .spawn(move || serve(plain, secure, web, sockets, stop))
        .map_err(|error| format!("Can't start the network thread: {error}"))?;
    Ok(OpenPorts {
        accepted,
        certificate,
        thread,
    })
}
