//! What the generator reads of the world, and the sizes of towns' reach.

use crate::poi::{CHUNK_SIZE, Poi, PoiKind, PoiMap};
use crate::road::{RoadKind, Roads};

/// The world as the generator sees it. Only the server has one: the pois'
/// seeds are secret.
pub struct QuestWorld<'a> {
    pub pois: &'a PoiMap,
    pub roads: &'a Roads,
    /// A place's name as players know it, if it has one (towns, castles,
    /// forts: the server's `place_name_of`, which needs the place's height).
    pub name_of: &'a dyn Fn(&Poi) -> Option<String>,
    /// How many moons the world has (`Celestial::moon_count`), for quests
    /// that wait for one in the sky.
    pub moons: u8,
}

/// The largest `radius`, in chunks: how far a poi may be from a town whose
/// quests could use it.
pub(super) const MOST_RADIUS: i64 = 7;

/// How far from a town its quests go, in chunks (about the road `RANGE` for
/// a town, less for smaller ones): dailies stay within half.
pub fn radius(kind: PoiKind, daily: bool) -> i64 {
    let full = match kind {
        PoiKind::Hamlet => 3,
        PoiKind::Village => 4,
        PoiKind::Town => 6,
        _ => MOST_RADIUS,
    };
    if daily { (full + 1) / 2 } else { full }
}

/// A town's size, smallest 0, or `None` for anything else.
pub(super) fn size(kind: PoiKind) -> Option<usize> {
    let road = RoadKind::of_town(kind)?;
    RoadKind::ALL.iter().position(|&k| k == road)
}

/// Whether town `a` counts as larger than town `b` for chains: by size,
/// ties by chunk order, so every pair of towns has one larger.
pub fn is_larger(a: &Poi, b: &Poi) -> bool {
    (size(a.kind), a.chunk) > (size(b.kind), b.chunk)
}

/// The near way round the planet from `from` to `to`, in grid points.
pub(super) fn offset(pois: &PoiMap, from: (i64, i64), to: (i64, i64)) -> (i64, i64) {
    let width = pois.width().max(1);
    let mut dx = (to.0 - from.0).rem_euclid(width);
    if dx > width / 2 {
        dx -= width;
    }
    (dx, to.1 - from.1)
}

/// How far apart two pois are, squared, in grid points.
pub(super) fn apart_squared(pois: &PoiMap, a: &Poi, b: &Poi) -> i64 {
    let (dx, dy) = offset(pois, (a.x, a.y), (b.x, b.y));
    dx * dx + dy * dy
}

/// Whether `poi` is within `chunks` of `town`.
pub(super) fn within(pois: &PoiMap, town: &Poi, poi: &Poi, chunks: i64) -> bool {
    apart_squared(pois, town, poi) <= (chunks * CHUNK_SIZE).pow(2)
}

/// Every poi within `chunks` of `around`, in a fixed order (by chunk).
pub(super) fn pois_within(pois: &PoiMap, around: &Poi, chunks: i64) -> Vec<Poi> {
    let reach = chunks * CHUNK_SIZE;
    let mut found: Vec<Poi> = (pois.in_area(
        (around.x - reach, around.y - reach),
        (around.x + reach, around.y + reach),
    ))
    .into_iter()
    .filter(|poi| within(pois, around, poi, chunks))
    .collect();
    found.sort_by_key(|poi| poi.chunk);
    found.dedup_by_key(|poi| poi.chunk);
    found
}
