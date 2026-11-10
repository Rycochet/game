//! Which towns are joined: a relative neighbourhood graph within `RANGE`.

use std::collections::BTreeMap;

use super::{MARGIN, RANGE, RoadKind, RoadLink};
use crate::poi::{Poi, PoiMap};

/// Every link whose road may pass through `min..=max`, sorted, with its kind.
pub(super) fn links_in_area(
    pois: &PoiMap,
    min: (i64, i64),
    max: (i64, i64),
) -> Vec<(RoadLink, RoadKind)> {
    // A road stays within `MARGIN` of the box its towns make, so a town more
    // than `RANGE + MARGIN` outside the area can't have one through it; a town
    // nearer two of those than they are to each other is within `RANGE` of one.
    let reach = 2 * RANGE + MARGIN;
    let towns: Vec<Poi> = pois
        .in_area(
            (min.0 - reach, min.1 - reach),
            (max.0 + reach, max.1 + reach),
        )
        .into_iter()
        .filter(|poi| poi.kind.is_town())
        .collect();
    let mut links = BTreeMap::new();
    for (i, a) in towns.iter().enumerate() {
        for b in &towns[i + 1..] {
            let apart = distance_squared(a, b);
            if apart > RANGE * RANGE || !crosses(a, b, min, max) {
                continue;
            }
            // A third town nearer both than they are to each other makes this
            // road the long way round: theirs go through it instead.
            let nearer = towns
                .iter()
                .any(|c| distance_squared(a, c) < apart && distance_squared(b, c) < apart);
            if nearer || a.chunk == b.chunk {
                continue;
            }
            let Some(kind) = RoadKind::between(a.kind, b.kind) else {
                continue;
            };
            links.insert(RoadLink::new(a.chunk, b.chunk), kind);
        }
    }
    links.into_iter().collect()
}

/// Whether the box round two towns, `MARGIN` wider, meets the area.
fn crosses(a: &Poi, b: &Poi, min: (i64, i64), max: (i64, i64)) -> bool {
    a.x.min(b.x) - MARGIN <= max.0
        && a.x.max(b.x) + MARGIN >= min.0
        && a.y.min(b.y) - MARGIN <= max.1
        && a.y.max(b.y) + MARGIN >= min.1
}

fn distance_squared(a: &Poi, b: &Poi) -> i64 {
    (a.x - b.x).pow(2) + (a.y - b.y).pow(2)
}
