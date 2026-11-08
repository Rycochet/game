//! Lightning striking the ground: for `ServerMessage::Lightning`.

use serde::{Deserialize, Serialize};

/// Where a bolt of lightning strikes, in wilderness cells (`wilderness::SCALE`
/// to a world map point, `centre_of` a point at its middle cell), and the seed
/// its shape is drawn from, so everyone sees the same bolt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strike {
    pub x: i64,
    pub y: i64,
    pub seed: u64,
}
