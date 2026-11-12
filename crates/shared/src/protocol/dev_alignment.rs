//! What dev tools can do to a character's hidden alignment (the debug menu's
//! Alignment panel).

use serde::{Deserialize, Serialize};

/// What `ClientMessage::DevAlignment` does, always answered with
/// `ServerMessage::DevAlignment`: the only way a client learns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevAlignment {
    /// Only asks.
    Ask,
    /// Moves it by this much.
    Shift(i16),
    /// Sets it to this.
    Set(i16),
}
