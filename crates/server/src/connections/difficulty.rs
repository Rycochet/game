//! A character's difficulty (`shared::difficulty`): deciding it on joining,
//! changing it when the player asks, and moving everyone onto one the server
//! allows when the host changes what it allows.

use bevy::prelude::*;
use shared::difficulty::{Difficulties, Difficulty};
use shared::protocol::ServerMessage;

use super::{Client, Connections, Phase};
use crate::{ServerOptions, ServerWorlds};

impl Client {
    /// The difficulty the character plays on: Normal until decided.
    pub(crate) fn playing_on(&self) -> Difficulty {
        self.difficulty.unwrap_or_default()
    }

    /// Tells the client its character's difficulty and what is allowed.
    /// `false` if it has gone.
    pub(super) fn send_difficulty(&self, allowed: Difficulties) -> bool {
        let current = match self.phase {
            Phase::Playing => Some(self.playing_on()),
            _ => None,
        };
        self.link
            .send(&ServerMessage::Difficulty { current, allowed })
    }

    /// On joining: what was saved, or else what they were made with or their
    /// player's default (`Hello`), moved onto the nearest the server allows.
    pub(super) fn decide_difficulty(&mut self, saved: Option<Difficulty>, allowed: Difficulties) {
        let wanted = saved.or(self.difficulty).unwrap_or_default();
        let decided = allowed.closest(wanted);
        if saved != Some(decided) {
            self.unsaved = true;
        }
        self.difficulty = Some(decided);
    }

    /// Puts the character on `to`, Health and Mana settled first at the old
    /// refilling. Tells the client its vitals again if it changed.
    fn set_difficulty(&mut self, to: Difficulty, now: u64) {
        if self.difficulty == Some(to) {
            return;
        }
        let before = self.limits(now);
        self.sheet.vitals.settle(before, now);
        self.difficulty = Some(to);
        self.unsaved = true;
        self.send_vitals(now);
    }

    /// The player asks for another difficulty (`SetDifficulty`): only one the
    /// server allows, and never to or from Ironman. Always answered with
    /// `Difficulty`. `false` if the client has gone.
    pub(super) fn change_difficulty(
        &mut self,
        to: Difficulty,
        allowed: Difficulties,
        now: u64,
    ) -> bool {
        if allowed.contains(to) && self.playing_on().may_change_to(to) {
            info!("{} now plays on {}", self.in_log(), to.name());
            self.set_difficulty(to, now);
        }
        self.send_difficulty(allowed)
    }

    /// The host asks the server to allow these (`SetDifficulties`).
    pub(super) fn ask_difficulties(&mut self, allowed: Difficulties) {
        if self.host {
            self.asked_difficulties = Some(allowed.or_all());
        } else {
            warn!("Ignoring SetDifficulties from someone other than the host");
        }
    }
}

/// After every client's messages: what the host asked to allow becomes the
/// server's (unless `HOW_DIFFICULTIES` decides), anyone playing on one no
/// longer allowed is moved to the nearest, and everyone is told.
pub(crate) fn apply_difficulties(
    mut connections: ResMut<Connections>,
    mut options: ResMut<ServerOptions>,
    worlds: Res<ServerWorlds>,
) {
    let asked = connections
        .clients
        .iter_mut()
        .filter_map(|client| client.asked_difficulties.take())
        .last();
    let Some(asked) = asked else {
        return;
    };
    if options.difficulties_fixed {
        warn!("Ignoring the host's difficulties: HOW_DIFFICULTIES sets them");
    } else if asked != options.difficulties {
        info!("Difficulties allowed: {}", asked.names());
        options.difficulties = asked;
    }
    let allowed = options.difficulties;
    let now = worlds.action_clock().ticks();
    for client in connections.clients.iter_mut().filter(|client| !client.gone) {
        if client.phase == Phase::Playing {
            let moved = allowed.closest(client.playing_on());
            client.set_difficulty(moved, now);
        }
        if matches!(client.phase, Phase::Playing | Phase::Choosing) {
            client.send_difficulty(allowed);
        }
    }
}
