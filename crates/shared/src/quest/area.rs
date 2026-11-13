//! What near a town its quests may use: each poi's hooks within the town's
//! radius (exclusive ones only if the town owns them), nearer ones likelier,
//! and gathering in the wilds round the town itself.

use super::Hook;
use super::owner::owner_among;
use super::world::{apart_squared, pois_within, radius};
use crate::poi::{CHUNK_SIZE, Poi, PoiMap};

/// A hook a quest may use, and how likely it is to be chosen.
#[derive(Clone, Copy, Debug)]
pub(super) struct Candidate {
    /// The poi, or the town itself for the wilds round it.
    pub poi: Poi,
    pub hook: Hook,
    pub weight: u64,
}

impl Candidate {
    /// Whether it is the wilds round the town rather than a poi.
    pub fn is_wilds(&self) -> bool {
        self.poi.kind.is_town()
    }
}

/// Everything near `town` its quests may use, in a fixed order. `towns`:
/// every town within `radius + MOST_RADIUS` of it, sorted by chunk, to
/// decide who owns what. Dailies use only shared hooks, nearer.
pub(super) fn candidates(pois: &PoiMap, town: &Poi, towns: &[Poi], daily: bool) -> Vec<Candidate> {
    let reach = radius(town.kind, daily);
    let mut found = vec![Candidate {
        poi: *town,
        hook: Hook::Gather,
        weight: hook_weight(Hook::Gather) * (reach as u64 + 1),
    }];
    for poi in pois_within(pois, town, reach) {
        if poi.kind.is_town() {
            continue;
        }
        let apart = apart_squared(pois, town, &poi).isqrt();
        let nearness = (reach * CHUNK_SIZE - apart).max(0) / CHUNK_SIZE + 1;
        for hook in Hook::of_poi(&poi) {
            if hook.is_exclusive()
                && (daily || owner_among(pois, towns, &poi, hook) != Some(town.chunk))
            {
                continue;
            }
            found.push(Candidate {
                poi,
                hook,
                weight: hook_weight(hook) * nearness as u64,
            });
        }
    }
    found
}

/// How often each hook is wanted.
fn hook_weight(hook: Hook) -> u64 {
    match hook {
        Hook::Boss | Hook::Item | Hook::Kills(_) => 3,
        Hook::Escortee | Hook::Explore | Hook::Gather | Hook::Collect(_) => 2,
        Hook::Visit => 1,
    }
}
