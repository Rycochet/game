//! Taking new connections and serving the game's routes (and the web client's files).

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::serve::ListenerExt;
use bevy::prelude::*;
use shared::protocol::SOCKET_PATH;
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::health::health_routes;
use super::web_files::{WebClient, WebFiles, web_files};
use super::{CERTIFICATE_PATH, NewConnections, SecurePort, Sockets, TlsListener, WEB_PATH};
use super::{stop_signal, upgrade};
use crate::connections::Connections;
use crate::oidc::Logins;
use crate::tls::Certificates;

/// Lets the connections that arrived join (they log in from there).
pub(super) fn accept_connections(new: Res<NewConnections>, mut connections: ResMut<Connections>) {
    let accepted = new.0.lock().expect("never poisoned");
    for link in accepted.try_iter() {
        info!("A client connected");
        connections.add(link);
    }
}

/// The routes that serve the game: its WebSockets, health and version, the
/// page players come back to from providers and the certificate if there are
/// any, and the web client's files under `WEB_PATH`
/// (where `/` sends browsers).
pub(super) fn game_routes(
    web: Option<Arc<WebFiles>>,
    sockets: Sockets,
    certificates: Option<Arc<Certificates>>,
) -> Router {
    let health = health_routes(sockets.heartbeat.clone());
    let logins = sockets.logins.as_ref().map(Logins::routes);
    let mut router = Router::new()
        .route(SOCKET_PATH, get(upgrade))
        .with_state(sockets)
        .merge(health);
    if let Some(logins) = logins {
        router = router.merge(logins);
    }
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
        // Temporary, so the address can change again without browsers
        // remembering this one.
        let to_web = || async { Redirect::temporary(WEB_PATH) };
        router = router
            .nest_service(WEB_PATH, web_files(web))
            .route("/", get(to_web))
            .route(WEB_PATH.trim_end_matches('/'), get(to_web));
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

/// What ends the web server.
pub(crate) enum Stop {
    /// Ctrl+C or SIGTERM, which sets this for the game to see and stop; the
    /// ports stay open until the program ends.
    Signal(Arc<AtomicBool>),
    /// The game closing the ports (an embedded server going offline): when the
    /// sender says so or is dropped.
    Closing(tokio::sync::watch::Receiver<()>),
}

impl Stop {
    /// Done when the ports should close.
    async fn wait(self) {
        match self {
            Self::Signal(stop_asked) => {
                stop_signal().await;
                stop_asked.store(true, Ordering::Relaxed);
                // The game stops the program.
                std::future::pending::<()>().await;
            }
            Self::Closing(mut closing) => {
                let _ = closing.changed().await;
            }
        }
    }
}

/// Runs the web server (the game's WebSockets, and the web client's files)
/// until `stop`; then its connections are cut as its runtime ends.
pub(super) fn serve(
    plain: Option<TcpListener>,
    secure: Option<SecurePort>,
    web: Option<WebClient>,
    sockets: Sockets,
    stop: Stop,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a tokio runtime");
    runtime.block_on(async move {
        if let Some(web) = &web {
            info!("Serving the web client from {}", web.files.display());
        }
        let web = web.map(WebFiles::start);
        tokio::select! {
            () = run(plain, secure, web, sockets) => {}
            () = stop.wait() => {}
        }
    });
}

/// Serves the ports, the plain one and the secure one, whichever there are.
async fn run(
    plain: Option<TcpListener>,
    secure: Option<SecurePort>,
    web: Option<Arc<WebFiles>>,
    sockets: Sockets,
) {
    let plain = plain.map(|listener| {
        listener.set_nonblocking(true).expect("can be non-blocking");
        // Small messages go at once, rather than waiting to join a bigger one.
        tokio::net::TcpListener::from_std(listener)
            .expect("a tokio listener")
            .tap_io(|stream| {
                let _ = stream.set_nodelay(true);
            })
    });
    let result = match (plain, secure) {
        (Some(plain), None) => axum::serve(plain, game_routes(web, sockets, None)).await,
        (plain, Some(secure)) => {
            let port = secure
                .listener
                .local_addr()
                .expect("a bound port has an address")
                .port();
            let plain_routes = if secure.plain_too {
                game_routes(web.clone(), sockets.clone(), None)
            } else {
                health_routes(sockets.heartbeat.clone()).fallback(move || use_the_secure_port(port))
            };
            let routes = game_routes(web, sockets, Some(secure.certificates.clone()));
            let secured = axum::serve(TlsListener::start(secure), routes).into_future();
            match plain {
                Some(plain) => {
                    let plain = axum::serve(plain, plain_routes).into_future();
                    let (plain, secured) = tokio::join!(plain, secured);
                    plain.and(secured)
                }
                None => secured.await,
            }
        }
        (None, None) => Ok(()),
    };
    if let Err(error) = result {
        error!("The web server stopped: {error}");
    }
}
