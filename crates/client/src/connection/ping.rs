//! Timing pings as close to the wire as possible: whatever carries a socket's
//! bytes sends them and spots the answers (`Pinger`), so a slow or frozen frame
//! doesn't count as network time. The game picks up the latest answer
//! (`LatestPong`) when it next runs.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use bevy::platform::time::Instant;
use shared::protocol::{ClientMessage, ServerMessage};

/// How often to ask the server's clock (which also times the round trip).
pub(super) const PING_EVERY: Duration = Duration::from_secs(1);

/// An answer to a `Ping`.
#[derive(Clone, Copy, Debug)]
pub(super) struct TimedPong {
    /// Seconds it took to come back.
    pub round_trip: f64,
    /// The world's clock when the server answered, in game minutes.
    pub clock: f64,
    /// When it arrived, as the game may only see it much later.
    pub received: Instant,
}

/// The latest answer the game hasn't seen yet. Older ones are of no use.
#[derive(Clone, Default)]
pub(super) struct LatestPong(Arc<Mutex<Option<TimedPong>>>);

impl LatestPong {
    pub fn set(&self, pong: TimedPong) {
        *self.0.lock().expect("never poisoned") = Some(pong);
    }

    pub fn take(&self) -> Option<TimedPong> {
        self.0.lock().expect("never poisoned").take()
    }
}

/// Makes pings, and recognises and times the answers.
#[derive(Clone)]
pub(super) struct Pinger {
    started: Instant,
    latest: LatestPong,
}

impl Pinger {
    pub fn new(latest: LatestPong) -> Self {
        Self {
            started: Instant::now(),
            latest,
        }
    }

    /// A `Ping` to send right now.
    pub fn ping(&self) -> Vec<u8> {
        ClientMessage::ping_bytes(self.started.elapsed().as_secs_f64())
    }

    /// Whether these bytes that just arrived were an answer, now dealt with.
    /// Anything else is for the link.
    pub fn took_pong(&self, bytes: &[u8]) -> bool {
        let Some((sent, clock)) = ServerMessage::read_pong(bytes) else {
            return false;
        };
        self.latest.set(TimedPong {
            round_trip: self.started.elapsed().as_secs_f64() - sent,
            clock,
            received: Instant::now(),
        });
        true
    }
}

#[cfg(test)]
mod tests;
