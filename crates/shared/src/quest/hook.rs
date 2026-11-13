//! What a poi offers quests: its hooks. A boss, an item and a lost one are
//! each one quest's alone (`owner`); counted kills, exploring, visiting and
//! gathering may be shared, as nobody's progress takes from anyone else's.

use serde::{Deserialize, Serialize};

use crate::creature::CreatureKind;
use crate::mob::who_lives_in;
use crate::poi::{Poi, PoiKind};
use crate::story::PlaceKind;

/// One thing a quest can be about at a poi.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Hook {
    /// A lone named boss, part of the site from the start.
    Boss,
    /// A unique quest item somewhere in the site.
    Item,
    /// Someone lost there (a pet or a relative), to be led home.
    Escortee,
    /// Counted kills of the site's creatures of this kind.
    Kills(CreatureKind),
    /// Reaching the site's deepest level.
    Explore,
    /// Going there (and having a look).
    Visit,
    /// Gathering herbs there.
    Gather,
    /// Bringing back a part of the site's creatures of this kind, dropped only
    /// for the quester (owner).
    Collect(CreatureKind),
    // New hooks go at the end: messages carry one by its place.
}

impl Hook {
    /// Whether only one quest may use it.
    pub const fn is_exclusive(self) -> bool {
        matches!(self, Self::Boss | Self::Item | Self::Escortee)
    }

    /// What a poi offers, in a fixed order. Towns offer nothing: they are
    /// only delivered to (`Target::Town`).
    pub fn of_poi(poi: &Poi) -> Vec<Self> {
        let kind = poi.kind;
        if kind.is_town() {
            return Vec::new();
        }
        let place = PlaceKind::of_poi(kind);
        let mut hooks = Vec::new();
        if has_boss(place) {
            hooks.push(Self::Boss);
        }
        hooks.push(Self::Item);
        if matches!(
            place,
            PlaceKind::Dwelling
                | PlaceKind::Farm
                | PlaceKind::Cave
                | PlaceKind::Ruin
                | PlaceKind::Camp
        ) {
            hooks.push(Self::Escortee);
        }
        if let Some(kind) = slain_kind(poi) {
            hooks.push(Self::Kills(kind));
            hooks.push(Self::Collect(kind));
        }
        if has_boss(place) && place != PlaceKind::Camp {
            hooks.push(Self::Explore);
        }
        hooks.push(Self::Visit);
        if place == PlaceKind::HerbalistHut {
            hooks.push(Self::Gather);
        }
        hooks
    }
}

/// Places a boss could lead others from.
fn has_boss(place: PlaceKind) -> bool {
    matches!(
        place,
        PlaceKind::Cave
            | PlaceKind::Crypt
            | PlaceKind::Ruin
            | PlaceKind::Tower
            | PlaceKind::Stronghold
            | PlaceKind::Camp
            | PlaceKind::Pyramid
    )
}

/// The kind a poi's counted kills are of: one of who lives there (less the
/// harmless), by the poi's own seed, likelier the more of them there are.
pub(super) fn slain_kind(poi: &Poi) -> Option<CreatureKind> {
    let kinds = slain_with_weights(poi.kind);
    (!kinds.is_empty()).then(|| poi.seed.derive("quest kills").rng().pick_weighted(&kinds))
}

/// As `slain_kinds`, with how many of each live there in 100.
pub(super) fn slain_with_weights(kind: PoiKind) -> Vec<(CreatureKind, u64)> {
    (who_lives_in(kind).iter())
        .filter(|(creature, _)| is_slain(*creature))
        .copied()
        .collect()
}

fn is_slain(kind: CreatureKind) -> bool {
    !matches!(
        kind,
        CreatureKind::Chicken | CreatureKind::Cattle | CreatureKind::Horse | CreatureKind::Beggar
    )
}
