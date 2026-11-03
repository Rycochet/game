//! Stopping when asked, by a signal or by the app.

use bevy::prelude::*;
use std::sync::atomic::Ordering;

use super::StopAsked;
use crate::ServerWorld;
use crate::connections::Connections;

/// Asked to stop: everyone is saved and disconnected, then the server stops.
pub(super) fn stop_when_asked(
    stop: Res<StopAsked>,
    mut connections: ResMut<Connections>,
    mut world: ResMut<ServerWorld>,
    mut exit: MessageWriter<AppExit>,
) {
    if stop.0.load(Ordering::Relaxed) {
        info!("Stopping: saving everyone");
        connections.disconnect_all(&mut world);
        world.storage.finish_writing();
        exit.write(AppExit::Success);
    }
}

/// Waits for Ctrl+C, or (on Unix) SIGTERM.
pub(super) async fn stop_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).expect("can watch for SIGTERM");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = term.recv() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
