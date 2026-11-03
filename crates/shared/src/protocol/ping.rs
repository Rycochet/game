//! Pings and pongs, as a few bytes rather than full messages.

use super::{ClientMessage, ServerMessage};

/// The size of a `Ping` as sent: its kind, then a float.
pub(super) const PING_BYTES: usize = 9;

/// The size of a `Pong` as sent: its kind, then two floats.
pub(super) const PONG_BYTES: usize = 17;

impl ClientMessage {
    /// A `Ping` as sent, for whoever carries the bytes to time it exactly.
    pub fn ping_bytes(sent: f64) -> Vec<u8> {
        postcard::to_allocvec(&Self::Ping { sent }).expect("messages always encode")
    }

    /// The `sent` of a `Ping`, if these bytes are one. Only decodes messages
    /// of a `Ping`'s size, so it can be asked of every message.
    pub fn read_ping(bytes: &[u8]) -> Option<f64> {
        if bytes.len() != PING_BYTES {
            return None;
        }
        match postcard::from_bytes(bytes) {
            Ok(Self::Ping { sent }) => Some(sent),
            _ => None,
        }
    }
}

impl ServerMessage {
    /// A `Pong` as sent, for whoever carries the bytes to answer straight away.
    pub fn pong_bytes(sent: f64, clock: f64) -> Vec<u8> {
        postcard::to_allocvec(&Self::Pong { sent, clock }).expect("messages always encode")
    }

    /// The `sent` and `clock` of a `Pong`, if these bytes are one. Only decodes
    /// messages of a `Pong`'s size, so it can be asked of every message.
    pub fn read_pong(bytes: &[u8]) -> Option<(f64, f64)> {
        if bytes.len() != PONG_BYTES {
            return None;
        }
        match postcard::from_bytes(bytes) {
            Ok(Self::Pong { sent, clock }) => Some((sent, clock)),
            _ => None,
        }
    }
}
