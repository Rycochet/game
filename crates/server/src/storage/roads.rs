//! Loading and saving roads, each worked out once per world.

use serde::{Deserialize, Serialize};
use shared::road::{ROAD_VERSION, Road, RoadLink};

use super::Storage;
use crate::saves::{Save, SaveKey};

/// `roads/<a>_<b>.ron`: the road between two towns, kept only so it needn't be
/// worked out again. It can always be: losing one costs nothing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoadSave {
    /// The `ROAD_VERSION` it was worked out with: another is worked out again.
    pub version: u32,
    /// `None`: there is water between the towns, so no road.
    pub road: Option<Road>,
}

impl Storage {
    /// A road saved by this version of the game: `Some(None)` if its towns
    /// can't be joined, `None` if it needs working out (also when it couldn't be
    /// read: it would only be worked out again).
    pub fn load_road(&self, link: RoadLink) -> Option<Option<Road>> {
        let queue = self.queue.as_ref()?;
        match queue.load(&SaveKey::Road(self.world, link)) {
            Ok(Some(Save::Road(save))) if save.version == ROAD_VERSION => Some(save.road),
            _ => None,
        }
    }

    pub fn save_road(&self, link: RoadLink, road: Option<Road>) {
        if let Some(queue) = &self.queue {
            let save = RoadSave {
                version: ROAD_VERSION,
                road,
            };
            queue.save(SaveKey::Road(self.world, link), Save::Road(save));
        }
    }
}
