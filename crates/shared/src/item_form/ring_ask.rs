//! What a new ring must be, if anything: its metal, how many settings, and its gems.

use crate::item::Quality;
use crate::item_form::gem::GemAsk;
use crate::item_form::gem::fine_weight;
use crate::item_form::request::{Field, Refused, RollRequest, keep_only};
use crate::item_form::ring_roll::{RING_METALS, setting_counts};
use crate::material::Material;

/// A request's ring options.
#[derive(Clone, Copy, Default)]
pub(super) struct RingAsk {
    pub(super) metal: Option<Material>,
    pub(super) settings: Option<u8>,
    pub(super) gem: GemAsk,
}

impl RingAsk {
    pub(super) fn of(request: &RollRequest) -> Self {
        Self {
            metal: request.material,
            settings: request.settings,
            gem: GemAsk::of(request),
        }
    }

    /// Asking for a gem means at least one stone to show it.
    pub(super) fn wants_stones(self) -> bool {
        self.gem.material.is_some() || self.gem.cut.is_some()
    }

    /// The metals to roll from, by how often: only those that can have the settings
    /// asked for, and only the metal asked for.
    pub(super) fn metals(self, quality: Quality) -> Vec<(Material, u64)> {
        let mut metals: Vec<_> = RING_METALS
            .into_iter()
            .map(|metal| {
                let counts = self.counts(metal, quality);
                let possible = counts.iter().any(|&(_, weight)| weight > 0);
                let weight = if possible {
                    fine_weight(metal, quality)
                } else {
                    0
                };
                (metal, weight)
            })
            .collect();
        if !keep_only(&mut metals, self.metal) {
            metals.iter_mut().for_each(|(_, weight)| *weight = 0);
        }
        metals
    }

    /// How many settings, by how often, for this metal: none never when a stone is
    /// wanted, and only the number asked for.
    pub(super) fn counts(self, metal: Material, quality: Quality) -> [(u8, u64); 5] {
        let mut counts = setting_counts(metal, quality);
        if self.wants_stones() {
            counts[0].1 = 0;
        }
        if !keep_only(&mut counts, self.settings) {
            counts = counts.map(|(count, _)| (count, 0));
        }
        counts
    }

    /// Whether a ring can be made so at this quality.
    pub(super) fn check(self, quality: Quality) -> Result<(), Refused> {
        if self
            .metal
            .is_some_and(|metal| !RING_METALS.contains(&metal))
        {
            return Err(Refused(Field::Material));
        }
        if !self.metals(quality).iter().any(|&(_, weight)| weight > 0) {
            return Err(Refused(Field::Settings));
        }
        self.gem.check(quality, false)
    }
}
