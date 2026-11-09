//! What a client asks of doors.

use serde::{Deserialize, Serialize};

/// Something done to doors, in turn with the player's steps
/// (`ClientMessage::Door`). The new state reaches the client with its level's
/// cells (`LevelChunk::doors`); one that can't be done is answered with
/// `ServerMessage::DoorRefused`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoorAsk {
    /// Open the closed door at `(x, y)`, next to the player (walking into
    /// it): away from them, or towards them for a secret door. A locked one is
    /// unlocked first, if they carry its key.
    Open { x: i64, y: i64 },
    /// Close every open door next to the player with nothing in its doorway,
    /// and lock every closed one whose key they carry (the L key). Broken
    /// doors can't be closed.
    CloseNear,
    /// Developer tools: open, or close, every door of the level the player is
    /// on that can be.
    DevAll { open: bool },
    /// Developer tools: the key to every lock on a door beside the player, or
    /// if none, to the nearest lock on their level (Key for This Door).
    DevKey,
}
