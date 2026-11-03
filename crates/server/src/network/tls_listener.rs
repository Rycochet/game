//! `TlsListener`: the secure port, accepting TLS connections.

use axum::serve::Listener;
use bevy::prelude::*;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

use super::{CHECK_FILES_EVERY, HANDSHAKE_LIMIT, SecurePort};
use crate::lets_encrypt::is_domain_check;

/// Hands over connections once their TLS handshake is done, so a slow client
/// doesn't hold up anyone else.
pub(super) struct TlsListener {
    ready: tokio::sync::mpsc::Receiver<(TlsStream<tokio::net::TcpStream>, SocketAddr)>,
    address: SocketAddr,
}

impl TlsListener {
    /// Starts accepting on the secure port, and checking for replaced
    /// certificate files.
    pub(super) fn start(secure: SecurePort) -> Self {
        let SecurePort {
            listener,
            certificates,
            ask_lets_encrypt,
            ..
        } = secure;
        listener.set_nonblocking(true).expect("can be non-blocking");
        let address = listener.local_addr().expect("a bound port has an address");
        let listener = tokio::net::TcpListener::from_std(listener).expect("a tokio listener");
        let acceptor = TlsAcceptor::from(Arc::new(certificates.server_config()));
        if let Some(issuing) = certificates.issuing().filter(|_| ask_lets_encrypt) {
            tokio::spawn(issuing.run());
        }
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
                        // Dropped: the handshake was all a domain check wanted.
                        Ok(Ok(stream)) if is_domain_check(stream.get_ref().1.alpn_protocol()) => {}
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
