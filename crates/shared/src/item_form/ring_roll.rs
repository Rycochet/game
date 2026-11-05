//! Rolling a new ring: its size, metal, band, settings and their gems.

use crate::creature::Size;
use crate::item::{Item, Quality};
use crate::item_form::gem::roll_gem;
use crate::item_form::ring_ask::RingAsk;
use crate::item_form::{BandWidth, GemForm, Mount, Pattern, Profile, RingForm, Setting};
use crate::material::Material;
use crate::rng::Rng;

/// The share out of 100 of settings left empty when a ring is made.
const EMPTY_PERCENT: u64 = 5;
/// The share out of 100 of gems after the first that match it.
const SAME_GEM_PERCENT: u64 = 90;
/// The share out of 100 of gems a size too big for their setting: people do it.
const OVERSIZED_PERCENT: u64 = 3;
/// The share out of 100 of gems a size smaller than their setting.
const UNDERSIZED_PERCENT: u64 = 20;
/// How far apart settings go round a band, in 256ths of a turn.
pub(super) const SETTING_SPACING: u8 = 18;

/// A ring made for anyone: mostly for those of ordinary size.
pub(super) fn roll_ring_size(rng: &mut Rng) -> Size {
    rng.pick_weighted(&[
        (Size::Tiny, 5),
        (Size::Small, 15),
        (Size::Medium, 60),
        (Size::Large, 15),
        (Size::Huge, 4),
        (Size::Giant, 1),
    ])
}

/// The metals rings are made of, in the order they were first rolled from, so rolls
/// stay as they were when more metals came for armour.
pub(super) const RING_METALS: [Material; 4] = [
    Material::Copper,
    Material::Gold,
    Material::Iron,
    Material::Silver,
];

/// Quality, then metal (better favours finer), then its band, then how many settings
/// (finer and better have more), then the gems. `new_gem` makes each gem an item.
/// What is asked for is kept to, if `ask.check` passed.
pub(super) fn roll_ring(
    rng: &mut Rng,
    quality: Quality,
    size: Size,
    ask: RingAsk,
    new_gem: &mut impl FnMut(GemForm, Size, Quality) -> Item,
) -> RingForm {
    let metal = rng.pick_weighted(&ask.metals(quality));
    let profile = rng.pick_weighted(&[
        (Profile::Domed, 30),
        (Profile::Flat, 30),
        (Profile::Round, 25),
        (Profile::Square, 10),
        (Profile::Knife, 5),
    ]);
    let width = rng.pick_weighted(&[
        (BandWidth::Thin, 30),
        (BandWidth::Medium, 50),
        (BandWidth::Wide, 20),
    ]);
    let pattern = rng.pick_weighted(&[
        (Pattern::Plain, 60),
        (Pattern::Hammered, 15),
        (Pattern::Engraved, 15),
        (Pattern::Twisted, 10),
    ]);
    let count = rng.pick_weighted(&ask.counts(metal, quality));
    let smaller: i8 = rng.pick_weighted(&[(0, 50), (1, 35), (2, 15)]);
    let setting_size = size.stepped(-smaller);
    let mount = rng.pick_weighted(&[(Mount::Claws, 60), (Mount::Bezel, 40)]);
    let mut first: Option<GemForm> = None;
    let mut settings = Vec::new();
    for index in 0..count {
        let mut gem = None;
        if rng.range_u64(0, 100) >= EMPTY_PERCENT {
            let form = match first {
                Some(same) if rng.range_u64(0, 100) < SAME_GEM_PERCENT => same,
                _ => roll_gem(rng, quality, false, ask.gem).0,
            };
            first.get_or_insert(form);
            let gem_size = roll_gem_size(rng, setting_size);
            gem = Some(new_gem(form, gem_size, roll_gem_quality(rng, quality)));
        }
        settings.push(Setting {
            angle: setting_angle(index, count),
            size: setting_size,
            mount,
            gem,
        });
    }
    RingForm {
        metal,
        profile,
        width,
        pattern,
        settings,
    }
}

/// How many settings, by how often. Most rings of poor metal have none; a finer metal
/// and a better make have more.
pub(super) fn setting_counts(metal: Material, quality: Quality) -> [(u8, u64); 5] {
    let fine = metal.tier() + quality as u64;
    [
        (0, 60),
        (1, fine * 8),
        (2, fine * 3),
        (3, fine),
        // Only the very finest. Whole-number division leaves the rest without.
        (5, fine * fine / 16),
    ]
}

/// In a row across the top of the band, evenly either side of it.
pub(super) fn setting_angle(index: u8, count: u8) -> u8 {
    let from_first = index as i32 * SETTING_SPACING as i32;
    let row = (count as i32 - 1) * SETTING_SPACING as i32;
    (from_first - row / 2).rem_euclid(256) as u8
}

/// Usually made for the setting, sometimes smaller, and rarely too big.
pub(super) fn roll_gem_size(rng: &mut Rng, setting: Size) -> Size {
    let roll = rng.range_u64(0, 100);
    if roll < OVERSIZED_PERCENT {
        setting.stepped(1)
    } else if roll < OVERSIZED_PERCENT + UNDERSIZED_PERCENT {
        setting.stepped(-1)
    } else {
        setting
    }
}

/// Near the ring's, which says how well it was made rather than what is in it. Only
/// rolled artifacts are named, so set gems stop below that.
pub(super) fn roll_gem_quality(rng: &mut Rng, ring: Quality) -> Quality {
    let step: i32 = rng.pick_weighted(&[(-1, 25), (0, 60), (1, 15)]);
    let best = Quality::Legendary as i32;
    Quality::ALL[(ring as i32 + step).clamp(0, best) as usize]
}
