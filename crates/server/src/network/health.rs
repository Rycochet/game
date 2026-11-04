//! Whether the server is well, at `HEALTH_PATH`, and what it is, at
//! `VERSION_PATH`: answered on every port, the plain one too while it otherwise
//! only says to use the secure one, so a health check needs no certificate.
//!
//! Healthy means the game is still running, not only the web server: the game
//! stamps a `Heartbeat` every tick, and a stale one answers 503.

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use bevy::prelude::*;
use shared::protocol::{ServerVersion, VERSION_PATH};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Where the server says whether it is well (`200 ok`, or `503`).
pub const HEALTH_PATH: &str = "/_healthz";

/// The game is unwell once it hasn't ticked for this long (a tick is 50 ms).
const HEALTHY_WITHIN: Duration = Duration::from_secs(10);

/// When the game last ticked, readable by the network thread. Cloning it gives
/// another handle on the same time.
#[derive(Resource, Clone, Default)]
pub(crate) struct Heartbeat(Arc<Mutex<Option<Instant>>>);

impl Heartbeat {
    /// The game has ticked.
    pub(crate) fn beat(&self) {
        *self.0.lock().expect("never poisoned") = Some(Instant::now());
    }

    /// Whether the game has ticked lately.
    fn healthy(&self) -> bool {
        let last = *self.0.lock().expect("never poisoned");
        last.is_some_and(|at| at.elapsed() < HEALTHY_WITHIN)
    }
}

/// Keeps the app's `Heartbeat` (added if it has none) beating every tick.
pub(crate) fn track_heartbeat(app: &mut App) -> Heartbeat {
    if let Some(heartbeat) = app.world().get_resource::<Heartbeat>() {
        return heartbeat.clone();
    }
    let heartbeat = Heartbeat::default();
    app.insert_resource(heartbeat.clone())
        .add_systems(Last, |heartbeat: Res<Heartbeat>| heartbeat.beat());
    heartbeat
}

/// The health and version routes.
pub(super) fn health_routes(heartbeat: Heartbeat) -> Router {
    Router::new()
        .route(
            HEALTH_PATH,
            get(move || async move {
                let status = if heartbeat.healthy() {
                    StatusCode::OK
                } else {
                    StatusCode::SERVICE_UNAVAILABLE
                };
                (
                    status,
                    [(header::CACHE_CONTROL, "no-store")],
                    status_text(status),
                )
            }),
        )
        .route(VERSION_PATH, get(version))
}

fn status_text(status: StatusCode) -> &'static str {
    if status.is_success() {
        "ok\n"
    } else {
        "the game has stopped\n"
    }
}

/// This server's `ServerVersion`, readable by web clients from other sites.
async fn version() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
            (header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
        ],
        ServerVersion::this().to_json(),
    )
}
