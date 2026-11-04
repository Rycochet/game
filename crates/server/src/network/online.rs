//! An embedded server opened to other players: its host (the game's own
//! player) asks with `SetOnline`, and its secure port opens, or closes again.
//! Only the secure port: there is no proxy in front of a player's own game.
//!
//! While open, guests may log in, nobody can pause, and dev tools are the host's
//! alone. Closing tells the other players it is going offline on purpose, so
//! they don't try again, and saves them.

use bevy::prelude::*;
use shared::protocol::{OnlineSettings, OnlineState, ServerAddress, ServerMessage};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Mutex;

use super::NewConnections;
use super::health::{Heartbeat, track_heartbeat};
use super::ports::{OpenPorts, open_ports};
use super::routes::{Stop, accept_connections};
use crate::connections::{self, Connections};
use crate::tls::{CertificateSource, Security};
use crate::{ServerName, ServerOptions, ServerWorlds};

/// Lets an embedded server's host open it to other players. `data` is its data
/// folder, where the certificate is kept (`tls/`).
pub struct OnlinePlugin {
    pub data: Option<PathBuf>,
}

/// Where the certificate is kept.
#[derive(Resource)]
struct CertificateFolder(Option<PathBuf>);

/// The open port, if there is one.
#[derive(Resource, Default)]
struct OnlineSlot(Option<Online>);

/// An open port, closed when this is dropped.
struct Online {
    /// Dropping it tells the network thread to stop.
    closing: Option<tokio::sync::watch::Sender<()>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Online {
    fn drop(&mut self) {
        self.closing.take();
        if let Some(thread) = self.thread.take() {
            // So the port is free before it is opened again.
            let _ = thread.join();
        }
    }
}

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        track_heartbeat(app);
        app.insert_resource(CertificateFolder(
            self.data.as_ref().map(|data| data.join("tls")),
        ))
        .init_resource::<OnlineSlot>()
        .add_systems(
            Update,
            (
                accept_connections
                    .run_if(resource_exists::<NewConnections>)
                    .before(connections::handle_messages),
                go_online.after(connections::handle_messages),
            ),
        );
    }
}

/// Does what the host asked: opens the port (again, with new settings), or
/// closes it.
#[allow(clippy::too_many_arguments)]
fn go_online(
    mut commands: Commands,
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
    mut options: ResMut<ServerOptions>,
    mut name: ResMut<ServerName>,
    folder: Res<CertificateFolder>,
    mut slot: ResMut<OnlineSlot>,
    heartbeat: Res<Heartbeat>,
) {
    let Some(asked) = connections.take_online_request() else {
        return;
    };
    if slot.0.is_some() {
        let going = ServerMessage::GoingOffline(name.0.clone());
        connections.send_others_away(&mut worlds, &going);
        // Closed now, so the port is free to open again below.
        slot.0 = None;
        commands.remove_resource::<NewConnections>();
        info!("Closed to other players");
    }
    options.online = false;
    options.guests = false;
    options.pausable = true;
    let Some(settings) = asked else {
        connections.tell_host(&ServerMessage::Online(OnlineState::Offline));
        return;
    };
    let opened = open(&settings, &folder, &worlds, &heartbeat);
    let (ports, closing) = match opened {
        Ok(opened) => opened,
        Err(reason) => {
            warn!("Can't open to other players: {reason}");
            connections.tell_host(&ServerMessage::Online(OnlineState::Failed(reason)));
            return;
        }
    };
    name.0.clone_from(&settings.name);
    options.online = true;
    options.guests = true;
    options.pausable = false;
    options.player_limit = settings.player_limit;
    if worlds.clock().is_paused() {
        worlds.clock().resume();
        connections.tell_host(&ServerMessage::Paused {
            paused: false,
            clock: worlds.clock().seconds(),
        });
    }
    let (certificate, fingerprint) = ports.certificate.unwrap_or_default();
    info!("Open to other players as {:?}", settings.name);
    commands.insert_resource(NewConnections(Mutex::new(ports.accepted)));
    slot.0 = Some(Online {
        closing: Some(closing),
        thread: Some(ports.thread),
    });
    connections.tell_host(&ServerMessage::Online(OnlineState::Online {
        certificate,
        fingerprint,
    }));
}

/// Opens the secure port these settings ask for.
fn open(
    settings: &OnlineSettings,
    folder: &CertificateFolder,
    worlds: &ServerWorlds,
    heartbeat: &Heartbeat,
) -> Result<(OpenPorts, tokio::sync::watch::Sender<()>), String> {
    let ip: IpAddr = settings.bind_address.trim().parse().map_err(|_| {
        format!(
            "{:?} isn't an address of this computer",
            settings.bind_address
        )
    })?;
    let Some(folder) = folder.0.clone() else {
        return Err("this server keeps no files, so it has no certificate".into());
    };
    // The public address's host goes in the certificate, for browsers' sake.
    let names = ServerAddress::parse(&settings.public_address)
        .map(|address| vec![address.host])
        .unwrap_or_default();
    let security = Security::Tls {
        address: SocketAddr::new(ip, settings.port),
        source: CertificateSource::Generated { folder, names },
        plain_too: false,
    };
    let (closing, closed) = tokio::sync::watch::channel(());
    let ports = open_ports(
        worlds.clock().clone(),
        heartbeat.clone(),
        None,
        &security,
        None,
        Stop::Closing(closed),
    )?;
    Ok((ports, closing))
}
