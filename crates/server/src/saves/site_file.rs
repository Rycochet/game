//! The name of each site's save file.

use crate::storage::SiteId;

pub(super) fn site_file_name(site: SiteId) -> String {
    match site {
        SiteId::Poi(chunk) => format!("poi_{}_{}.ron", chunk.x, chunk.y),
        SiteId::Dev {
            kind,
            seed,
            ruin: 0,
        } => format!("dev_{}_{seed}.ron", kind as usize),
        SiteId::Dev { kind, seed, ruin } => {
            format!("dev_{}_{seed}_ruin{ruin}.ron", kind as usize)
        }
        SiteId::DevArea(area) => format!("dev_area_{:016x}.ron", area.key()),
        SiteId::Wilderness => "wilderness.ron".into(),
    }
}
