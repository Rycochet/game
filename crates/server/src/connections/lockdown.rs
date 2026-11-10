//! Lockdown (owner, 2026-11-10): a networked server can't go offline as an
//! embedded one does, so instead it can be locked down, letting in only
//! accounts with Server Options (Owners and Admins by default). Those may
//! turn it on and off from their Online (`SetLockdown`); `HOW_LOCKDOWN` starts
//! it on. It is kept in `server.ron`, so it lasts past a restart, and each
//! change goes in the audit log.

use bevy::prelude::*;
use shared::protocol::ServerMessage;
use shared::role::Permission;

use super::{Client, Connections};
use crate::storage::now;
use crate::{ServerOptions, ServerWorlds};

/// What anyone else is told, logging in or sent away.
pub(crate) const LOCKED_DOWN: &str =
    "this server is locked down: only its staff can play on it just now";

impl Client {
    /// Whether lockdown keeps it out: logged in, and without Server Options.
    pub(super) fn kept_out(&self, options: &ServerOptions) -> bool {
        options.lockdown && !self.may(Permission::ServerOptions)
    }

    /// Tells it whether the server is locked down, if it may change that.
    /// `false` if it has gone.
    pub(super) fn send_lockdown(&self, options: &ServerOptions) -> bool {
        if !options.lockable || !self.may(Permission::ServerOptions) {
            return true;
        }
        self.link.send(&ServerMessage::Lockdown(options.lockdown))
    }
}

/// After every client's messages: does what was asked of lockdown, keeping and
/// noting it, telling everyone who may change it; then, while locked down,
/// sends away anyone playing who may not (as a role taken away does too).
pub(crate) fn apply_lockdown(
    mut connections: ResMut<Connections>,
    mut worlds: ResMut<ServerWorlds>,
    mut options: ResMut<ServerOptions>,
) {
    let asks: Vec<_> = connections
        .clients
        .iter_mut()
        .filter_map(|client| Some((client.asked_lockdown.take()?, client.account?)))
        .collect();
    for (on, account) in asks {
        if on == options.lockdown {
            continue;
        }
        options.lockdown = on;
        worlds.keep_lockdown(on);
        let what = match on {
            true => "locked the server down",
            false => "opened the server again",
        };
        let note = format!("Account {} {what}", account.0);
        connections.accounts.roles.record(now(), &note);
        for client in connections.clients.iter().filter(|c| !c.gone) {
            client.send_lockdown(&options);
        }
    }
    if !options.lockdown {
        return;
    }
    for client in connections.clients.iter_mut() {
        let Some(account) = client.account else {
            continue;
        };
        if !client.gone && client.kept_out(&options) {
            info!("Account {} was sent away: locked down", account.0);
            client
                .link
                .send(&ServerMessage::Refused(LOCKED_DOWN.into()));
            client.disconnect(worlds.of(client.world));
        }
    }
    connections.clients.retain(|client| !client.gone);
}
