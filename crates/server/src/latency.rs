//! How long a client's connection takes, measured by the server itself: the
//! WebSocket pings it sends (`Pings`) are answered by the client's network code,
//! so a frozen game doesn't count, and nothing the client says is believed
//! (`Latency`). A client can make itself look slower by answering late, so this
//! must never earn it anything; it can't look faster, as the answer has to
//! carry a number it only learns from the ping.

use std::collections::VecDeque;
use std::hash::{BuildHasher, RandomState};
use std::time::{Duration, Instant};

/// The most pings remembered while waiting for answers.
const MAX_WAITING: usize = 64;

/// How much of each new measurement goes into the averages (as TCP does it).
const SMOOTHING: f64 = 1.0 / 8.0;
const JITTER_SMOOTHING: f64 = 1.0 / 4.0;

/// A connection this slow on average, in milliseconds, is having trouble...
const SLOW_MS: f64 = 400.0;
/// ...or one varying this much, which on TCP is how lost packets show up.
const JITTER_MS: f64 = 150.0;
/// It is over the trouble once both are under this share of those.
const RECOVERED_SHARE: f64 = 0.5;

/// The pings sent on one connection and not yet answered.
pub(crate) struct Pings {
    /// Makes each ping's number unguessable.
    key: RandomState,
    sent: u64,
    waiting: VecDeque<(u64, Instant)>,
}

impl Pings {
    pub fn new() -> Self {
        Self {
            key: RandomState::new(),
            sent: 0,
            waiting: VecDeque::new(),
        }
    }

    /// What to send in a ping going now.
    pub fn next(&mut self, now: Instant) -> [u8; 8] {
        self.sent += 1;
        let number = self.key.hash_one(self.sent);
        if self.waiting.len() == MAX_WAITING {
            self.waiting.pop_front();
        }
        self.waiting.push_back((number, now));
        number.to_le_bytes()
    }

    /// How long the ping this answers took, if it answers one. Earlier pings
    /// are forgotten: a client need only answer the latest it has seen.
    pub fn answered(&mut self, payload: &[u8], now: Instant) -> Option<Duration> {
        let number = u64::from_le_bytes(payload.try_into().ok()?);
        let index = self.waiting.iter().position(|&(n, _)| n == number)?;
        let (_, sent) = self.waiting[index];
        self.waiting.drain(..=index);
        Some(now.duration_since(sent))
    }

    /// When the oldest ping still waiting was sent.
    pub fn waiting_since(&self) -> Option<Instant> {
        self.waiting.front().map(|&(_, sent)| sent)
    }
}

/// What the round trips of one connection have been like.
#[derive(Default)]
pub(crate) struct Latency {
    smoothed_ms: Option<f64>,
    jitter_ms: f64,
    troubled: bool,
}

impl Latency {
    /// Takes in a round trip. `Some` when this changes whether the connection
    /// is in trouble.
    pub fn measured(&mut self, round_trip: Duration) -> Option<bool> {
        let ms = round_trip.as_secs_f64() * 1000.0;
        match self.smoothed_ms {
            None => self.smoothed_ms = Some(ms),
            Some(smoothed) => {
                self.jitter_ms += ((ms - smoothed).abs() - self.jitter_ms) * JITTER_SMOOTHING;
                self.smoothed_ms = Some(smoothed + (ms - smoothed) * SMOOTHING);
            }
        }
        let smoothed = self.smoothed_ms.unwrap_or(ms);
        let trouble = if self.troubled {
            smoothed > SLOW_MS * RECOVERED_SHARE || self.jitter_ms > JITTER_MS * RECOVERED_SHARE
        } else {
            smoothed > SLOW_MS || self.jitter_ms > JITTER_MS
        };
        self.set_troubled(trouble)
    }

    /// A ping has gone unanswered for too long. `Some` as for `measured`.
    pub fn no_answer(&mut self) -> Option<bool> {
        self.set_troubled(true)
    }

    fn set_troubled(&mut self, troubled: bool) -> Option<bool> {
        let changed = self.troubled != troubled;
        self.troubled = troubled;
        changed.then_some(troubled)
    }

    /// The average round trip so far, in milliseconds.
    pub fn smoothed_ms(&self) -> Option<f64> {
        self.smoothed_ms
    }

    /// How much round trips vary, in milliseconds.
    pub fn jitter_ms(&self) -> f64 {
        self.jitter_ms
    }
}

#[cfg(test)]
mod tests;
