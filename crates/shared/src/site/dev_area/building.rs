//! A Dev Area round a kind of poi's site: its ground floor as the generator
//! makes it, in the level's own coordinates, so its cells, town buildings,
//! fields and way in need no moving. Its stairs are left out.

use super::DevArea;
use super::layout::{clear_round, walled, with_stairs};
use crate::poi::PoiKind;
use crate::rng::Seed;
use crate::site::{Exit, Site, SiteLevel};

/// The area: `size` × `size` round the middle of the site's ground floor,
/// cut off where the floor is bigger, the rest open ground.
pub(super) fn make(area: &DevArea, kind: PoiKind) -> SiteLevel {
    let floor = Site::new(kind, Seed::new(area.seed)).level(0);
    let (fx, fy, fw, fh) = floor.level.bounds();
    let size = i64::from(area.size);
    let from = (fx + fw / 2 - size / 2, fy + fh / 2 - size / 2);
    let inside = |x: i64, y: i64| {
        (from.0..from.0 + size).contains(&x) && (from.1..from.1 + size).contains(&y)
    };
    let mut level = walled(from, size);
    for y in fy..fy + fh {
        for x in fx..fx + fw {
            if inside(x, y) {
                level.set(x, y, floor.level.cell(x, y));
                level.set_room(x, y, floor.level.room(x, y));
            }
        }
    }
    // Its own way in, if it is in the area; else the west side, cleared.
    let way_in = floor
        .arrival(Exit::WorldMap)
        .filter(|&(x, y)| inside(x, y) && level.is_walkable(x, y));
    let at = way_in.unwrap_or_else(|| {
        let at = (from.0 + 1, from.1 + size / 2);
        clear_round(&mut level, at);
        at
    });
    let mut made = with_stairs(level, at);
    // Only what is wholly inside: the rest is cut off.
    made.buildings = floor
        .buildings
        .into_iter()
        .filter(|building| {
            building.look.rects.iter().all(|rect| {
                inside(rect.x, rect.y) && inside(rect.x + rect.width - 1, rect.y + rect.height - 1)
            })
        })
        .collect();
    made.fields = floor
        .fields
        .into_iter()
        .filter(|field| {
            let rect = field.rect;
            inside(rect.x, rect.y) && inside(rect.x + rect.width - 1, rect.y + rect.height - 1)
        })
        .collect();
    made
}
