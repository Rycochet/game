//! What players are told of a door.

use serde::{Deserialize, Serialize};

use super::{DoorStyle, Swing};
use crate::material::Material;

/// A door as a player sees it, at cell `(x, y)`: sent with its level chunk
/// (`protocol::LevelChunk::doors`), as it is now or as remembered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoorLook {
    pub x: i64,
    pub y: i64,
    pub swing: Swing,
    pub material: Material,
    pub style: DoorStyle,
}
