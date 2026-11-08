//! What players and mobs share in a fight on the server: an ability winding up,
//! and counting attacks. The rules are `shared::combat`'s; who does what is in
//! `connections/combat.rs` (players) and `entities/fight.rs` (mobs).

use shared::combat::Swing;
use shared::creature::CreatureKind;
use shared::id::Id;
use shared::rng::{Rng, Seed};
use shared::skill::{SkillId, Windows};
use shared::vitals::{Limits, Vital, Vitals};

/// What an ability winding up will do when it lands.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Act {
    /// An attack.
    Strike(Swing),
    /// A Heal (players only, so far).
    Heal,
}

/// An ability begun and not landed yet.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Pending {
    /// Who it is aimed at (a player healing themselves: their own id).
    pub target: Id,
    pub act: Act,
    /// The action tick it lands at.
    pub lands_at: u64,
}

/// One creature's part in fighting: what it is winding up, and how many
/// attacks it has made (so hands take turns).
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Fight {
    pub pending: Option<Pending>,
    pub attacks: u64,
    /// A Heal that landed on another player, given once every client has
    /// had its turn (`connections::heal_players`).
    pub heal_out: Option<Id>,
}

/// A player's open practice windows (`shared::skill::Windows`): one per skill, and
/// one per kind of creature for the blocks spent fighting it; and whether they
/// sneak, with their recent steps for Sneaking's pace. Not saved.
#[derive(Clone, Debug, Default)]
pub(crate) struct Learning {
    pub skills: Windows<SkillId>,
    pub foes: Windows<CreatureKind>,
    pub sneak: crate::connections::Sneak,
}

/// A creature harmed this tick, for others who see it (`ServerWorld::harmed`).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Harm {
    /// Who was harmed.
    pub id: Id,
    /// Health left, per mille.
    pub left: u16,
    /// The player told already, by `AbilityLanded`: the one who struck, or
    /// the one struck.
    pub told: Id,
}

/// How many action ticks this much energy takes to come back, at this speed.
pub(crate) fn ticks_for(energy: u32, speed: u32) -> u64 {
    u64::from(energy).div_ceil(u64::from(speed.max(1)))
}

/// The luck of a blow, the same whoever asks: from the tick and who is in it.
pub(crate) fn luck(tick: u64, attacker: Id, defender: Id) -> Rng {
    let (a, b) = attacker.0.as_u64_pair();
    let (c, d) = defender.0.as_u64_pair();
    let hash = Seed::new(tick)
        .derive("blow")
        .at(&[a as i64, b as i64, c as i64, d as i64]);
    Seed::new(hash).rng()
}

/// How much Health is left, per mille of the most: for health bars.
pub(crate) fn health_left(vitals: &Vitals, limits: Limits, now: u64) -> u16 {
    let most = u64::from(limits.health.max(1)) * 1000;
    let left = u64::from(vitals.at(Vital::Health, limits, now).milli());
    (left * 1000 / most).min(1000) as u16
}
