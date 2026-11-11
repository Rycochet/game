//! The Generate POI panel's parts, and new seeds.

use bevy::prelude::*;
use shared::poi::PoiKind;
use shared::rng::Seed;

/// Open the Generate POI panel for this kind (the submenu chose it).
#[derive(Message, Clone, Copy, Debug)]
pub(in crate::debug_menu) struct GenerateKind(pub PoiKind);

/// What the Generate POI panel is set to, besides its seed box.
#[derive(Resource, Default)]
pub(in crate::debug_menu) struct GenerateForm {
    pub(super) kind: Option<PoiKind>,
    pub(super) ruin: u8,
    pub(super) seeds_made: i64,
}

#[derive(Component, Clone, Copy)]
pub(in crate::debug_menu) struct RuinButton(pub(super) u8);

#[derive(Component)]
pub(in crate::debug_menu) struct SeedBox;

#[derive(Component)]
pub(in crate::debug_menu) struct NewSeedButton;

#[derive(Component)]
pub(in crate::debug_menu) struct GenerateButton;

/// The most digits of a seed the new seed button makes: enough for many, and
/// short enough to write down and type in again.
pub(super) const NEW_SEEDS: u64 = 1_000_000_000;

/// A seed to try. `now` is any number that keeps changing, `made` how many so far.
pub(super) fn new_seed(now: u64, made: i64) -> u64 {
    Seed::new(now).at(&[made]) % NEW_SEEDS
}
