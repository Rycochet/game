//! Taking new connections and serving the game's routes (and the web client's files).

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::serve::ListenerExt;
use bevy::prelude::*;
use shared::protocol::SOCKET_PATH;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tower_http::services::ServeDir;

use super::{CERTIFICATE_PATH, NewConnections, SecurePort, Sockets, TlsListener};
use super::{stop_signal, upgrade};
use crate::connections::Connections;
use crate::tls::Certificates;

/// Lets the connections that arrived join (they log in from there).
pub(super) fn accept_connections(new: Res<NewConnections>, mut connections: ResMut<Connections>) {
    let accepted = new.0.lock().expect("never poisoned");
    for link in accepted.try_iter() {
        info!("A client connected");
        connections.add(link);
    }
}

/// The routes that serve the game: its WebSockets, the certificate if there
/// is one, and the web client's files.
pub(super) fn game_routes(
    web: Option<PathBuf>,
    sockets: Sockets,
    certificates: Option<Arc<Certificates>>,
) -> Router {
    let mut router = Router::new()
        .route(SOCKET_PATH, get(upgrade))
        .with_state(sockets);
    if let Some(certificates) = certificates {
        router = router.route(
            CERTIFICATE_PATH,
            get(move || async move {
                (
                    [(header::CONTENT_TYPE, "application/x-pem-file")],
                    certificates.public_pem(),
                )
            }),
        );
    }
    if let Some(web) = web {
        router = router.fallback_service(ServeDir::new(web));
    }
    router
}

/// What the plain port answers when the secure one is in use, to everything.
async fn use_the_secure_port(secure_port: u16) -> impl IntoResponse {
    (
        StatusCode::UPGRADE_REQUIRED,
        [(header::UPGRADE, "TLS/1.3")],
        format!(
            "This server only takes secure connections: use https:// or wss:// on port {secure_port}.\n"
        ),
    )
}

/// Runs the web server (the game's WebSockets, and the web client's files)
/// until asked to stop.
pub(super) fn serve(
    listener: TcpListener,
    secure: Option<SecurePort>,
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
        tokio::spawn(async move {
            stop_signal().await;
            stop_asked.store(true, Ordering::Relaxed);
        });
        if let Some(web) = &web {
            info!("Serving the web client from {}", web.display());
        }
        let Some(secure) = secure else {
            if let Err(error) = axum::serve(listener, game_routes(web, sockets, None)).await {
                error!("The web server stopped: {error}");
            }
            return;
        };
        let port = secure
            .listener
            .local_addr()
            .expect("a bound port has an address")
            .port();
        let plain_routes = if secure.plain_too {
            game_routes(web.clone(), sockets.clone(), None)
        } else {
            Router::new().fallback(move || use_the_secure_port(port))
        };
        let plain = axum::serve(listener, plain_routes);
        let routes = game_routes(web, sockets, Some(secure.certificates.clone()));
        let secured = axum::serve(TlsListener::start(secure), routes);
        let (plain, secured) = tokio::join!(plain.into_future(), secured.into_future());
        if let Err(error) = plain.and(secured) {
            error!("The web server stopped: {error}");
        }
    });
}
