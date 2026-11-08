//! How long a character has been played, and how long since its player last did
//! anything: playtime only counts while they are about, and the idle are sent away.

use shared::energy::ACTION_TICKS_PER_SECOND;
use shared::protocol::{ClientMessage, ServerMessage};

use super::Client;

/// Action ticks in a minute.
const TICKS_PER_MINUTE: u64 = 60 * ACTION_TICKS_PER_SECOND as u64;

/// Playtime stops counting this long after the player last did anything:
/// 5 minutes of action ticks (owner, 2026-11-06).
pub(crate) const ACTIVE_FOR: u64 = 5 * TICKS_PER_MINUTE;

/// How long a player may do nothing before being disconnected, unless the
/// server says otherwise (`ServerOptions::idle_minutes`, `HOW_IDLE_MINUTES`).
/// Longer than refilling Health and Mana takes, as resting isn't idling.
pub const IDLE_MINUTES: u16 = 15;

/// Playtime and the last thing done, in action ticks: these stop with the
/// clock when paused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Activity {
    /// All the character's playtime, saved with them (`CharacterSave::playtime`).
    pub(crate) played: u64,
    /// When the player last did something.
    last_done: u64,
    /// The tick playtime has been counted to.
    counted_to: u64,
}

impl Activity {
    /// A character joining at `now`, played this long before.
    pub(crate) fn joined(played: u64, now: u64) -> Self {
        Self {
            played,
            last_done: now,
            counted_to: now,
        }
    }

    /// The player did something.
    pub(crate) fn did_something(&mut self, now: u64) {
        self.last_done = self.last_done.max(now);
    }

    /// Counts playtime up to `now`, as far as `ACTIVE_FOR` after the player
    /// last did something. Whether it grew.
    pub(crate) fn count(&mut self, now: u64) -> bool {
        let until = now.min(self.last_done + ACTIVE_FOR);
        let grew = until.saturating_sub(self.counted_to);
        self.played += grew;
        self.counted_to = self.counted_to.max(now);
        grew > 0
    }

    /// How long since the player last did anything.
    pub(crate) fn idle_for(&self, now: u64) -> u64 {
        now.saturating_sub(self.last_done)
    }
}

/// Whether this message is the player doing something. Pings, looking items
/// up and the like happen by themselves; so do logging in, the host's own
/// settings and the player's difficulty.
pub(super) fn shows_activity(message: &ClientMessage) -> bool {
    !matches!(
        message,
        ClientMessage::Hello { .. }
            | ClientMessage::Login(_)
            | ClientMessage::ChooseCharacter { .. }
            | ClientMessage::CreateCharacter { .. }
            | ClientMessage::Ping { .. }
            | ClientMessage::LookUpItems(_)
            | ClientMessage::SetOnline(_)
            | ClientMessage::SetPaused(_)
            | ClientMessage::SetDifficulty(_)
            | ClientMessage::SetDifficulties(_)
    )
}

impl Client {
    /// Counts its playtime to `now`, rusting skills set aside, and sends it
    /// away if it has done nothing
    /// for `idle_minutes` (0: never). The host, whose server it is, is never
    /// sent away. `false` if the client is to be disconnected.
    pub(super) fn keep_time(&mut self, now: u64, idle_minutes: u16) -> bool {
        let before = self.activity.played;
        if self.activity.count(now) {
            self.unsaved = true;
            // Skills rust from disuse, looked at once a minute of playtime.
            let played = self.activity.played;
            if played / TICKS_PER_MINUTE != before / TICKS_PER_MINUTE
                && self.sheet.skills.rust(played)
                && !self.link.send(&ServerMessage::Sheet(self.sheet.clone()))
            {
                return false;
            }
        }
        let limit = u64::from(idle_minutes) * TICKS_PER_MINUTE;
        if self.host || limit == 0 || self.activity.idle_for(now) < limit {
            return true;
        }
        bevy::log::info!(
            "{:?} was idle for {idle_minutes} minutes",
            self.character_id
        );
        self.link.send(&ServerMessage::Refused(format!(
            "you were idle for {idle_minutes} minutes"
        )));
        false
    }
}

#[cfg(test)]
mod tests;
