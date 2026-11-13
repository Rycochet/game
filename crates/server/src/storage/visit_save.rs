//! Where in a site a character is, saved with them.

use serde::{Deserialize, Serialize};
use shared::spot::Spot;

use super::SiteId;

/// Where in a site a character is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisitSave {
    pub site: SiteId,
    pub level: u32,
    /// Where on it, in units (older saves' cells read as their middles).
    #[serde(deserialize_with = "super::spot_compat::spot")]
    pub position: Spot,
    /// They walked into the site from the wilderness, where leaving it takes
    /// them back.
    #[serde(default)]
    pub from_wilderness: bool,
}
