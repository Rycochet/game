//! Where a character was before going into a Dev Area.

use serde::{Deserialize, Serialize};
use shared::spot::Spot;

use super::saves::VisitSave;

/// Where a character was before going into a Dev Area, to go back to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DevReturn {
    pub world_position: (i64, i64),
    /// The site level, or the wilderness, they were on, if any.
    pub visit: Option<VisitSave>,
    /// Where in the Hall they were, if there.
    #[serde(default, deserialize_with = "super::spot_compat::optional_spot")]
    pub hall: Option<Spot>,
}
